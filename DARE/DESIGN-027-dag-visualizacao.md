# DESIGN: DAG — visualização (`dare dag viz`) (Microplano 027)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (Blueprint gerado; aguarda aprovação humana do Blueprint)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/027-dag-visualizacao.md`  
> **Referência:** Microplano **026** (ranks / `dare-dag`) · **020** (validate / load) · **005** (path safety / atomic write) · **007** (`DagDocument`) · Documento Mestre §24 · baseline TS 3.18.1 · skill `/dare-dag-viz`  
> **Posição:** 27 de 56  
> **Arquivo:** `DARE/DESIGN-027-dag-visualizacao.md`  
> **Escopo deste ciclo apenas:** comando **`dare dag viz`** nos formatos **Mermaid**, **DOT** e **Excalidraw**, com `--dag` / `--format` / `--output`, ordenação determinística e goldens. **Não** `dare execute` (028+).

---

## 1. DESCRIÇÃO

Este Design cobre a superfície **`dare dag viz`**: ler um `dare-dag.yaml`, calcular ranks (026), e emitir uma visualização **determinística** do grafo estático em um dos três formatos suportados — Mermaid (`.mmd`), Graphviz DOT (`.dot`) ou Excalidraw (`.excalidraw`) — para stdout ou ficheiro via `--output`.

Resolve a lacuna entre o runtime DAG já disponível em biblioteca (026) e a UX de inspeção usada por developers e skills IDE (`/dare-dag-viz`, refine, runner `--viz`). Quem consome: humanos a revisar o plano, agentes que regeneram `DARE/dag-graph.*`, e CI/smoke que comparam goldens.

Entrega: `crates/dare-dag/src/viz.rs` + `crates/dare-cli/src/commands/dag.rs`, fixtures/goldens, docs + DEC (nº no Blueprint; sugerido **DEC-028**).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Mermaid | Fixture → string/file golden estável | Snapshot bit-a-bit (EOL normalizado) |
| O-02 | DOT | Idem | Snapshot |
| O-03 | Excalidraw | JSON válido + golden estável (ordem de elementos fixa) | Snapshot + parse JSON |
| O-04 | `--format` | `mermaid` \| `dot` \| `excalidraw` (aliases Blueprint) | Clap + unit |
| O-05 | `--dag` | Default `DARE/dare-dag.yaml`; path jail 005 | Smoke |
| O-06 | `--output` / `-o` | Write atómico sob project root; ausente → stdout | Smoke |
| O-07 | Determinismo | Mesmo DAG → mesmos nós/edges ordenados (id lexico) | Golden ×2 |
| O-08 | Ranks nas viz | Layout/agrupamento usa `compute_ranks` 026 | Unit |
| O-09 | Path safety | Output fora do root → InvalidInput | Unit |
| O-10 | Ralph + docs | fmt/clippy/test (+ audit se deps) + `cli-dag-viz.md` + DEC | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Inspeção visual do plano (Ciclo 6) |
| Tech Lead | Time DARE CLI Rust | Escopo 027; não puxar execute |
| Engenheiro CLI | Time implementação | `viz.rs` + `commands/dag.rs` |
| Usuário Final | Devs | `dare dag viz -f mermaid -o …` |
| Agentes IDE | 4 harnesses | Capability `dare-dag-viz` / slash |
| Compat | Baseline TS 3.18.1 | Diffs classificados |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `viz` | MUST | `crates/dare-dag/src/viz.rs` com API pública |
| RF-02 | CLI `dare dag viz` | MUST | Subcomando clap em `commands/dag.rs`; wired em `main.rs` |
| RF-03 | `--format` / `-f` | MUST | Valores: `mermaid`, `dot`, `excalidraw` (case-insensitive **ou** exact — Blueprint congela) |
| RF-04 | Default format | MUST | Se omitido: **`mermaid`** (alinhar Mestre / uso comum `-o *.mmd`) |
| RF-05 | `--dag <path>` | MUST | Default relativo `DARE/dare-dag.yaml` sob project root; relativo/absoluto sob jail 005 |
| RF-06 | `--output` / `-o <path>` | MUST | Se presente: escreve ficheiro (atomic_write); se ausente: imprime body em **stdout** (human); `--json` envelope 004 com body se aplicável |
| RF-07 | Project root | MUST | `find_project_root`; ausente → InvalidInput 4 |
| RF-08 | DAG ausente | MUST | NotFound 3 |
| RF-09 | DAG parse inválido | MUST | Config 4 (007) |
| RF-10 | Gerar Mermaid | MUST | `flowchart`/`graph` com nós = task ids; edges `dep --> task` (ou inverso documentado); ranks opcionalmente como subgraph — **Blueprint congela sintaxe** |
| RF-11 | Gerar DOT | MUST | `digraph` com nós/edges; ids escapados se necessário |
| RF-12 | Gerar Excalidraw | MUST | JSON `.excalidraw` v2 compatível com excalidraw.com; retângulos por task; setas = deps; layout por rank (colunas) |
| RF-13 | Ordenação nós | MUST | Emitir nós em ordem **id lexicográfica** (ou rank↑ depois id — Blueprint congela **uma** regra e aplica aos 3 formatos) |
| RF-14 | Ordenação edges | MUST | Edges ordenados por `(from, to)` lexico |
| RF-15 | Ranks | MUST | Usar `dare_dag::compute_ranks` (ou validated); ciclo → erro tipado (não emitir viz parcial) |
| RF-16 | Labels | MUST | Nó mostra `id` e, se couber, `title` truncado (cap Blueprint, ex. 40 chars) |
| RF-17 | Complexity visual (Excalidraw) | SHOULD | Cores por complexity LOW/MED/HIGH (skill: azul/laranja/rosa) |
| RF-18 | Status visual | SHOULD | Se `.dare/state.json` existir e for v1 válido, colorir por status; senão tratar todos como PENDING (sem falhar) |
| RF-19 | Zero writes no DAG/state | MUST | Só escreve `--output` (e nunca muta `dare-dag.yaml` / state) |
| RF-20 | Goldens | MUST | ≥1 fixture DAG × 3 formatos sob `tests/fixtures/dag/viz/` (ou similar) |
| RF-21 | Escape / sanidade | MUST | IDs Mermaid/DOT seguros (sem injeção de sintaxe); titles escapados |
| RF-22 | Capability | SHOULD | `dare-dag-viz` com `cli_commands` incluindo superfície `dag`/`viz` (forma exacta no Blueprint) |
| RF-23 | Docs + DEC | MUST | `docs/compatibility/cli-dag-viz.md` + DEC no DECISION-LOG (**DEC-028** sugerido) |
| RF-24 | Mensagens en-US | MUST | Erros de domínio/CLI em inglês |
| RF-25 | Format desconhecido | MUST | Usage exit 2 (clap) |
| RF-26 | Smoke CLI | MUST | `dare dag viz -f mermaid` exit 0; `-o` cria ficheiro; dag missing → 3 |
| RF-27 | `--json` | SHOULD | Envelope ok; `data` contém `format`, `dag`, `outputPath` opcional, `body` se stdout |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (esboço — Blueprint congela)

```text
dare dag viz [--dag <path>] [--format|-f mermaid|dot|excalidraw] [--output|-o <path>]
# + globais --json / --no-color (004)
```

### API de domínio (esboço)

```text
dare_dag::viz::VizFormat { Mermaid, Dot, Excalidraw }
dare_dag::viz::render(doc: &DagDocument, format: VizFormat, opts: &VizOptions) -> Result<String, DagGraphError>
  // opts: include_status from RuntimeStateV1?, title_max, …
dare_dag::viz::write_output(root, rel, body) -> CoreResult<()>  // ou CLI usa atomic_write direto
```

### Contratos de disco

| Path | Papel | Mutação |
|------|-------|---------|
| `DARE/dare-dag.yaml` (ou `--dag`) | Input | **Read-only** |
| `.dare/state.json` | Status opcional (SHOULD) | **Read-only** |
| path `--output` | Artefacto viz | **Create/overwrite** (atómico) |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo input → mesmo output (sem timestamps nos goldens Mermaid/DOT; Excalidraw sem campos voláteis ou normalizados) | Golden |
| RNF-02 | Performance | DAG ≤ 500 tasks: render < 500 ms típico debug | Smoke informal |
| RNF-03 | Disponibilidade | Funciona sem state.json | Unit |
| RNF-04 | Observabilidade | Erros tipados; sem panic em YAML malformado | Unit |
| RNF-05 | Manutenibilidade | Domínio em `dare-dag`; CLI thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths | CI 003 |
| RNF-07 | Cap I/O | Caps 007 ao ler DAG/state | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dag` e `--output` sob `ProjectRoot` / `SafeRelativePath` | OWASP A03 / 005 |
| RS-02 | Não incluir `subtask_prompt` / secrets no diagrama | OWASP A02 |
| RS-03 | Truncar titles longos; escapar caracteres especiais de formato | Injection |
| RS-04 | `cargo audit` / `deny` sem CVE HIGH/CRITICAL se deps novas | OWASP A06 |
| RS-05 | Sem shell ao gerar viz | Supply chain |
| RS-06 | Não seguir symlinks de escape no write | 005 |
| RS-07 | Cap de tamanho do output gerado (rejeitar ou truncar documentado) | Availability |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Domínio | `dare-dag` (+ `viz.rs`) | `0.1.0-alpha.0` |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| Parse / ranks | `dare-contracts` + `compute_ranks` 026 | — |
| Path / atomic | `dare-core` | 005 |
| Root walk | `dare-project` | **só CLI** |
| JSON (Excalidraw) | serde_json | workspace |
| Saída | OutputRenderer 004 | DEC-005 |
| Testes | tempfile + goldens | workspace |
| Container | `Dockerfile.rust` + `docker-compose.ci.yml` | 003 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem | Local | read/write | In/Out | DAG YAML; opcional state; ficheiro `-o` | CLI / viz |
| Mermaid preview / VS Code | Consumidor | ficheiro | Out | `.mmd` | Utilizador |
| Graphviz | Consumidor | ficheiro | Out | `.dot` | Utilizador |
| excalidraw.com | Consumidor | ficheiro JSON | Out | `.excalidraw` | Utilizador |
| Baseline TS 3.18.1 | Referência | — | In | formatos / flags | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisito:** microplano **026** concluído (ranks disponíveis).
- Mensagens en-US.
- Visualização do **grafo estático** do YAML — não substitui canvas runtime (026) nem execute (028).
- Não alterar schema `dare-dag.yaml` / `RuntimeStateV1`.
- Diffs vs TS → DEC + classification matrix.
- Excalidraw: sem dependência npm runtime; gerar JSON nativamente em Rust.

---

## 10. FORA DO ESCOPO (v1)

- `dare execute --status/--next/--watch/…` (→ **028–029**).
- Regeneração automática do DAG / refine (→ **033**).
- `dare graph viz` (GraphRAG — ciclo distinto).
- UI interativa / servidor HTTP de preview.
- Layout force-directed complexo além de colunas por rank.
- Export PNG/SVG rasterizado.
- Mutação de state ou YAML.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Excalidraw schema drift | Média | Médio | Golden mínimo + doc campos usados; versão type fixa |
| R-02 | Diff vs TS (ordem edges / subgraph) | Alta | Médio | DEC-028; SoT nativo + classification |
| R-03 | IDs inválidos em Mermaid/DOT | Média | Alto | Escape/`id` sanitizer partilhado |
| R-04 | Goldens flaky (EOL CRLF) | Média | Médio | Normalizar `\n` nos asserts |
| R-05 | State ausente / corrupt | Baixa | Baixo | SHOULD: ignorar state inválido com warn opcional |
| R-06 | Output path escape | Baixa | Alto | Jail 005 + testes |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF formatos Mermaid/DOT/Excalidraw + flags `--dag`/`--format`/`--output` aceites
- [ ] Regra única de ordenação nós/edges aceite
- [ ] Política state/status (SHOULD) aceite ou deferida
- [ ] Fora de escopo execute 028+ explícito
- [ ] Reuso 026 ranks / 020 load / 005 paths aceite
- [ ] Riscos R-01…R-06 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-027-dag-visualizacao.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-dag/src/viz.rs` | Render Mermaid/DOT/Excalidraw (**criar**) |
| `crates/dare-dag/src/lib.rs` | Re-exports |
| `crates/dare-cli/src/commands/dag.rs` | Clap `dag viz` (**criar**) |
| `crates/dare-cli/src/main.rs` | Wiring |
| `tests/fixtures/dag/viz/` | Goldens (**criar**) |
| `docs/compatibility/cli-dag-viz.md` | Docs (**criar**) |
| `docs/DECISION-LOG.md` | DEC-028 (sugerido) |
| `assets/capability-matrix.yml` | `dare-dag-viz` cli_commands (SHOULD) |

## Apêndice B — Estado atual (gap)

| Capacidade | Hoje | 027 |
|------------|------|-----|
| `compute_ranks` / load DAG | ✅ 026/007 | Reusar |
| `viz.rs` | ❌ | Implementar |
| `dare dag viz` CLI | ❌ | Implementar |
| Goldens 3 formatos | ❌ | Criar |
| Docs DEC | ❌ | Criar |

## Apêndice C — Convenções visuais (referência skill; Blueprint congela)

| Complexity | Excalidraw fill (skill) |
|------------|-------------------------|
| LOW | `#e3f2fd` |
| MED | `#fff3e0` |
| HIGH | `#fce4ec` |

| Status | Indicação |
|--------|-----------|
| PENDING | cinza / stroke normal |
| RUNNING | azul / stroke pontilhado |
| DONE | verde |
| FAILED | vermelho |
| SKIPPED | (Blueprint: cinza tracejado ou omitir destaque) |

Posicionamento Excalidraw (skill): ~120×60px; ranks em colunas; setas = `depends_on`.

## Apêndice D — Exit codes (alinhar 004 / validate)

| Code | Quando |
|------|--------|
| 0 | Viz gerada (stdout ou ficheiro) |
| 1 | Internal / erro de domínio não mapeado |
| 2 | Usage (format inválido via clap) |
| 3 | DAG NotFound |
| 4 | InvalidInput (root/jail/output) **ou** Config (YAML parse) **ou** ciclo/`DagGraphError` |
| 5 | Io ao ler/escrever |

🟡 Mapa exacto ciclo → 4 vs 1: **Blueprint congela** (preferência: **4** `invalid_input`).

## Apêndice E — Aceite do microplano (mapeamento)

| Critério microplano | RF / O |
|---------------------|--------|
| Golden files estáveis | O-01…O-03, RF-20, RNF-01 |
| Formatos abrem nas tools | RF-10…12; smoke manual/doc |
| Paths output seguros | O-09, RF-06, RS-01 |
| fmt/clippy/test | O-10 |
| Compat classificada | RF-23, R-02 |

## Apêndice F — Próximos passos DARE

1. Revisar e **aprovar** este Design (humano).  
2. `/dare-blueprint` → `DARE/BLUEPRINT-027-dag-visualizacao.md`.  
3. `/dare-tasks` → `TASKS-027` + `dare-dag-027.yaml` + `EXECUTION-027/`.  
4. Executar; ao closeout → [`028-execute-status-next-e-watch.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/028-execute-status-next-e-watch.md).
