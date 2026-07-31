# DESIGN: Execute — complete, fail, reset e Ralph inicial (Microplano 029)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (Blueprint gerado; aguarda aprovação humana do Blueprint)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/029-execute-complete-fail-reset-e-ralph-inicial.md`  
> **Referência:** Microplano **028** (`dare execute` status/next/watch · DEC-029) · **026** (`transition` / `ensure_state` / cascade / canvas) · **006** (`SafeCommand` / timeout / exit 124) · **004** (saída / `ErrorKind`) · **005** (path safety) · Documento Mestre §2.1 / §5.2 / §25 · baseline TS 3.18.1 · skill `/dare-execute`  
> **Posição:** 29 de 56  
> **Arquivo:** `DARE/DESIGN-029-execute-complete-fail-reset-e-ralph-inicial.md`  
> **Escopo deste ciclo apenas:** transições CLI **`--complete`**, **`--fail`**, **`--reset`** + **Ralph Loop inicial** (build → test → lint por stack, timeout 600 s). **Não** `--agent` / worktrees / budget (→ **030+**). **Não** review anti-stub completo / mutation / formal / best-of-N (→ **032** / **049**). **Não** GraphRAG avançado (→ **040+**).

---

## 1. DESCRIÇÃO

Este Design fecha a fatia determinística do orquestrador **`dare execute`**: permitir que humanos e agentes IDE **concluam**, **falhem** ou **reiniciem** tasks no runtime (`.dare/state.json`), com o **Ralph Loop** (build → test → lint) como gate obrigatório antes de `DONE`.

O problema: o binário nativo já expõe navegação (`--status` / `--next` / `--watch`, 028) e a máquina de estados library-first (`transition`, attempts, cascade, 026), mas ainda **não** materializa as flags de mutação nem o crate de verificação que o Ciclo 7 e as skills `/dare-dag-run*` / `/dare-execute` esperam. Sem `--complete` com Ralph, qualquer `DONE` seria inseguro (stubs / builds quebrados).

Quem consome: agentes IDE após implementar uma task; engenheiros a marcar falha ou reset; CI/smokes de gates. Entrega: crate **`dare-verify`** (`ralph.rs` + adapters de stack) + extensão de `crates/dare-cli/src/commands/execute.rs`, artefatos em `.dare/verification/**`, docs + DEC (sugerido **DEC-030**).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | `--complete` com Ralph | Happy path: gates OK → task `DONE` + attempt `passed=true` | Smoke + unit |
| O-02 | Gate falho bloqueia DONE | Qualquer gate ≠0 → **não** `DONE`; status permanece elegível a retry / `FAILED` conforme Blueprint | Unit + smoke |
| O-03 | Timeout 600 s | Processo de gate que excede 600 s → exit **124** (código observável) | Integração / mock runner |
| O-04 | `--fail` | `dare execute --fail <id> [--reason …]` → `FAILED` + cascade skip + attempt/erro | Smoke |
| O-05 | `--reset` preserva histórico | Após reset: status `PENDING`, `output`/`error` limpos, **`attempts` intactos** | Unit |
| O-06 | Attempts + outputs | Cada complete/fail registra attempt (`n`, `at`, `passed`) e persiste output/error com caps | Unit |
| O-07 | Ingestão básica pós-DONE | Após DONE: artefato em `.dare/verification/<id>.json` (e/ou hook mínimo de graph — Blueprint congela) | Integração FS |
| O-08 | Adapters por stack | ≥1 adapter real (`rust-axum` / workspace) + tabela extensível; stack desconhecida → erro tipado | Unit |
| O-09 | JSON + human | `--json` envelope 004 com `data` tipado (gates, taskId, status) | Smoke |
| O-10 | Ralph + docs | fmt/clippy/test (+ audit se deps) + `cli-execute-mutations.md` (nome Blueprint) + DEC-030 | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 7 completo: mutações + Ralph inicial |
| Tech Lead | Time DARE CLI Rust | Escopo 029 só; não puxar agent/verify avançado |
| Engenheiro CLI | Time implementação | `dare-verify` + flags execute |
| Usuário Final | Devs | `dare execute --complete` após implementar task |
| Agentes IDE | 4 harnesses | Skills execute / dag-run após código |
| Compat | Baseline TS 3.18.1 | Diffs classificados (DEC-030) |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-verify` | MUST | Workspace member `crates/dare-verify`; módulo `ralph` público; sem ciclos crate (CLI → verify → core/contracts/dag) |
| RF-02 | API Ralph | MUST | Função(ões) tipadas: resolver stack → correr gates build→test→lint → `RalphReport` (ok, steps[], duration, timeout?) |
| RF-03 | Adapters de stack | MUST | Tabela hardcoded inicial (alinhar Mestre): pelo menos **`rust-axum`** / detecção workspace Rust; stubs ou comandos para nestjs/fastapi/laravel/go/react/vue/leptos/mcp-* conforme Blueprint (mínimo: Rust real + resto documentado ou `not implemented`) |
| RF-04 | Timeout 600 s | MUST | Cada gate (ou loop agregado — Blueprint congela) usa `SafeCommand::timeout(Duration::from_secs(600))`; timeout → código **124** |
| RF-05 | CLI `--complete <id>` | MUST | Flag mutuamente exclusiva com status/next/watch/fail/reset/agent; arg id obrigatório |
| RF-06 | Fluxo `--complete` | MUST | `load_dag` + `ensure_state` → validar id no DAG → (🟡 auto-`Start` se `PENDING` — Blueprint confirma) → Ralph → se ok: `transition(Complete { output })` + canvas; se fail: **não** Complete |
| RF-07 | `--output` em complete | MUST | Flag `--output <text>` (ou stdin doc) persiste em `task.output` no Complete; default string estável se omitido |
| RF-08 | Bloquear DONE | MUST | Se qualquer gate exit ≠0 ou timeout → exit ≠0; state **sem** `DONE` para essa task |
| RF-09 | CLI `--fail <id>` | MUST | `transition(Fail { error })` + cascade; requer estado válido (🟡 `RUNNING` ou auto-Start — Blueprint); `--reason` / `--error` para mensagem |
| RF-10 | CLI `--reset <id>` | MUST | `transition(Reset)`: volta a `PENDING`; limpa `output`/`error`; **preserva `attempts`** |
| RF-11 | Cascading skip | MUST | Após Fail (e Complete se aplicável): `apply_cascading_skip` via path `transition` 026 |
| RF-12 | Canvas refresh | MUST | Após mutação bem-sucedida: `RefreshCanvas::Yes` (ou write explícito) |
| RF-13 | Attempts | MUST | Complete → `passed=true`; Fail → `passed=false`; `n` monotónico; `at` RFC3339 via `Clock` |
| RF-14 | Caps de output | MUST | Respeitar `limits.task_output_chars` (default 4000) ao gravar output/error/stderr de gates (truncate + nota) |
| RF-15 | Artefato verification | MUST | Escrever `.dare/verification/<taskId>.json` (jail 005) com resumo dos gates (aspectos `build|test|lint`, exit, truncated stdout/stderr hashes ou tails) |
| RF-16 | Ingestão básica pós-DONE | MUST | Após Complete OK: passo mínimo — verification file **e** 🟡 opcional `graph ingest` no-op/stub se crate graph ausente; Blueprint congela “file-only” vs chamada real |
| RF-17 | Detecção de stack | MUST | Ler `dare.config.json` / convenção projeto (alinhar discover 018); fallback `rust-axum` neste repo; desconhecida → InvalidInput/Config 4 |
| RF-18 | `--dag` | MUST | Mesmo default/jail que 028 |
| RF-19 | Flags exclusivas | MUST | Combinar complete+fail / complete+status / etc. → Usage **2** |
| RF-20 | Task inexistente | MUST | id fora do DAG → NotFound **3** |
| RF-21 | Transição inválida | MUST | Complete/Fail/Reset de status ilegal → InvalidInput **4** (mensagem en-US) |
| RF-22 | Lock | MUST | Contenção em `transition` → Io **5** |
| RF-23 | Sem agent | MUST | Flag `--agent` **ausente** neste ciclo (029 não adiciona stub Usage ambíguo — ou Usage 2 se presente por engano no clap; preferir ausente) |
| RF-24 | Sem verify avançado | MUST | Sem mutation/formal/best-of-N/fail-to-pass neste ciclo |
| RF-25 | Docs + DEC | MUST | `docs/compatibility/cli-execute-mutations.md` (ou extensão de `cli-execute-status.md` — Blueprint) + **DEC-030** |
| RF-26 | Capability | SHOULD | Matrix `dare-execute` permanece `cli_commands: ["execute"]` (já 028); atualizar instructions se necessário |
| RF-27 | Mensagens en-US | MUST | Erros e headlines de domínio em inglês |
| RF-28 | Smoke CLI | MUST | complete ok; complete gate-fail; fail; reset; timeout→124 (mock); missing id→3 |
| RF-29 | `--json` | MUST | `data`: `action`, `taskId`, `status`, `ralph`/`gates`, `verificationPath` |
| RF-30 | argv-only | MUST | Gates via `SafeCommand` (006); **sem** shell concatenado |
| RF-31 | Skip audit gate | COULD | Flag `--skip-ralph` **proibida** em produção; se existir só sob `cfg(test)` / env CI documentado — default **não** expor |
| RF-32 | Parallel complete | SHOULD | Dois `--complete` concorrentes: um vence lock; outro Io 5 — sem state parcial |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (esboço — Blueprint congela)

```text
dare execute --complete <id> [--output <text>] [--dag <path>]
dare execute --fail <id> [--reason <text>] [--dag <path>]
dare execute --reset <id> [--dag <path>]
# + globais --json / --no-color (004)
# reusa --status / --next / --watch (028)
# --agent ABSENTE (030+)
```

### API de domínio (esboço)

```text
// crates/dare-verify/src/ralph.rs
pub struct RalphReport { pub ok: bool, pub steps: Vec<GateStep>, … }
pub struct GateStep { pub aspect: GateAspect, /* build|test|lint */, pub exit_code: i32, … }
pub fn run_ralph(root, stack, runner: &dyn ProcessRunner) -> CoreResult<RalphReport>

// crates/dare-cli — orchestration
// complete: Start? → run_ralph → transition(Complete) | abort
```

### Contratos de disco

| Path | Papel | Mutação |
|------|-------|---------|
| `DARE/dare-dag.yaml` (ou `--dag`) | Input | **Read-only** |
| `.dare/state.json` | Runtime | **Write** via `transition` / ensure |
| `DARE/.canvas.md` | Observabilidade | **Write** após mutação |
| `.dare/verification/<id>.json` | Baseline Ralph inicial | **Write** pós-gates / pós-DONE |
| `.dare/state.json.darelock` | Lock | Em paths que persistem state |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesma stack + mock runner → mesmo `RalphReport` shape | Unit |
| RNF-02 | Performance | Overhead de orquestração (excl. gates reais) < 200 ms | Informal |
| RNF-03 | Disponibilidade | Funciona sem `.dare/verification` prévio (cria dirs) | Integração |
| RNF-04 | Observabilidade | Steps Ralph em human + JSON; correlation_id | 004 |
| RNF-05 | Manutenibilidade | Verify em crate próprio; CLI thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux argv + timeout | CI 003 |
| RNF-07 | Cap I/O | Truncate stdout/stderr de gates; caps 007 | Unit |
| RNF-08 | Integridade | Crash mid-Ralph: state não fica `DONE`; verification parcial OK se marcado `ok:false` | Teste falha parcial |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dag`, task id, paths de verification sob `ProjectRoot` / `SafeRelativePath` | OWASP A03 / 005 |
| RS-02 | Redact secrets em logs/erros/artefatos verification (tails de stderr) | OWASP A02 / 004 |
| RS-03 | CLI local: não expor state fora do project root; task id só do DAG do projeto | OWASP A01 |
| RS-04 | `cargo audit` / `deny` sem CVE HIGH/CRITICAL se deps novas | OWASP A06 |
| RS-05 | Secrets só via env; denylist 006 no spawn dos gates | Supply chain / 006 |
| RS-06 | **Sem shell**; argv-only `SafeCommand` | 006 / microplano |
| RS-07 | Timeout 600 s obrigatório (DoS / hang de build) | Availability |
| RS-08 | Não marcar `DONE` se Ralph falhou (integridade do método) | Integrity |
| RS-09 | `--output` / `--reason` truncados + redact | OWASP A03 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Verify | **`dare-verify`** (NOVO) | `0.1.0-alpha.0` |
| Domínio state | `dare-dag` (`transition`, `Transition::{Complete,Fail,Reset,Start}`) | **026** |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| Processos | `dare-core` `SafeCommand` / `ProcessRunner` / mock | **006** |
| Contratos | `dare-contracts` (`RuntimeStateV1`, `AttemptRecord`, limits) | 007 |
| Path / lock / atomic | `dare-core` | 005 |
| Config / stack | `dare-config` / `dare-project` | 008 / 018 |
| Saída | OutputRenderer 004 | DEC-005 |
| Testes | tempfile + mock runner + fixtures DAG | workspace |
| Container | `Dockerfile.rust` + `docker-compose.ci.yml` | 003 |

**Deps novas:** crate `dare-verify` no workspace; preferir reutilizar deps existentes (sem crates HTTP/LLM).

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem | Local | read/write | In/Out | state; verification; canvas | CLI / verify |
| Toolchain do projeto | Processos | argv | Out→In | build/test/lint exit+stdio | Ralph adapters |
| Agentes IDE | Consumidor | stdout / JSON | Out | resultado complete/fail | Skills execute |
| Baseline TS 3.18.1 | Referência | — | In | flags / exit 124 / Ralph | Compat |
| GraphRAG ingest | Opcional mínimo | — | Out | 🟡 file-only ou stub | Blueprint |
| Review / mutation | — | — | — | **Fora** (032/049) | — |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **006**, **026** e **028** concluídos.
- Mensagens en-US.
- Ações execute **mutuamente exclusivas** (clap).
- Não alterar schema `RuntimeStateV1` / `dare-dag.yaml` sem ADR (attempts já existem).
- Diffs vs TS → DEC-030 + classification matrix.
- Timeout Ralph = **600 s**; código observável **124** (pode exigir bypass do mapa 1–5 de `ErrorKind` — Blueprint congela mecanismo).
- Reset **MUST** preservar `attempts` (aceite do microplano).
- Não implementar `--agent`, worktrees, budget, decay, review semântico completo, mutation, formal.

---

## 10. FORA DO ESCOPO (v1)

- `dare execute --agent`, worktrees, budget, failure-signature decay (→ **030–031**).
- Gate `dare review` anti-stub / verdito semântico completo (→ **032**).
- Verify avançado: fail-to-pass, anti-tamper, mutation, formal, best-of-N (→ **049**).
- GraphRAG ingest completo / Neo4j (→ **040–043**).
- `dare bench` / dashboard gates (→ **049** / **051**).
- Alterar política 028 de “no Start-on-next” (permanece; Start só no path complete/fail se Blueprint exigir).
- Sub-DAG / refine / splice (→ **033**).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Exit **124** fora do mapa `ErrorKind` 1–5 | Alta | Alto | Blueprint: `ExitCode` direto na CLI para timeout Ralph; documentar em DEC-030 |
| R-02 | Complete exige `RUNNING` mas 028 não faz Start | Alta | Alto | Auto-`Start` no início de `--complete`/`--fail` se `PENDING` (Classe B vs TS se diferir) |
| R-03 | Adapters de todas as stacks no mesmo PR | Alta | Médio | MUST: Rust real; demais stacks tabela + `not implemented` ou comandos mínimos |
| R-04 | Ralph longo (cargo test) nos smokes | Alta | Médio | Mock `ProcessRunner` nos testes; um smoke opcional `#[ignore]` real |
| R-05 | Diff TS: complete sem Ralph / flags extras | Média | Médio | DEC-030 Classe B/C; matriz de classificação |
| R-06 | Verification path traversal via task id | Baixa | Alto | Sanitizar id (já IDs DAG) + `SafeRelativePath` |
| R-07 | Reset apagar attempts por engano | Média | Alto | Teste unitário explícito + aceite microplano |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF `--complete` / `--fail` / `--reset` + exclusividade aceites
- [ ] Ralph build→test→lint + timeout 600 s / exit 124 aceites
- [ ] Política “gate falho **bloqueia** DONE” aceite
- [ ] Reset preserva `attempts` aceite
- [ ] Escopo adapters (Rust MUST; outras stacks) aceite
- [ ] Ingestão pós-DONE mínima (verification file ± stub graph) aceite
- [ ] Fora de escopo 030+/032/049 explícito
- [ ] Riscos R-01…R-07 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-029-execute-complete-fail-reset-e-ralph-inicial.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-verify/` | Crate NOVO — Ralph + adapters |
| `crates/dare-verify/src/ralph.rs` | Loop build→test→lint |
| `crates/dare-cli/src/commands/execute.rs` | Estender complete/fail/reset |
| `crates/dare-cli/src/main.rs` | Clap flags + wiring |
| `Cargo.toml` (workspace) | Member `dare-verify` |
| `.dare/verification/<id>.json` | Artefato de gates |
| `docs/compatibility/cli-execute-mutations.md` | Docs (**criar**; nome Blueprint) |
| `docs/DECISION-LOG.md` | **DEC-030** |
| `tests/fixtures/` | Fixtures Ralph / DAG mutation |

## Apêndice B — Estado atual (gap)

| Capacidade | Hoje | 029 |
|------------|------|-----|
| `transition` Complete/Fail/Reset/Start | ✅ 026 | Reusar via CLI |
| `dare execute` status/next/watch | ✅ 028 | Estender flags |
| `SafeCommand` + timeout 124 | ✅ 006 | Reusar nos gates |
| Crate `dare-verify` / Ralph | ❌ | **Criar** |
| `.dare/verification/**` | ❌ | **Criar** |
| CLI `--complete/--fail/--reset` | ❌ (TS only / orquestração externa) | **Implementar** |
| Docs DEC mutações | ❌ | Criar DEC-030 |

## Apêndice C — Semântica `--complete` (normativa pretendida)

```text
1. resolve root + dag + task id
2. load_dag + ensure_state
3. if task missing → NotFound 3
4. if status PENDING → transition(Start)   // 🟡 Blueprint confirma
5. if status not RUNNING → InvalidInput 4
6. run_ralph(stack) with timeout 600s per gate (or budget total — Blueprint)
7. write .dare/verification/<id>.json (ok=false on failure)
8. if !ralph.ok → exit (gate exit or 124); DO NOT Complete
9. transition(Complete { output }) + canvas refresh
10. ingestão básica pós-DONE (verification final ok=true; ± graph stub)
11. print human/JSON success
```

Aceite microplano: **“DONE exige gates aprovados.”**

## Apêndice D — Exit codes

| Code | Quando |
|------|--------|
| 0 | complete/fail/reset OK |
| 1 | Internal / Ralph gate falhou (exit do processo ≠0, ≠124) — 🟡 ou propagar exit do gate; Blueprint congela |
| 2 | Usage (flags exclusivas / id ausente) |
| 3 | DAG ou task NotFound |
| 4 | InvalidInput / Config / transição ilegal / stack desconhecida |
| 5 | Io (lock / write) |
| **124** | Timeout de gate Ralph (alinhar 006 / Mestre) |

## Apêndice E — Aceite do microplano (mapeamento)

| Critério microplano | RF / O |
|---------------------|--------|
| DONE exige gates aprovados | O-01, O-02, RF-06, RF-08, Apêndice C |
| Timeout retorna 124 | O-03, RF-04, RS-07, Apêndice D |
| Reset preserva histórico | O-05, RF-10, R-07 |
| fmt/clippy/test | O-10 |
| Compat classificada | RF-25, R-05 |
| Attempts e outputs | O-06, RF-13, RF-14 |
| Ingestão básica pós-DONE | O-07, RF-15, RF-16 |
| Adapters + build/test/lint | O-08, RF-02, RF-03 |

## Apêndice F — Relação com DEC-029

- **DEC-029** (028): status/next/watch; **no** Start-on-next.  
- **DEC-030** (este ciclo): complete/fail/reset + Ralph; Start **apenas** no path de mutação (se aprovado). Não revoga DEC-029.

## Apêndice G — Próximos passos DARE

1. Revisar e **aprovar** este Design (humano).  
2. `/dare-blueprint` → `DARE/BLUEPRINT-029-execute-complete-fail-reset-e-ralph-inicial.md`.  
3. `/dare-tasks` → `TASKS-029` + `dare-dag-029.yaml` + `EXECUTION-029/`.  
4. Executar; ao closeout → [`030-execute-agent-mock-worktrees-e-budget.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/030-execute-agent-mock-worktrees-e-budget.md).
