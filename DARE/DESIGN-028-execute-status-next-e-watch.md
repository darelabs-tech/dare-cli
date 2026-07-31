# DESIGN: Execute — status, next e watch (Microplano 028)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (Blueprint gerado; aguarda aprovação humana do Blueprint)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/028-execute-status-next-e-watch.md`  
> **Referência:** Microplano **026** (ranks / state / canvas / `next_executable`) · **020** (validate / load DAG) · **007** (`DagDocument` / limits) · **004** (saída / exit codes) · **005** (path safety) · Documento Mestre §2.1 / §25 · baseline TS 3.18.1 · skills `/dare-dag-run`, `/dare-dag-run-parallel`  
> **Posição:** 28 de 56  
> **Arquivo:** `DARE/DESIGN-028-execute-status-next-e-watch.md`  
> **Escopo deste ciclo apenas:** superfície **`dare execute`** nas ações **`--status`** (default), **`--next`** e **`--watch`** — navegação e observação determinísticas do DAG. **Não** `--complete` / `--fail` / `--reset` / Ralph (→ **029**). **Não** `--agent` / worktrees / budget (→ **030+**).

---

## 1. DESCRIÇÃO

Este Design cobre a primeira fatia do orquestrador **`dare execute`**: permitir que humanos e agentes IDE **inspecionem** o progresso do DAG (`--status`), **obtenham as próximas tasks prontas** com prompts compostos (`--next`), e **observem** o canvas/estado sem mutar runtime (`--watch`).

O problema: o runtime library-first já existe em `dare-dag` (026 — ranks, `ensure_state`, cascading skip, canvas, `next_executable`), mas a CLI ainda não expõe a superfície que as skills `/dare-dag-run*` e o TS 3.18.1 usam para orquestrar. Sem `--status`/`--next`/`--watch`, o ciclo Execute não arranca de forma determinística no binário nativo.

Quem consome: engenheiros a acompanhar o canvas; agentes Antigravity/Cursor/Claude/Codex que chamam `dare execute --next` e consomem o prompt; CI/smokes que assertam JSON e exit codes. Entrega: `crates/dare-dag/src/execution.rs` + `crates/dare-cli/src/commands/execute.rs`, fixtures, docs + DEC (sugerido **DEC-029**).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | `--status` default | `dare execute` sem flags ≡ `--status`; exit 0 com resumo | Smoke |
| O-02 | `--next` menor rank | Só tasks do **menor** rank entre candidatas `next_executable` | Unit + smoke |
| O-03 | Prompt composto | `subtask_prompt` + secção pais; cada output de pai ≤ `parent_context_chars` | Unit (cap exacto) |
| O-04 | `--watch` read-only | Após watch: `state.json` bytes/mtime inalterados (e sem writes de transição) | Integração FS |
| O-05 | Canvas | `--status` e `--next` refrescam `DARE/.canvas.md` de forma determinística | Snapshot / mtime |
| O-06 | JSON | `--json` envelope 004 com `data` tipado por ação | Smoke |
| O-07 | DAG vazio | 0 tasks → mensagem estável + exit documentado (Blueprint) | Unit |
| O-08 | DAG bloqueado | Sem ready + PENDING restantes bloqueados → mensagem “blocked”/equivalente | Unit |
| O-09 | All resolved | Sem PENDING/RUNNING elegíveis → “All tasks resolved” | Smoke |
| O-10 | Ralph + docs | fmt/clippy/test (+ audit se deps) + `cli-execute-status.md` (nome Blueprint) + DEC | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Orquestração Execute no alpha Rust (Ciclo 7) |
| Tech Lead | Time DARE CLI Rust | Escopo 028 só; não puxar Ralph/agent |
| Engenheiro CLI | Time implementação | `execution.rs` + `commands/execute.rs` |
| Usuário Final | Devs | `dare execute --next` / canvas ao vivo |
| Agentes IDE | 4 harnesses | Skills dag-run / parallel |
| Compat | Baseline TS 3.18.1 | Diffs classificados (DEC) |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo domínio | MUST | `crates/dare-dag/src/execution.rs` com APIs públicas de compose / status view (Blueprint congela nomes) |
| RF-02 | CLI `dare execute` | MUST | `crates/dare-cli/src/commands/execute.rs` + wiring `main.rs`; ações **mutuamente exclusivas** |
| RF-03 | Default `--status` | MUST | Sem `--next`/`--watch`/`--complete`… → comportamento status |
| RF-04 | Flag `--dag` | MUST | Default `DARE/dare-dag.yaml`; jail 005; missing → NotFound 3 |
| RF-05 | Project root | MUST | `find_project_root`; ausente → InvalidInput 4 |
| RF-06 | Load + ensure | MUST | `load_dag` + `ensure_state` (merge PENDING + cascade) antes de status/next |
| RF-07 | `--status` human | MUST | Resumo: counts DONE/RUNNING/PENDING/FAILED/SKIPPED; progresso; lista ordenada rank↑/id (alinhar canvas 026) |
| RF-08 | `--status` canvas | MUST | Escreve/atualiza `DARE/.canvas.md` via `canvas::write` após ensure |
| RF-09 | `--next` cascade | MUST | Aplicar cascading skip (via ensure/transition path 026) antes de selecionar ready |
| RF-10 | `--next` seleção | MUST | Candidatas = `next_executable`; **filtrar ao menor rank** presente; ordenar id lexico dentro do rank |
| RF-11 | `--next` human | MUST | Por task: id, title, complexity, rank, `spec_file` se houver, **prompt composto** |
| RF-12 | Compose prompt | MUST | Base = `subtask_prompt` da task; anexar “Upstream context” / pais com **tail** do `output` de cada dep `DONE`, cada snippet ≤ `limits.parent_context_chars` (default 2000; 0 inválido já rejeitado em validate 020) |
| RF-13 | Ordem dos pais no prompt | MUST | Pais em ordem **id lexico** das deps (ou ordem `depends_on` — **Blueprint congela uma** regra) |
| RF-14 | Cap total | SHOULD | Cap opcional do prompt total (além do per-parent); se omitido no Blueprint, só per-parent |
| RF-15 | `--next` zero ready | MUST | Mensagem estável: all resolved **ou** blocked (distinguir: existe PENDING inelegível vs nada PENDING/RUNNING) |
| RF-16 | `--watch` | MUST | Loop ou single-shot documentado: imprime snapshot status/canvas **sem** `transition` e **sem** mutar `state.json` (aceite microplano: “Watch nao altera estado”) |
| RF-17 | `--watch` intervalo | SHOULD | Flag `--interval` / default (ex. 2s) — **Blueprint congela**; Ctrl+C / cancel → exit 0 limpo |
| RF-18 | `--watch` canvas | SHOULD | Pode **ler** `.canvas.md` / state; **não** obrigado a reescrever; se reescrever for idêntico bit-a-bit, preferir **zero write** |
| RF-19 | JSON `--status` | MUST | `data`: counts, tasks[{id,status,rank,…}], `dag`, `canvasPath` |
| RF-20 | JSON `--next` | MUST | `data`: `ready: [{id, title, rank, complexity, specFile, prompt}]`, `rank`, `blocked`/`resolved` flags |
| RF-21 | JSON `--watch` | SHOULD | Mesmo shape que status por tick **ou** stream documentado; Blueprint congela |
| RF-22 | DAG vazio | MUST | `tasks.len()==0` → não panic; mensagem en-US; exit **0 ou 4** (Blueprint) |
| RF-23 | DAG bloqueado | MUST | Após cascade, 0 ready e ∃ PENDING → reportar blocked (deps FAILED/SKIPPED) |
| RF-24 | All resolved | MUST | 0 PENDING e 0 RUNNING (e sem ready) → “All tasks resolved” (texto estável para skills) |
| RF-25 | Parse / ciclo | MUST | YAML inválido → Config 4; ciclo em ranks → InvalidInput 4 (mensagem com `cycle`) |
| RF-26 | Lock contenção | MUST | `ensure_state` lock held → Io 5 (`file lock held`) — alinhar 026 |
| RF-27 | Zero Ralph | MUST | Nenhuma invocação de build/test/lint neste microplano |
| RF-28 | Sem Start implícito | MUST | `--next` **não** marca tasks RUNNING (Start fica em 029/`--agent` ou política Blueprint se divergir do TS — default: **não** Start) |
| RF-29 | Docs + DEC | MUST | `docs/compatibility/cli-execute-*.md` + **DEC-029** (sugerido; 028 já usado por viz) |
| RF-30 | Capability | SHOULD | Matrix: superfície `execute` em entry adequada (`dare-dag-run` / nova) com `cli_commands: ["execute"]` |
| RF-31 | Mensagens en-US | MUST | Erros e headlines de domínio em inglês |
| RF-32 | Smoke CLI | MUST | status/next/watch (+ empty/blocked/resolved) cobertos em `cli_smoke` ou equivalente |
| RF-33 | Flags exclusivas | MUST | Combinar `--status`+`--next` → Usage 2 (clap) |
| RF-34 | `--json` global | MUST | Envelope ADR-002 / 004 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (esboço — Blueprint congela)

```text
dare execute [--status] [--dag <path>]
dare execute --next [--dag <path>]
dare execute --watch [--dag <path>] [--interval <secs>]
# + globais --json / --no-color (004)
# flags 029+ (--complete/--fail/--reset/--agent) ABSENTES ou stub Usage neste ciclo
```

### API de domínio (esboço)

```text
// crates/dare-dag/src/execution.rs
pub fn ready_at_min_rank(doc, state, ranks) -> Vec<String>
  // next_executable filtrado ao min rank

pub fn compose_task_prompt(doc, state, task_id, parent_context_chars) -> Result<String, …>
  // subtask_prompt + upstream tails; nunca inclui secrets além do que já está em state.output

pub struct StatusSnapshot { counts, tasks: Vec<…>, ranks, … }
pub fn build_status_snapshot(doc, state, ranks) -> StatusSnapshot
```

### Contratos de disco

| Path | Papel | Mutação |
|------|-------|---------|
| `DARE/dare-dag.yaml` (ou `--dag`) | Input | **Read-only** |
| `.dare/state.json` | Runtime | **Read/merge** em status/next via `ensure_state`; **Read-only** em watch |
| `DARE/.canvas.md` | Observabilidade | **Write** em status/next; **preferir read-only** em watch |
| `.dare/state.json.darelock` | Lock | Adquirido só em paths que persistem state |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo DAG+state → mesmo `--next` order e prompts (tails estáveis) | Unit ×2 |
| RNF-02 | Performance | DAG ≤ 500 tasks: status/next < 500 ms típico (excl. watch sleep) | Smoke informal |
| RNF-03 | Disponibilidade | Funciona sem state prévio (`ensure_state` cria) | Integração |
| RNF-04 | Observabilidade | Erros tipados; correlation_id em `--json` | 004 |
| RNF-05 | Manutenibilidade | Domínio em `dare-dag`; CLI thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths + lock | CI 003 |
| RNF-07 | Cap I/O | Caps 007 ao ler DAG/state; parent tails capados | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dag` sob `ProjectRoot` / `SafeRelativePath` | OWASP A03 / 005 |
| RS-02 | Redact em erros/logs; não ampliar vazamento de secrets além do já persistido em `state.output` | OWASP A02 / 004 |
| RS-03 | CLI local: sem auth de rede; não expor state fora do project root | OWASP A01 |
| RS-04 | `cargo audit` / `deny` sem CVE HIGH/CRITICAL se deps novas | OWASP A06 |
| RS-05 | Secrets só via env já existentes — nenhum hardcoded | Supply chain |
| RS-06 | Sem shell; sem spawn de Ralph/processos neste ciclo | 006 / microplano |
| RS-07 | Cap `parent_context_chars` obrigatório (DoS de prompt) | Availability |
| RS-08 | Watch não corrompe state sob cancelamento | Integridade |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Domínio | `dare-dag` (+ `execution.rs`) | `0.1.0-alpha.0` |
| Runtime reuse | `ensure_state`, `next_executable`, `compute_ranks`, `canvas`, `TaskStatus` | **026** |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| Contratos | `dare-contracts` (`DagDocument`, `RuntimeStateV1`, limits) | 007 |
| Path / lock / atomic | `dare-core` | 005 |
| Root walk | `dare-project` | **só CLI** |
| Saída | OutputRenderer 004 | DEC-005 |
| Testes | tempfile + fixtures DAG 026 | workspace |
| Container | `Dockerfile.rust` + `docker-compose.ci.yml` | 003 |

**Deps novas:** nenhuma obrigatória além do workspace.

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem | Local | read/write | In/Out | DAG YAML; state; canvas | CLI / dare-dag |
| Agentes IDE | Consumidor | stdout / JSON | Out | prompts `--next` | Skills dag-run |
| Baseline TS 3.18.1 | Referência | — | In | flags / mensagens | Compat |
| Ralph / verify | — | — | — | **Fora** (029) | — |

---

## 9. RESTRIÇÕES

- **Pré-requisito:** microplano **026** concluído (state/canvas/`next_executable`).
- Mensagens en-US.
- Ações 028 **mutuamente exclusivas** com as de 029+ no clap (mesmo comando `execute`, flags faseadas).
- Não alterar schema `dare-dag.yaml` / `RuntimeStateV1` sem ADR.
- Diffs vs TS → DEC + classification matrix.
- `--watch` **MUST NOT** alterar estado (critério de aceite do microplano).
- Não implementar Ralph, Start-on-next (salvo decisão Blueprint explícita), agent, worktrees.

---

## 10. FORA DO ESCOPO (v1)

- `dare execute --complete` / `--fail` / `--reset` + Ralph + attempts (→ **029**).
- `dare execute --agent`, worktrees, budget, decay (→ **030–031**).
- Gate `dare review` / verify avançado (→ **029** / **032** / **049**).
- Ingestão GraphRAG pós-DONE (→ **029+** / **040+**).
- Mutação de `dare-dag.yaml` / refine / sub-DAG (→ **033**).
- Dashboard HTTP / REST execute (→ **051**).
- Paralelismo real de agentes (skill IDE; CLI só lista ready).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Diff vs TS (`--next` inicia RUNNING) | Alta | Médio | Default nativo: **não** Start; DEC-029 Classe B/C |
| R-02 | Ambiguidade “menor rank” vs lista multi-rank | Média | Alto | Filtro `min_rank` explícito + testes |
| R-03 | Watch escreve canvas “igual” e falha aceite | Média | Médio | Política zero-write em watch + teste mtime/hash |
| R-04 | Prompt explode com muitos pais | Média | Médio | Cap per-parent; SHOULD cap total |
| R-05 | Lock held em status/next paralelo | Média | Médio | Mensagem Io estável; doc fail-fast 026 |
| R-06 | Skills quebram se texto “All tasks resolved” mudar | Baixa | Alto | String canónica no Blueprint + golden |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF `--status` / `--next` / `--watch` + exclusividade aceites
- [ ] Regra “só menor rank” + compose `parent_context_chars` aceites
- [ ] Política watch **read-only** (sem mutar state) aceite
- [ ] Política Start-on-next (default: não) aceite ou alterada
- [ ] Fora de escopo 029+ (Ralph/agent) explícito
- [ ] Reuso 026 (`ensure_state`, `next_executable`, canvas) aceite
- [ ] Riscos R-01…R-06 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-028-execute-status-next-e-watch.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-dag/src/execution.rs` | Compose prompt + ready-at-min-rank (**criar**) |
| `crates/dare-dag/src/lib.rs` | `mod execution`; re-exports |
| `crates/dare-cli/src/commands/execute.rs` | Clap `execute` status/next/watch (**criar**) |
| `crates/dare-cli/src/main.rs` | Wiring |
| `tests/fixtures/dag/` | Fixtures empty / blocked / chain (reusar + criar) |
| `docs/compatibility/cli-execute-status.md` (nome final no Blueprint) | Docs (**criar**) |
| `docs/DECISION-LOG.md` | **DEC-029** (sugerido) |
| `assets/capability-matrix.yml` | `cli_commands: ["execute"]` (SHOULD) |

## Apêndice B — Estado atual (gap)

| Capacidade | Hoje | 028 |
|------------|------|-----|
| `ensure_state` / cascade / canvas | ✅ 026 | Reusar |
| `next_executable` | ✅ 026 | Filtrar min-rank + expor CLI |
| `compose_task_prompt` | ❌ | Implementar |
| `dare execute` CLI | ❌ | Implementar status/next/watch |
| Ralph / complete/fail/reset | ❌ | **029** |
| Docs DEC execute | ❌ | Criar |

## Apêndice C — Semântica `--next` (normativa pretendida)

```text
1. resolve root + dag
2. load_dag
3. ensure_state (lock → merge → cascade → save)  // pode escrever state+canvas
4. compute_ranks
5. ids = next_executable(doc, state, ranks)
6. if ids empty → resolved | blocked message
7. min_r = min rank(ids); ready = { id | rank(id)==min_r } sorted lexico
8. for each id: print compose_task_prompt(...)
```

Aceite microplano: **“Next retorna somente menor rank executavel.”**

## Apêndice D — Exit codes (alinhar 004)

| Code | Quando |
|------|--------|
| 0 | Status/next/watch OK (incl. resolved/blocked informativos — **ou** blocked→1; Blueprint congela) |
| 1 | Internal / domínio mapeado a falha (se Blueprint escolher blocked≠0) |
| 2 | Usage (flags exclusivas / args inválidos) |
| 3 | DAG NotFound |
| 4 | InvalidInput (root/jail) **ou** Config (YAML) **ou** ciclo |
| 5 | Io (lock held, write canvas/state) |

🟡 Exit de “blocked” vs “resolved”: preferência Design = **ambos exit 0** com mensagem distinta (skills parseiam texto/JSON). Blueprint confirma.

## Apêndice E — Aceite do microplano (mapeamento)

| Critério microplano | RF / O |
|---------------------|--------|
| Next só menor rank executável | O-02, RF-10, Apêndice C |
| Parent context respeita limite | O-03, RF-12, RS-07 |
| Watch não altera estado | O-04, RF-16, RS-08 |
| fmt/clippy/test | O-10 |
| Compat classificada | RF-29, R-01 |
| Canvas atualizado | O-05, RF-08 |
| JSON | O-06, RF-19…21 |
| DAG vazio e bloqueado | O-07, O-08, RF-22, RF-23 |

## Apêndice F — Próximos passos DARE

1. Revisar e **aprovar** este Design (humano).  
2. `/dare-blueprint` → `DARE/BLUEPRINT-028-execute-status-next-e-watch.md`.  
3. `/dare-tasks` → `TASKS-028` + `dare-dag-028.yaml` + `EXECUTION-028/`.  
4. Executar; ao closeout → [`029-execute-complete-fail-reset-e-ralph-inicial.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/029-execute-complete-fail-reset-e-ralph-inicial.md).
