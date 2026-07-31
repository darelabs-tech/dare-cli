# DESIGN: Drivers reais de agentes (Microplano 031)

> **Versão:** v1.0 | **Data:** 2026-07-23 | **Status:** APPROVED (execução concluída 8/8)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/031-drivers-reais-de-agentes.md`  
> **Referência:** Microplano **030** (`AgentDriver` / mock / worktrees / DEC-031) · **006** (`SafeCommand` / `CancelFlag` / timeout **124**) · **024** (`dare-ai` env overrides `DARE_*_COMMAND`) · **034** (guard preflight exit **6**) · Documento Mestre §15.2–15.3 / §27 (Ciclo 9) · baseline TS 3.18.1 · docs `cli-execute-agent.md`  
> **Posição:** 31 de 56  
> **Arquivo:** `DARE/DESIGN-031-drivers-reais-de-agentes.md`  
> **Escopo deste ciclo apenas:** drivers reais **`codex`**, **`claude`**, **`cursor`**, **`antigravity`** em `crates/dare-agent/src/drivers/**`, `doctor`, command overrides, normalização token/cost, redaction, suite comum detection/success/failure/timeout/cancel/malformed/missing. **Não** política `decay` / REPLAN / spliceSubDag (→ **033**). **Não** Claude API direta / Anthropic SDK (→ opcional futuro; Mestre §15.3 passo 6). **Não** best-of-N / approval TTY rank (→ **049** / posterior). **Não** `dare ai` (→ **050**).

---

## 1. DESCRIÇÃO

Este Design completa o Ciclo 9 do orquestrador: ligar `dare execute --agent` aos **agentes reais** (Codex CLI JSONL, Claude Code CLI, Cursor Agent CLI, Antigravity CLI) sobre o chassi já entregue no **030** (`AgentDriver` sync, worktrees, budget, política `fixed`, Ralph-on-Done) e o preflight **034** (exit **6**).

O problema: hoje `resolve_driver` só aceita `mock`/`noop`; qualquer outro id retorna exit **4** `"driver not implemented"`. Sem drivers reais, o loop autônomo não executa tasks em IDEs/CLIs de produção. Quem consome: engenheiros e CI que dry-run com mock e sobem para drivers reais; orquestração DARE em projetos piloto. Entrega: módulos sob `crates/dare-agent/src/drivers/**`, registro em `resolve_driver`, docs + **DEC-037** (próximo id livre após DEC-036 GraphRAG).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Driver Codex | `doctor` + `run` via JSONL; suite comum | Unit + smoke (mock runner) |
| O-02 | Driver Claude Code CLI | `doctor` + `run`; suite comum | Unit + smoke |
| O-03 | Driver Cursor Agent CLI | `doctor` + `run`; suite comum | Unit + smoke |
| O-04 | Driver Antigravity CLI | `doctor` + `run`; suite comum | Unit + smoke |
| O-05 | Suite comum | detection / doctor / success / failure / timeout / cancel / malformed / missing exe / secret redaction | 9 casos × 4 drivers (fixtures) |
| O-06 | Command overrides | Env `DARE_*_COMMAND` altera argv (mesmo padrão 024) | Unit |
| O-07 | Token/cost | Quando o CLI reporta, `AgentRunResult.tokens` (e cost se Blueprint) preenchidos; senão `None` | Unit |
| O-08 | Redaction | Stdout/stderr/summary/logs sem secrets (`dare_core::redact`) | Unit + smoke |
| O-09 | Missing executable | Diagnóstico estável (doctor `ok=false` ou run→InvalidInput/Internal tipado) | Unit |
| O-10 | Docs + DEC | Atualizar `cli-execute-agent.md` + **DEC-037**; fmt/clippy/test/audit | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 9: agent loop com CLIs reais |
| Tech Lead | Time DARE CLI Rust | Suite comum; sem SDK Anthropic neste ciclo |
| Engenheiro CLI | Time implementação | `dare-agent/drivers` + wire `resolve_driver` |
| Usuário Final | Devs / CI | `--driver codex\|claude\|cursor\|antigravity` |
| Compat | Baseline TS 3.18.1 | Diffs Classe A/B/C em DEC-037 |
| Security | Guard 034 | Preflight já ativo; drivers não bypassam exit 6 |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `drivers/` | MUST | `crates/dare-agent/src/drivers/{mod,codex,claude,cursor,antigravity}.rs` (nomes Blueprint); reexport via `lib.rs` |
| RF-02 | IDs canónicos | MUST | CLI `--driver`: `codex` \| `claude` \| `cursor` \| `antigravity` \| `mock` \| `noop` (lowercase); desconhecidos → **4** |
| RF-03 | `resolve_driver` | MUST | Resolve os 4 reais + mock/noop; remove `"driver not implemented"` para esses ids |
| RF-04 | Trait sync | MUST | Continuar `AgentDriver` **sync** + `&CancelFlag` (Classe B vs Mestre `async_trait`; alinhado 030/024) |
| RF-05 | Codex JSONL | MUST | Spawn argv-only; parse eventos JSONL (stdout); mapear success/failure/timeout; tokens quando presentes |
| RF-06 | Claude Code CLI | MUST | Spawn Claude Code CLI (não Anthropic SDK); argv-only; parse saída normalizada |
| RF-07 | Cursor Agent CLI | MUST | Spawn Cursor agent CLI; argv-only; parse saída |
| RF-08 | Antigravity CLI | MUST | Spawn Antigravity CLI; argv-only; parse saída |
| RF-09 | `doctor` | MUST | Por driver: detecção de executável + versão/health (`DriverHealth { driver, ok, detail }`); sem rede obrigatória |
| RF-10 | Command overrides | MUST | Env (alinhar 024): `DARE_CODEX_COMMAND`, `DARE_CLAUDE_COMMAND`, `DARE_CURSOR_COMMAND`, `DARE_ANTIGRAVITY_COMMAND` — whitespace-split argv, **sem** shell |
| RF-11 | Defaults | MUST | Defaults estáveis documentados (ex. `codex exec …`, Blueprint congela flags `--json`/`--sandbox`/approval) |
| RF-12 | Sandbox / approval | SHOULD | Quando o CLI suportar, passar flags documentadas; se omitidas, defaults seguros (Blueprint); **sem** `--require-approval` TTY neste ciclo |
| RF-13 | Timeout | MUST | Respeitar timeout de processo (006) → `AgentRunStatus::Timeout` → CLI exit **124** (paridade mock 030) |
| RF-14 | Cancel | MUST | Poll/`CancelFlag` durante run; status `Cancelled`; cleanup worktree pelo loop 030 |
| RF-15 | Malformed output | MUST | JSONL/stdout inválido → `Failure` (ou Internal tipado Blueprint) **sem** panic; mensagem estável en-US |
| RF-16 | Missing executable | MUST | Doctor `ok=false` com detail; `run` falha com mensagem `"executable not found: …"` (exit CLI **1** ou **4** — Blueprint congela) |
| RF-17 | Token/cost | MUST | Preencher `tokens: Option<u64>` quando o CLI reportar; cost opcional em campo extra ou omitido até Blueprint |
| RF-18 | Caps | MUST | Truncar stdout/stderr ao `stdout_cap_chars` do `AgentRequest` (030) antes de persistir/log |
| RF-19 | Redact | MUST | Aplicar `dare_core::redact` a stdout, stderr, summary e erros antes de log/JSON/attempt |
| RF-20 | Env allowlist | SHOULD | Não propagar secrets do host além do necessário; denylist 006 em subprocess env |
| RF-21 | cwd | MUST | `run` usa `req.cwd` (worktree path do loop 030); jail sob ProjectRoot já garantido pelo orquestrador |
| RF-22 | Model | SHOULD | Se `--model`/request.model presente e CLI aceitar, passar; senão ignorar silenciosamente |
| RF-23 | Suite comum | MUST | Fixtures/tests: detection, doctor, success, failure, timeout, cancel, malformed, missing exe, secret redaction — por driver (ProcessRunner mockável) |
| RF-24 | Smokes CLI | MUST | Com binário fake/env override: pelo menos 1 smoke success + 1 missing-exe por driver **ou** smoke paramétrico documentado |
| RF-25 | Docs | MUST | Atualizar `docs/compatibility/cli-execute-agent.md` (drivers reais + env); **DEC-037** |
| RF-26 | Matriz status | MUST | `000A-MATRIZ-DE-STATUS.md` 031 → Concluído |
| RF-27 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-28 | Sem shell | MUST | Todo spawn via `SafeCommand` argv-only (006) |
| RF-29 | Guard intacto | MUST | Preflight 034 permanece; FAIL → exit **6** antes de `run` |
| RF-30 | Policy | MUST | Continuar só `fixed` neste ciclo; `decay` → **4** (033) |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (esboço — Blueprint congela flags Codex/Claude)

```text
dare execute --agent --driver codex|claude|cursor|antigravity|mock|noop \
  [--task ID] [--budget-tokens N] [--policy fixed] [--dag PATH]
# + globais --json / --no-color
# Guard FAIL → exit 6 (034)
# Timeout driver → 124
```

### Env (canónico — alinhar 024)

| Variável | Uso |
|----------|-----|
| `DARE_CODEX_COMMAND` | Override argv Codex |
| `DARE_CLAUDE_COMMAND` | Override argv Claude Code |
| `DARE_CURSOR_COMMAND` | Override argv Cursor |
| `DARE_ANTIGRAVITY_COMMAND` | Override argv Antigravity |
| `DARE_AGENT_MOCK` / `DARE_AGENT_SKIP_RALPH` | Inalterados (030) |

### API (extensão)

```text
// crates/dare-agent
pub fn resolve_driver(id: &str) -> CoreResult<Box<dyn AgentDriver>>;
// ids: mock|noop|codex|claude|cursor|antigravity
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | `doctor` local (PATH/stat) | < 2 s tipicamente |
| RNF-02 | Determinismo | Parse JSONL/ordem de eventos estável em testes | Golden fixtures |
| RNF-03 | Segurança | Sem secrets em logs/JSON/attempts | `redact` + auditoria testes |
| RNF-04 | Segurança | Spawn argv-only; sem shell | 100% SafeCommand |
| RNF-05 | Observabilidade | Tracing: driver id, exit code, duration, tokens opcional | Sem PII |
| RNF-06 | Manutenibilidade | ProcessRunner injetável (paridade `dare-ai` Codex) | Unit sem binário real |
| RNF-07 | Compat | Diffs vs TS classificados em DEC-037 | 0 Classe A sem ADR |
| RNF-08 | Cross-platform | PATH lookup Windows/Unix | Smokes/CI matrix |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--driver` id (allowlist) e paths de cwd já jailed pelo orquestrador | OWASP A03 |
| RS-02 | N/A senhas de usuário; secrets de CLI (API keys em env do subprocess) nunca ecoados | OWASP A02 |
| RS-03 | Driver não eleva privilégios; roda no worktree do projeto | OWASP A01 |
| RS-04 | `cargo audit` / deny sem HIGH/CRITICAL novas deps | OWASP A06 |
| RS-05 | Overrides só via env `DARE_*_COMMAND`; nunca hardcoded secrets | Supply chain |
| RS-06 | Argv-only; denylist env 006 no subprocess | RS-06 processo |
| RS-07 | Redaction obrigatória em stdout/stderr/summary antes de persistir attempt | Secrets |
| RS-08 | Guard preflight (034) não pode ser bypassado por driver | Exit 6 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | workspace `1.85.0` |
| Crate | `dare-agent` | estender; sem ciclo `dare-cli` |
| Processo | `dare-core` SafeCommand / CancelFlag | 006 |
| Redaction | `dare_core::redact` | 004 |
| CLI | `dare execute --agent` | 030 + wire |
| Testes | ProcessRunner mock / fixtures JSONL | unit + smoke |
| Externos | Codex / Claude Code / Cursor / Antigravity CLIs | instalados no host; CI com fake argv |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Codex CLI | Agent runtime | subprocess + JSONL | Saída | prompt → eventos/result/tokens | Time DARE |
| Claude Code CLI | Agent runtime | subprocess | Saída | prompt → stdout/stderr | Time DARE |
| Cursor Agent CLI | Agent runtime | subprocess | Saída | prompt → stdout/stderr | Time DARE |
| Antigravity CLI | Agent runtime | subprocess | Saída | prompt → stdout/stderr | Time DARE |
| Anthropic API SDK | LLM direto | — | — | **Fora deste ciclo** | — |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** Microplanos **006**, **024**, **030** concluídos (matriz); **034** já mergeado (preflight).
- **Limitações técnicas:** Trait **sync** (Classe B); sem `async_trait` neste ciclo.
- **IDs:** Manter ids curtos CLI (`claude`, não `claude-code`) — **Classe B** vs ids `dare-ai` ProviderId; documentar no DEC.
- **Sem** dependência nova de SDK Anthropic / HTTP client só para drivers.
- **CI:** Não exigir binários reais instalados; usar overrides + fake executables.
- **Regulatórias:** Sem PII em telemetria; redaction obrigatória.

---

## 10. FORA DO ESCOPO (v1 / este microplano)

- Política `decay`, REPLAN, FRESH_START, ESCALATE, `spliceSubDag` (→ **033**).
- Claude **API direta** / `@anthropic-ai/sdk` (Mestre §15.3 passo 6).
- `--require-approval rank|none` com TTY (→ posterior / 049+).
- Best-of-N / mutation / formal (→ **049**).
- Comando `dare ai` (→ **050**).
- Mudança de contrato de disco sem ADR.
- Reimplementar worktrees/budget/fixed (já **030**).
- Alterar surface Guard além do consumo do preflight existente.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | CLIs mudam formato JSONL/flags | Alta | Alto | Parser tolerante + fixtures versionadas; DEC Classe B |
| R-02 | Binário ausente em CI | Alta | Médio | Overrides + fake PATH; doctor não falha o build |
| R-03 | Leak de API key em logs | Média | Alto | `redact` obrigatório + testes secret redaction |
| R-04 | Timeout/hang de CLI real | Média | Alto | Timeout 006 + CancelFlag; smokes com fake |
| R-05 | Confusão ids `claude` vs `claude-code` (dare-ai) | Média | Baixo | DEC + docs tabela de ids |
| R-06 | Sandbox flags inseguros por default | Baixa | Alto | Defaults restritivos; documentar overrides |
| R-07 | Diff grande vs TS (SDK Claude) | Média | Médio | Aceitar Classe B: CLI-only neste ciclo |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo limitado a 4 drivers CLI + doctor + overrides + redaction + suite comum
- [ ] Confirmado: **sem** Anthropic SDK / Claude API direta neste ciclo
- [ ] Confirmado: ids CLI `codex|claude|cursor|antigravity` (não ProviderId de `dare-ai`)
- [ ] Trait sync + SafeCommand + exit codes (4/6/124) alinhados a 030/034
- [ ] DEC proposto **DEC-037**; docs `cli-execute-agent.md` a atualizar
- [ ] Fora de escopo 033/049/050 explícito
- [ ] Aprovado para `/dare-blueprint` → `DARE/BLUEPRINT-031-drivers-reais-de-agentes.md`

---

## Apêndice A — Suite comum (aceite microplano)

Para **cada** driver `codex|claude|cursor|antigravity`:

| Caso | Resultado esperado |
|------|-------------------|
| detection / doctor | `DriverHealth` coerente (`ok` true/false) |
| success | `AgentRunStatus::Success` + summary |
| failure | `Failure` + stderr redigido |
| timeout | `Timeout` → CLI **124** |
| cancellation | `Cancelled` |
| malformed output | `Failure` sem panic |
| missing executable | Diagnóstico claro |
| secret redaction | Nenhum secret em stdout/stderr/summary/logs |

---

## Apêndice B — Relação com 024 (`dare-ai`)

| Aspecto | `dare-ai` (024) | `dare-agent` (031) |
|---------|-----------------|---------------------|
| Propósito | Enrichment Design/Blueprint | Execução de task no worktree |
| Trait | `AiProvider` | `AgentDriver` |
| Ids | `codex`, `claude-code`, `cursor-cli`, `antigravity-cli` | `codex`, `claude`, `cursor`, `antigravity` |
| Env overrides | `DARE_*_COMMAND` | **Reutilizar as mesmas vars** |
| Reuso de código | `parse_argv_override` 🟡 Blueprint: reexport/deps `dare-ai` **ou** duplicar helper mínimo em `dare-agent` (evitar ciclo) | Congelar em Blueprint |

---

## Próximas etapas

1. Revisar e **aprovar** este Design (ajustar se necessário).
2. Executar `/dare-blueprint` gerando `DARE/BLUEPRINT-031-drivers-reais-de-agentes.md`.
3. Gerar `TASKS-031` + `dare-dag-031.yaml` + `EXECUTION-031/` e executar com Ralph Loop.
