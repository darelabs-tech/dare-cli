# DESIGN: DAG — parser, ranks e state store (Microplano 026)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (Blueprint gerado; aguarda aprovação humana do Blueprint)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/026-dag-parser-ranks-e-state-store.md`  
> **Referência:** Microplanos **005** (path + `FileLock`) · **007** (`RuntimeStateV1` / DAG YAML) · **020** (`dare-dag` validate + ciclos) · Documento Mestre §5.2 · §24 (Ciclo 6) · baseline TS 3.18.1  
> **Posição:** 26 de 56  
> **Arquivo:** `DARE/DESIGN-026-dag-parser-ranks-e-state-store.md`  
> **Escopo deste ciclo apenas:** núcleo de **grafo (ranks longest-path)**, **cascading skip**, **state store v1 com lock + escritas atómicas**, **canvas base** e **property tests**. Tudo que pertence a **027** (viz) / **028+** (execute CLI) fica em **Fora do Escopo**.

---

## 1. DESCRIÇÃO

Este Design cobre o **núcleo do DAG Runner** no crate `dare-dag` **antes** da execução de agentes e dos comandos `dare execute` / `dare dag viz`. O problema: o validate (020) já lê e valida `DARE/dare-dag.yaml` e detecta ciclos, mas ainda **não** calcula ranks, **não** gerencia `.dare/state.json` com transições seguras, **não** aplica cascading skip e **não** materializa o canvas ao vivo — peças sem as quais 027–029 não conseguem orquestrar.

A entrega é API de domínio em `crates/dare-dag/src/{graph,state,canvas}.rs` (mais testes/property tests/fixtures), reutilizando parse/`RuntimeStateV1` de **007**, `FileLock`/`atomic_write` de **005**, e detecção de ciclos de **020** (sem duplicar DFS). Quem consome: crates/comandos posteriores (`dag viz`, `execute`), CI, e developers que inspecionam estado/ranks via API/testes.

**Não** introduz a superfície CLI completa de execute/viz neste microplano — só o núcleo verificável (ranks ≡ baseline, crash não corrompe state, concorrência clara).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Ranks longest-path | Mesmo DAG → mesmos ranks por task (golden / property) | ≡ baseline / fixtures |
| O-02 | Ciclos | Grafo cíclico → erro tipado; reusa lógica 020 | Unit |
| O-03 | Cascading skip | Fixpoint: deps FAILED/SKIPPED → dependentes SKIPPED | Unit + property |
| O-04 | State store v1 | Load/save `.dare/state.json` via 007 + transitions API | Unit/integração FS |
| O-05 | File lock | Segunda aquisição concorrente → erro claro **ou** serializa | Unit/integração |
| O-06 | Transições atómicas | Crash a meio → state final válido ou anterior intacto | Integração (kill/simulado) |
| O-07 | Canvas base | Escreve `DARE/.canvas.md` (tabela + progresso) | Snapshot |
| O-08 | Property tests | ≥1 propriedade ranks + ≥1 skip | `cargo test` |
| O-09 | Path safety | Toda R/W sob `ProjectRoot` / `SafeRelativePath` | Unit |
| O-10 | Ralph + docs | fmt/clippy/test (+ audit se deps) + DEC + doc compat | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Núcleo DAG antes de agentes |
| Tech Lead | Time DARE CLI Rust | Escopo 026; não puxar viz/execute CLI |
| Engenheiro CLI | Time implementação | `dare-dag` graph/state/canvas |
| Usuário Final | Devs / agentes | Estado e ranks corretos para execute futuro |
| CI | Pipelines | Goldens + property tests |
| Compat | Baseline TS 3.18.1 | Ranks / state / skip alinhados ou classificados |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `graph` | MUST | `crates/dare-dag/src/graph.rs` com API pública documentada |
| RF-02 | Construir grafo a partir de `DagDocument` (007) | MUST | Tasks + `depends_on`; IDs únicos já assumidos pós-validate |
| RF-03 | Rank longest-path | MUST | Roots (`depends_on: []`) → **rank 0**; demais → `1 + max(rank(deps))`; memoizado; determinístico |
| RF-04 | Ordenação estável | MUST | Empates de rank: ordenar por `id` lexicográfico quando a API expõe listas |
| RF-05 | Ciclos no grafo de ranks | MUST | Se ciclo, **não** inventar ranks; retornar erro tipado reutilizando detecção 020 (ou helper partilhado) |
| RF-06 | Pré-condição validate | SHOULD | API de ranks documenta: caller deve validar DAG (020) antes; opcional helper `compute_ranks_validated` |
| RF-07 | Cascading skip (fixpoint) | MUST | Enquanto existir task não-SKIPPED com **alguma** dep em `{FAILED, SKIPPED}`, marcar SKIPPED; iterar até ponto fixo |
| RF-08 | Cascading skip — ordem | MUST | Resultado final independente da ordem de visita (determinístico) |
| RF-09 | Status canónicos | MUST | Enum/strings: `PENDING`, `RUNNING`, `DONE`, `FAILED`, `SKIPPED` (alinhar TS; Blueprint congela casing) |
| RF-10 | State store — load | MUST | Ler `.dare/state.json` via `dare_contracts::load_runtime_state`; `version != 1` → Config |
| RF-11 | State store — init | MUST | Se ausente: criar v1 com tasks do DAG em `PENDING` (e `dependsOn` espelhado); path default `.dare/state.json` |
| RF-12 | State store — transitions | MUST | API tipada para transições válidas (ex. PENDING→RUNNING, RUNNING→DONE/FAILED, *→SKIPPED via skip engine); inválidas → erro |
| RF-13 | `updatedAt` | MUST | Atualizar timestamp ISO-8601 em cada persistência bem-sucedida |
| RF-14 | File lock | MUST | Antes de mutar state: `FileLock::try_acquire` no path de state (ou sibling documentado); `WouldBlock` → Io claro |
| RF-15 | Escrita atómica | MUST | Persistência via `write_json_atomic` / `save_runtime_state` (007); sem truncate parcial observável |
| RF-16 | Crash safety | MUST | Teste: simular falha pós-lock pré-rename (se aplicável) → ficheiro anterior ou novo completo; nunca JSON truncado |
| RF-17 | Canvas base | MUST | `crates/dare-dag/src/canvas.rs` gera `DARE/.canvas.md` com título, tabela ID/Title/Status, `Progress: done/total`, barra ASCII |
| RF-18 | Canvas — write | MUST | Escrita sob jail 005; atómica ou write+replace documentado no Blueprint |
| RF-19 | Canvas — refresh | MUST | Função `refresh_canvas(root, dag, state)` chamável após transição (sem CLI execute) |
| RF-20 | Property tests | MUST | (a) ranks ≥ 0 e `rank(t) > rank(d)` para cada dep `d` de `t` em DAGs acíclicos; (b) skip é idempotente / fixpoint |
| RF-21 | Fixtures ranks/skip | MUST | ≥1 DAG diamond / chain / fan-out com golden ranks; ≥1 cenário cascading skip |
| RF-22 | Export API | MUST | `lib.rs` re-exporta graph/state/canvas públicos necessários a 027/028 |
| RF-23 | Docs + DEC | MUST | `docs/compatibility/dag-runtime.md` (ou nome Blueprint) + DEC no DECISION-LOG |
| RF-24 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-25 | CLI nova | COULD | Comando debug (`dare dag ranks` / similar) — **não** bloqueia aceite; 027/028 são os consumidores |
| RF-26 | Golden vs TS 3.18.1 | SHOULD | Comparar ranks em fixtures partilhadas; diffs → classification + DEC |
| RF-27 | `next_executable` helper | SHOULD | Dado state+ranks, listar tasks PENDING cujo deps ⊆ DONE, menor rank primeiro (prep 028; sem wiring execute) |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície de domínio (esboço — Blueprint congela assinaturas)

```text
dare_dag::graph::compute_ranks(doc: &DagDocument) -> Result<BTreeMap<String, u32>, DagGraphError>
dare_dag::graph::tasks_by_rank(ranks: &BTreeMap<...>) -> BTreeMap<u32, Vec<String>>

dare_dag::state::ensure_state(root, dag) -> CoreResult<RuntimeStateV1>
dare_dag::state::apply_cascading_skip(state: &mut RuntimeStateV1, dag: &DagDocument) -> usize  // n mudadas
dare_dag::state::transition(root, task_id, Transition) -> CoreResult<RuntimeStateV1>
  // adquire FileLock → load → validate → mutate → save_atomic → refresh_canvas opcional

dare_dag::canvas::render(dag, state) -> String
dare_dag::canvas::write(root, dag, state) -> CoreResult<()>
```

### Contratos de disco

| Path | Papel | Mutação neste microplano |
|------|-------|---------------------------|
| `DARE/dare-dag.yaml` | Grafo estático | **Read-only** |
| `.dare/state.json` | Runtime v1 | **Create / update** (atómico + lock) |
| `DARE/.canvas.md` | Canvas ao vivo | **Create / update** |

Qualquer alteração de schema/ID/exit → teste de compat + ADR se breaking.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo DAG+state → mesmos ranks, skip set e canvas (timestamps normalizados em golden) | Golden |
| RNF-02 | Performance | DAG ≤ 500 tasks: ranks + 1 skip fixpoint < 100 ms típico em debug | Smoke informal |
| RNF-03 | Concorrência | Dois writers: um sucede; outro falha com mensagem estável **ou** fila documentada | Teste |
| RNF-04 | Observabilidade | Erros tipados (`DagGraphError` / `CoreError`); sem panic em state corrupt | Unit |
| RNF-05 | Manutenibilidade | Sem dependência `dare-cli`; sem ciclo crate | Clippy / dep graph |
| RNF-06 | Compatibilidade | Win/macOS/Linux (lock + paths) | CI 003 |
| RNF-07 | Cap I/O | Respeitar caps 007 em load state/DAG | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths de state/canvas/DAG sob `ProjectRoot` | OWASP A03 / 005 |
| RS-02 | Não logar `output`/`error` completos de tasks em traces (truncar) | OWASP A02 |
| RS-03 | Lock + atomic write — sem TOCTOU óbvio no happy path | Integrity |
| RS-04 | `cargo audit` / `cargo deny` sem CVE HIGH/CRITICAL se deps novas | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell | Supply chain |
| RS-06 | Rejeitar `version != 1` no state; não “upcast” silencioso | Integrity / Mestre |
| RS-07 | Limitar tamanho de campos ao persistir (reusar caps / Blueprint) | Availability |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-dag` | `0.1.0-alpha.0` (estender) |
| Contratos | `dare-contracts` (`DagDocument`, `RuntimeStateV1`) | 007 |
| FS / lock | `dare-core` (`FileLock`, `ProjectRoot`, atomic write) | 005 |
| Validate/ciclos | `dare-dag::validate` | 020 |
| Serde / JSON | workspace | — |
| Property tests | `proptest` (ou crate já no workspace — Blueprint escolhe) | — |
| Testes FS | `tempfile` | workspace |
| Container | `Dockerfile.rust` + `docker-compose.ci.yml` | 003 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem | Local | read/write | In/Out | `dare-dag.yaml`, `.dare/state.json`, `DARE/.canvas.md` | `dare-dag` |
| Baseline TS 3.18.1 | Referência | — | In | ranks / skip / state shape | Compat |
| CLI execute/viz | Consumidor futuro | API Rust | Out | ranks, state, canvas | 027 / 028 |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **005, 007, 020** concluídos (código presente).
- Mensagens de erro de domínio em **en-US**.
- **Não** alterar schema `RuntimeStateV1` sem ADR + migration note.
- **Não** implementar `dare execute` / `dare dag viz` neste Design.
- Cascading skip **não** executa agentes — só muta status em memória/disco.
- Ranks **não** persistem no YAML; só derivados do DAG (e opcionalmente cache em memória).
- Tensão Mestre §24 vs §25: microplano **026** é a autoridade — inclui skip + store + canvas **base**; a **CLI** de execute permanece em **028**.
- Diffs intencionais vs TS → DEC + classification matrix.

---

## 10. FORA DO ESCOPO (v1)

- `dare dag viz` Mermaid/DOT/Excalidraw (→ **027**).
- `dare execute --status/--next/--complete/--fail/--reset/--watch` (→ **028–029**).
- Agent drivers, worktrees, budget, Ralph Loop completo (→ **029–031**).
- Sub-DAG / refine / REPLAN (→ **033**).
- GraphRAG ingest pós-DONE (→ **040+**).
- UI interativa / TTY approval por rank.
- Migração automática state v1 → v2.
- Alterar contrato `dare-dag.yaml` v2.1.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Ranks off-by-one vs TS (0 vs 1) | Alta | Alto | Congelar roots=0; golden + DEC |
| R-02 | Duplicar DFS de ciclos | Média | Médio | Extrair/reusar `collect_cycles` 020 |
| R-03 | Lock Windows vs Unix (fs4) | Média | Médio | Testes CI cross-platform; DEC-006 |
| R-04 | Skip infinite loop / ordem | Baixa | Alto | Fixpoint + property “monótono + termina” |
| R-05 | Canvas diverge do TS | Média | Baixo | Snapshot mínimo (tabela+progress); 027 refinará |
| R-06 | Escopo “execute” vaza para 026 | Alta | Alto | Checklist Fora de Escopo; review Blueprint |
| R-07 | State parcial se bypass atomic | Baixa | Alto | Só `save_runtime_state` / `write_json_atomic` |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF ranks (0-based longest-path) e RF cascading skip aceites
- [ ] Política lock + atomic write + crash safety aceite
- [ ] Canvas base (formato mínimo) aceite
- [ ] Fora de escopo 027/028+ explícito
- [ ] Reuso 005/007/020 (sem reinventar parse/lock/ciclos) aceite
- [ ] Riscos R-01…R-07 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-026-dag-parser-ranks-e-state-store.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-dag/src/graph.rs` | Ranks + helpers de grafo (**criar**) |
| `crates/dare-dag/src/state.rs` | Store, skip, transitions + lock (**criar**) |
| `crates/dare-dag/src/canvas.rs` | Render/write canvas (**criar**) |
| `crates/dare-dag/src/validate.rs` | Ciclos — **reusar** |
| `crates/dare-dag/src/lib.rs` | Re-exports |
| `crates/dare-contracts/src/state.rs` | `RuntimeStateV1` I/O |
| `crates/dare-contracts/src/dag.rs` | Parse DAG |
| `crates/dare-core/src/fs/lock.rs` | `FileLock` |
| `tests/fixtures/dag/` | Estender com ranks/skip |
| `crates/dare-contracts/tests/fixtures/state.v1.json` | Fixture state |
| `docs/compatibility/dag-runtime.md` | Docs (**criar**; nome final no Blueprint) |
| `docs/DECISION-LOG.md` | DEC-027 (nº a confirmar no Blueprint) |

## Apêndice B — Estado atual (gap)

| Capacidade | Hoje | 026 |
|------------|------|-----|
| Parse YAML / validate / ciclos | ✅ 007+020 | Reusar |
| `RuntimeStateV1` load/save | ✅ 007 | Envolver com transitions+lock |
| `FileLock` | ✅ 005 | Usar no state store |
| Ranks longest-path | ❌ | Implementar |
| Cascading skip | ❌ | Implementar |
| Canvas writer | ❌ (só artefatos manuais) | Implementar base |
| Property tests | ❌ | Adicionar |
| `dare execute` / `dag viz` | ❌ | **Fora** |

## Apêndice C — Algoritmos (congelar no Blueprint)

### C.1 Longest-path rank

```text
rank(t) = 0                          if depends_on(t) = ∅
rank(t) = 1 + max_{d ∈ deps(t)} rank(d)   otherwise
```

Memoização DFS; ciclo → erro (não `u32::MAX`).

### C.2 Cascading skip (fixpoint)

```text
repeat:
  for each task t where status(t) ∉ {SKIPPED, DONE}:  // Blueprint: DONE nunca vira SKIPPED
    if ∃ d ∈ depends_on(t) with status(d) ∈ {FAILED, SKIPPED}:
      status(t) ← SKIPPED
until no change
```

🟡 Detalhe: tasks `RUNNING` atingidas por skip — **Blueprint congela** (preferência: não skip automático de RUNNING; ou forçar SKIPPED — alinhar TS).

### C.3 Canvas mínimo

Espelhar estrutura observada em `DARE/.canvas.md`:

- Título `# DARE DAG Execution — …`
- `**Updated:**` ISO
- Tabela `| ID | Title | Status | … |`
- `## Progress: X/Y tasks (P%)` + barra `█`/`░`

## Apêndice D — Aceite do microplano (mapeamento)

| Critério microplano | RF / O |
|---------------------|--------|
| Ranks ≡ baseline | O-01, RF-03, RF-26 |
| Crash não corrompe state | O-06, RF-15, RF-16 |
| Concorrência falha ou serializa | O-05, RF-14, RNF-03 |
| fmt/clippy/test | O-10 |
| Compat classificada | RF-26, R-01 |
| Property tests | RF-20 |

## Apêndice E — Próximos passos DARE

1. Revisar e **aprovar** este Design (humano).  
2. `/dare-blueprint` → `DARE/BLUEPRINT-026-dag-parser-ranks-e-state-store.md`.  
3. `/dare-tasks` → `TASKS-026` + `dare-dag-026.yaml` + `EXECUTION-026/`.  
4. Executar tasks; ao closeout → microplano [`027-dag-visualizacao.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/027-dag-visualizacao.md).
