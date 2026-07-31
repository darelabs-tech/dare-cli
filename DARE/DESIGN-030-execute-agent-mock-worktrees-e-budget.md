# DESIGN: Execute agent — mock, worktrees e budget (Microplano 030)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (Blueprint gerado; aguarda aprovação humana do Blueprint)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/030-execute-agent-mock-worktrees-e-budget.md`  
> **Referência:** Microplano **029** (`--complete` / Ralph / DEC-030) · **006** (`SafeCommand` / `CancelFlag` / timeout 124) · **005** (path safety) · **026** (state / attempts) · Documento Mestre §2.1 / §15 / §26 (Ciclo 8) · baseline TS 3.18.1 · skill `/dare-execute`  
> **Posição:** 30 de 56  
> **Arquivo:** `DARE/DESIGN-030-execute-agent-mock-worktrees-e-budget.md`  
> **Escopo deste ciclo apenas:** infraestrutura de **`dare execute --agent`** com driver **`mock`/`noop`**, worktrees, `BudgetTracker`, cancelamento, `failureSignature` e política **`fixed`**. **Não** drivers reais Claude/Codex/Cursor/Antigravity (→ **031**). **Não** política `decay` / REPLAN / spliceSubDag (→ **031+** / **033**). **Não** guard completo (→ **034**; preflight stub/no-op neste ciclo). **Não** best-of-N / mutation / formal (→ **049**).

---

## 1. DESCRIÇÃO

Este Design abre o Ciclo 8 do orquestrador: validar a **máquina autônoma** de `dare execute --agent` **sem** variabilidade de LLMs reais — usando um `AgentDriver` mock/noop, worktrees isolados, orçamento de tokens e cancelamento determinístico.

O problema: após 029 (mutações + Ralph), o binário ainda não tem o loop agente→candidatos→budget→política. Sem mock + worktrees + budget, os drivers reais (031) e o decay (posterior) não têm chassi testável. Quem consome: CI/smokes da máquina de estados; engenheiros a dry-run `--agent --driver mock`; futuros drivers. Entrega: crate **`dare-agent`** + `crates/dare-cli/src/commands/execute_agent.rs` (ou módulo em `execute.rs` — Blueprint congela), paths `.dare/agent-worktrees/**`, docs + DEC (sugerido **DEC-031**).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Trait `AgentDriver` | API tipada `doctor` + `run` (+ tipos `AgentRequest` / `AgentRunResult`) | Unit compile + docs |
| O-02 | Mock success | `--driver mock` (modo success) → `AgentRunResult` status success | Unit + smoke |
| O-03 | Mock failure | Mock modo fail → status failure + stderr redigido | Unit |
| O-04 | Mock timeout | Mock modo timeout → exit observável **124** ou status timeout | Unit + smoke |
| O-05 | Worktree create | Cria `.dare/agent-worktrees/<id>/` + branch `dare/agent-<id>` (nome Blueprint) | Integração git |
| O-06 | Worktree cleanup | Após run (ok/fail/cancel): worktree removida **ou** marcada recuperável | Integração FS |
| O-07 | Budget interrupt | `--budget-tokens N` esgotado → para loop; exit tipado (Blueprint) | Unit |
| O-08 | Cancel | `CancelFlag` / Ctrl+C / token → run aborta limpo; worktree cleanup | Unit + smoke |
| O-09 | `failureSignature` | Tentativa falha grava `failureSignature` (sha256[0..8] de aspecto+stderr normalizado) em attempt | Unit |
| O-10 | Política `fixed` | Só decisões `DONE` / `CONTINUE` / `STOP` (sem REPLAN/FRESH_START/ESCALATE) | Unit |
| O-11 | Docs + DEC | `cli-execute-agent.md` (nome Blueprint) + DEC-031; fmt/clippy/test/audit | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 8: máquina agent sem LLM real |
| Tech Lead | Time DARE CLI Rust | Escopo mock-only; não puxar 031/034 |
| Engenheiro CLI | Time implementação | `dare-agent` + CLI `--agent` |
| Usuário Final | Devs / CI | Dry-run determinístico `--driver mock` |
| Compat | Baseline TS 3.18.1 | Diffs classificados (DEC-031) |
| Guard (futuro) | Microplano 034 | Preflight real deferido |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-agent` | MUST | Workspace member; deps `dare-core` (+ contracts se preciso); **sem** ciclo com `dare-cli` |
| RF-02 | Trait `AgentDriver` | MUST | `doctor(&self) -> CoreResult<DriverHealth>`; `run(&self, req, cancel) -> CoreResult<AgentRunResult>` — 🟡 sync (alinhar `dare-ai`) vs async Mestre; Blueprint congela |
| RF-03 | Tipos request/result | MUST | `AgentRequest`: prompt, cwd/worktree path, limits, env allowlist stub, model opcional; `AgentRunResult`: status enum, summary, stdout/stderr redacted, tokens/cost optional, duration |
| RF-04 | Driver `mock` | MUST | Modos: success / fail / timeout (via env `DARE_AGENT_MOCK` ou flag `--mock-mode` — Blueprint); **noop** alias ou variante que retorna success imediato sem I/O |
| RF-05 | Driver registry | MUST | Só `mock` (e `noop` se separado) neste ciclo; `claude|codex|cursor|antigravity` → InvalidInput `"driver not implemented"` exit **4** |
| RF-06 | CLI `--agent` | MUST | Flag em `dare execute`, exclusiva vs status/next/watch/complete/fail/reset; requer `--driver` |
| RF-07 | `--driver <id>` | MUST | Default **`mock`** quando `--agent` (Blueprint confirma); valor case-sensitive lowercase |
| RF-08 | `--budget-tokens <u64>` | MUST | `BudgetTracker` acumula tokens reportados; ≥ budget → interrompe; default Blueprint (ex. ilimitado=0 ou valor alto) |
| RF-09 | Worktree manager | MUST | Create under `.dare/agent-worktrees/<safe-id>/` via `git worktree add` argv-only; branch nome canónico |
| RF-10 | Worktree jail | MUST | Paths sob `ProjectRoot`; id path-safe; deny escape |
| RF-11 | Cleanup | MUST | API `cleanup_worktree` + recovery scan de órfãos (list + remove stale) |
| RF-12 | Cancel token | MUST | Propagar `CancelFlag` (006) ao `run`; mock respeita cancel mid-flight |
| RF-13 | `failureSignature` | MUST | Em attempt failed: `failureSignature = hex(sha256(aspect + "\0" + normalize(stderr))[0..8])` (algoritmo Blueprint); campo já no contrato `AttemptRecord` |
| RF-14 | Política `fixed` | MUST | Enum `AgentDecision::{ Done, Continue, Stop }`; sem REPLAN/FRESH_START/ESCALATE neste ciclo |
| RF-15 | Loop mínimo | MUST | Orquestração: pick next ready (reusar 028) **ou** task id opcional → worktree → mock run → Ralph? 🟡 Blueprint: Ralph após mock success **sim** (reusar 029) **ou** defer; default sugerido: **chamar `--complete` path / Ralph após success mock** |
| RF-16 | Guard preflight | SHOULD | Hook no-op retornando OK até **034**; **não** exit 6 neste ciclo salvo stub documentado |
| RF-17 | Telemetria mínima | SHOULD | Log/tracing steps (driver, worktree path, budget remaining); sem dashboard |
| RF-18 | JSON | MUST | `--json` data: action `agent`, driver, decision, worktreePath, budget, result status |
| RF-19 | `--best-of <n>` | COULD | Se presente: criar até N worktrees; senão omitir flag (preferir ausente até 049) |
| RF-20 | `--policy` | MUST | Aceitar só `fixed` neste ciclo; `decay` → InvalidInput 4 |
| RF-21 | Docs + DEC | MUST | `docs/compatibility/cli-execute-agent.md` + **DEC-031** |
| RF-22 | Capability | SHOULD | Atualizar instructions `dare-execute` com `--agent --driver mock` |
| RF-23 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-24 | Smoke CLI | MUST | mock success; mock fail; budget stop; cancel (se testável); unknown driver→4 |
| RF-25 | Sem shell | MUST | `git worktree` via `SafeCommand` argv |
| RF-26 | Recovery | MUST | Comando interno ou `dare execute --agent --cleanup-worktrees` 🟡 — Blueprint congela flag vs API só lib |
| RF-27 | Git prerequisite | MUST | Repo sem `.git` → InvalidInput/Config 4 mensagem estável |
| RF-28 | Parallel lock | SHOULD | Agent run adquire state lock; contenção → Io 5 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (esboço — Blueprint congela)

```text
dare execute --agent --driver mock [--budget-tokens N] [--policy fixed] [--dag PATH]
# + globais --json / --no-color
# --driver claude|… → 4 not implemented (031)
# --policy decay → 4
```

### API de domínio (esboço)

```text
// crates/dare-agent
pub trait AgentDriver {
  fn doctor(&self) -> CoreResult<DriverHealth>;
  fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult>;
}
pub struct MockDriver { pub mode: MockMode } // Success|Fail|Timeout
pub struct BudgetTracker { /* remaining tokens */ }
pub struct WorktreeManager { /* create / cleanup / list_orphans */ }
pub fn failure_signature(aspect: &str, stderr: &str) -> String;
pub enum FixedDecision { Done, Continue, Stop }
```

### Contratos de disco

| Path | Papel | Mutação |
|------|-------|---------|
| `.dare/agent-worktrees/**` | Worktrees candidatos | Create / remove |
| `.dare/state.json` | Attempts + failureSignature + status | Via transition 026/029 |
| `DARE/dare-dag.yaml` | Input | Read-only |
| `.git` / worktree refs | Git | Via SafeCommand |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo mock mode + budget → mesma decision sequence | Unit ×2 |
| RNF-02 | Performance | Overhead orquestração (excl. git) < 300 ms típico | Informal |
| RNF-03 | Disponibilidade | Funciona em repo git limpo; Windows/macOS/Linux worktree | CI 003 |
| RNF-04 | Observabilidade | tracing + JSON budget/decision | 004 |
| RNF-05 | Manutenibilidade | Crate `dare-agent` separado de `dare-ai` / `dare-harness` | Clippy |
| RNF-06 | Integridade | Cancel/crash: worktree órfão recuperável; state sem DONE espúrio | Teste falha parcial |
| RNF-07 | Cap I/O | Truncate stdout/stderr resultados; caps 007 | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dag`, worktree ids, budget bounds sob jail | OWASP A03 / 005 |
| RS-02 | Redact stdout/stderr/summary em `AgentRunResult` e logs | OWASP A02 / 004 |
| RS-03 | Worktrees só sob project root; sem path escape | OWASP A01 / 005 |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL se deps novas | OWASP A06 |
| RS-05 | Secrets só env; denylist 006 em spawns git/mock | Supply chain |
| RS-06 | argv-only (`git worktree`); sem shell | 006 |
| RS-07 | Budget finito evita loop infinito de CONTINUE | Availability |
| RS-08 | Cancel limpa ou marca worktree; não deixa lock state preso | Integridade |
| RS-09 | Guard real deferido — não fingir segurança de preflight | Honestidade / 034 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Agent | **`dare-agent`** (NOVO) | `0.1.0-alpha.0` |
| CLI | `dare-cli` + clap **4.5.40** | `--agent` |
| Process / cancel | `dare-core` SafeCommand, CancelFlag | **006** |
| State / Ralph | `dare-dag` + `dare-verify` | **026** / **029** |
| Hash signature | `sha2` (já workspace) | failureSignature |
| Git | CLI `git` via SafeCommand | worktrees |
| Saída | OutputRenderer 004 | DEC-005 |
| Testes | tempfile + mock + git init fixtures | workspace |
| Container | compose CI 003 | Fase 1 |

**Deps novas:** crate `dare-agent`; preferir pins existentes. Sem `@anthropic-ai/sdk` / HTTP LLM.

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Git CLI | SCM | argv | Out→In | worktree add/remove/list | WorktreeManager |
| Filesystem | Local | — | In/Out | `.dare/agent-worktrees`, state | Agent / CLI |
| Ralph / verify | Local | crate | In | pós-success mock (🟡) | 029 |
| Guard | — | — | — | Stub até **034** | — |
| Drivers reais | — | — | — | **Fora** (**031**) | — |
| Baseline TS 3.18.1 | Ref | — | In | flags / budget / worktree paths | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** **006**, **029** concluídos; **034** planejado para preflight real (não bloqueia 030).
- Mensagens en-US.
- `--agent` mutuamente exclusivo com outras ações execute.
- Política neste ciclo: **somente `fixed`**.
- Driver neste ciclo: **somente `mock`/`noop`**.
- Diffs vs TS / Mestre async → DEC-031 + classification.
- Não implementar REPLAN, spliceSubDag, decay, best-of-N real, drivers 031.

---

## 10. FORA DO ESCOPO (v1)

- Drivers `claude` / `codex` / `cursor` / `antigravity` / API Anthropic (→ **031**).
- Política `decay` e decisões FRESH_START / REPLAN / ESCALATE (→ **031+** / **033**).
- `dare guard` preflight real + exit **6** (→ **034**).
- `dare review` anti-stub no loop agent (→ **032**).
- Best-of-N Pareto / mutation / formal (→ **049**).
- `--require-approval` TTY rank gate (SHOULD defer se não essencial).
- Enrichment `dare-ai` misturado com agent (proibido pelo Mestre §15.1).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Mestre `async_trait` vs codebase sync | Alta | Médio | Congelar trait **sync** + CancelFlag (Classe B); DEC-031 |
| R-02 | Git worktree frágil no Windows | Alta | Alto | argv testados CI; smoke com `git init` fixture; recovery órfãos |
| R-03 | Pré-req 034 confundir exit 6 | Média | Médio | Preflight no-op documentado; sem exit 6 em 030 |
| R-04 | Ralph após mock duplica tempo | Média | Médio | Blueprint: Ralph on success **ou** flag `--no-ralph` test-only; smokes usam `DARE_RALPH_MOCK` |
| R-05 | Budget 0 vs “ilimitado” ambíguo | Média | Médio | Congelar: `0` = ilimitado **ou** Usage 2 se 0; DEC |
| R-06 | Loop CONTINUE infinito | Baixa | Alto | Max iterations + budget (RS-07) |
| R-07 | Diff path worktrees TS (`.dare/worktrees` vs `agent-worktrees`) | Média | Médio | Path microplano **`.dare/agent-worktrees`**; Classe B/C em DEC |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF AgentDriver + mock success/fail/timeout aceites
- [ ] Worktree create/cleanup/recovery aceites
- [ ] Budget + cancel + failureSignature aceites
- [ ] Política só `fixed` aceite
- [ ] Sync vs async trait + Ralph-após-mock decididos (ou ok para Blueprint)
- [ ] Guard stub até 034 explícito
- [ ] Fora de escopo 031/033/034/049 explícito
- [ ] Riscos R-01…R-07 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-030-execute-agent-mock-worktrees-e-budget.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-agent/` | Crate NOVO |
| `crates/dare-agent/src/driver.rs` | Trait + MockDriver |
| `crates/dare-agent/src/budget.rs` | BudgetTracker |
| `crates/dare-agent/src/worktree.rs` | WorktreeManager |
| `crates/dare-agent/src/policy.rs` | FixedDecision |
| `crates/dare-agent/src/signature.rs` | failure_signature |
| `crates/dare-cli/src/commands/execute_agent.rs` | CLI orchestration (**ou** extend execute.rs) |
| `.dare/agent-worktrees/` | Disco |
| `docs/compatibility/cli-execute-agent.md` | Docs |
| `docs/DECISION-LOG.md` | **DEC-031** |

## Apêndice B — Estado atual (gap)

| Capacidade | Hoje | 030 |
|------------|------|-----|
| execute status/next/watch | ✅ 028 | Reusar pick-ready |
| complete/fail/reset + Ralph | ✅ 029 | Reusar pós-success |
| CancelFlag / SafeCommand | ✅ 006 | Reusar |
| AttemptRecord.failureSignature | ✅ contrato | Preencher |
| `dare-agent` / AgentDriver | ❌ | **Criar** |
| Worktrees agent | ❌ | **Criar** |
| BudgetTracker | ❌ | **Criar** |
| `--agent` CLI | ❌ | **Criar** |

## Apêndice C — Semântica `--agent` mock (normativa pretendida)

```text
1. root + dag + exclusive --agent
2. resolve driver (mock only) else 4
3. policy must be fixed (default fixed)
4. guard_preflight_stub() → Ok  // até 034
5. ensure_state; select target task(s)  // Blueprint: --next min-rank single ou --task id
6. while budget ok && !cancel && decision==Continue:
     a. create worktree
     b. driver.run(request, cancel)
     c. record attempt + failureSignature if fail
     d. apply fixed policy → Done|Continue|Stop
     e. cleanup worktree (best-effort)
7. if Done → optional Ralph/complete path (🟡 Blueprint)
8. if budget exhausted → exit tipado (1 ou 4 — Blueprint)
9. print human/JSON
```

## Apêndice D — Exit codes

| Code | Quando |
|------|--------|
| 0 | Agent run completed (Done) ou noop success |
| 1 | Internal / mock fail terminal / budget exhausted (🟡) |
| 2 | Usage (flags exclusivas, budget inválido) |
| 3 | DAG/task NotFound |
| 4 | InvalidInput / unknown driver / policy decay / no git |
| 5 | Io (lock / worktree / state) |
| **124** | Timeout (mock timeout ou processo) |
| **6** | Reservado guard (**034**) — **não** emitir em 030 |

## Apêndice E — Aceite do microplano (mapeamento)

| Critério microplano | RF / O |
|---------------------|--------|
| Suite mock sucesso/falha/timeout | O-02…O-04, RF-04 |
| Orçamento interrompe | O-07, RF-08, RS-07 |
| Worktrees limpas/recuperáveis | O-05, O-06, RF-09…11, RF-26 |
| fmt/clippy/test | O-11 |
| Compat classificada | RF-21, R-01, R-07 |
| failureSignature + fixed | O-09, O-10, RF-13, RF-14 |

## Apêndice F — Relação com DEC anteriores

- **DEC-029:** no Start-on-next (observação).  
- **DEC-030:** complete/Ralph.  
- **DEC-031 (este):** `--agent` mock + worktrees + budget; Start/Complete só via política/Ralph path aprovado; **não** revoga DEC-029/030.

## Apêndice G — Próximos passos DARE

1. Revisar e **aprovar** este Design (humano).  
2. `/dare-blueprint` → `DARE/BLUEPRINT-030-execute-agent-mock-worktrees-e-budget.md`.  
3. `/dare-tasks` → `TASKS-030` + `dare-dag-030.yaml` + `EXECUTION-030/`.  
4. Executar; ao closeout → [`031-drivers-reais-de-agentes.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/031-drivers-reais-de-agentes.md).
