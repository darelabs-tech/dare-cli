# Task 009: Política, flags `--strict-handoff` e preflight estrito

> **Complexity:** HIGH · **depends_on:** `task-007`, `task-008` · **rank:** 4
> **Blueprint:** [§3.4, §4.1](../BLUEPRINT.md) · **RF:** RF-05 · **Decisões:** D-03, D-06.1, A-07 · **Objetivo:** O-01, O-04

## Objetivo

Ligar a camada de prova ao CLI, atrás de um opt-in que preserva o comportamento default byte a byte.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-cli/src/commands/handoff_policy.rs` | **criar** |
| `crates/dare-cli/src/commands/handoff_preflight.rs` | **criar** |
| `crates/dare-cli/src/commands/mod.rs` | editar — declarar os dois módulos |
| `crates/dare-cli/src/main.rs` | editar — 2 flags no subcomando `Execute` |
| `crates/dare-cli/src/commands/execute.rs` | editar — resolver política, preflight, `CliParentEvidence` |
| `crates/dare-cli/src/commands/execute_agent.rs` | editar — idem no loop de tentativas |

## Especificação

### `handoff_policy_from_cfg`

```rust
pub struct HandoffFlags { pub strict: bool, pub no_strict: bool }
pub fn handoff_policy_from_cfg(cfg: &DareConfig, flags: &HandoffFlags) -> HandoffPolicy
```

Leitura de `cfg.extra` **exatamente** como [`verify_enabled_from_cfg`](../../../../crates/dare-verify/src/advanced.rs#L135) faz:

| Chave | Default |
|---|---|
| `handoff.strict` | `false` |
| `handoff.requireAttestedRevision` | `false` |
| `handoff.requireContracts` | = valor resolvido de `strict` |

Precedência: `--strict-handoff` > `--no-strict-handoff` > config > `false`.
`required_kind` é sempre `HandoffKind::VerifiedDeliverable` no v1.

**Não ampliar `dare.config.json` gerado pelo `init`** (RF-10). A chave é lida de `extra` quando presente; ausência total significa todos os defaults.

### `run_handoff_preflight` (D-06.1)

```rust
pub struct PreflightReport {
    pub schema_version: u32,   // 1
    pub strict: bool,
    pub tasks_total: u32,
    pub tasks_with_contract: u32,
    pub violations: Vec<(String, ContractViolation)>,
}
pub fn run_handoff_preflight(
    root: &ProjectRoot, dag_rel: &SafeRelativePath,
    doc: &DagDocument, policy: &HandoffPolicy,
) -> CoreResult<PreflightReport>
```

Chamada **somente** quando `policy.strict`. Com `strict == false` não é invocada — custo zero no caminho default.

| Situação | Resultado |
|---|---|
| `tasks_with_contract == 0` | `CoreError::guard_fail("handoff preflight failed: strict mode enabled but no task contract found under {dir}")` → exit **6**, código `HANDOFF_STRICT_NO_CONTRACTS` |
| Diretório `contracts/` inexistente | Mesma recusa, e a mensagem **nomeia o diretório esperado** |
| Violações bloqueantes | `guard_fail` listando no máximo 10 + o total |
| ≥ 1 contrato e nenhuma violação | `Ok(report)` |

Siga a forma de [`dare-guard/src/preflight.rs::run_preflight`](../../../../crates/dare-guard/src/preflight.rs#L16).

**Por que o preflight existe (D-06.1).** Sem ele, quem liga `handoff.strict` num projeto sem contratos descobre isso task por task, no meio do run, com `HANDOFF_CONTRACT_MISSING` repetido. Separar "configurei errado" (uma falha, no início) de "falta evidência nesta task" (falha no consumo) é o que a linha A05 do DESIGN §7 exige.

### Flags no `main.rs`

```rust
/// Enforce verified handoffs before importing parent context (config `handoff.strict`).
#[arg(long = "strict-handoff", action = clap::ArgAction::SetTrue,
      overrides_with = "no_strict_handoff")]
strict_handoff: bool,

/// Disable strict handoff enforcement for this run.
#[arg(long = "no-strict-handoff", action = clap::ArgAction::SetTrue)]
no_strict_handoff: bool,
```

Sem `conflicts_with` além do par mútuo. **Nenhuma flag existente muda de semântica.**

### Wiring nos dois caminhos

- `strict == true`: preflight antes de qualquer composição, depois `compose_task_prompt_verified` com um `CliParentEvidence` que delega a `dare_verify::handoff::evaluate_parent`
- `strict == false`: continua chamando `compose_task_prompt` original — **o caminho default não passa pelo código novo**

Em modo compat, uma recusa imprime **só em stderr**:

```
warning: handoff from task-002 not verified (HANDOFF_RECEIPT_MISSING); imported anyway
         (compatibility mode — this is not verification; enable handoff.strict to block)
```

e o prompt é composto exatamente como hoje. **stdout não muda** — é o que o golden da `task-001` verifica.

## Validation Gates

- [ ] `cargo test -p dare-cli`
- [ ] `cargo clippy -p dare-cli --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `dare execute --help` lista as 2 flags novas e nenhuma existente mudou de texto
- [ ] Com `strict == false`, nenhuma chamada a `evaluate_parent` acontece (provar com contador em teste)

## Testes

| Teste | Comportamento |
|---|---|
| `policy_default_is_not_strict` | Config ausente → `strict == false` |
| `policy_from_config_true` | `{"handoff":{"strict":true}}` → `true` |
| `policy_flag_overrides_config_false` | `--strict-handoff` + config `false` → `true` |
| `policy_no_strict_overrides_config_true` | `--no-strict-handoff` + config `true` → `false` |
| `policy_require_contracts_follows_strict` | Default de `requireContracts` acompanha `strict` |
| `policy_require_attested_default_false` | `false` |
| `preflight_zero_contracts_v21` | `guard_fail`, exit 6, mensagem nomeia o diretório |
| `preflight_zero_contracts_legacy` | Idem para DAG legacy |
| `preflight_missing_dir_names_it` | Mensagem contém o path esperado |
| `preflight_passes_with_one_of_three` | `Ok`, `tasks_with_contract == 1` |
| `preflight_reports_violations_capped` | > 10 violações → lista 10 + total |
| `preflight_not_called_when_not_strict` | Contador de chamadas == 0 |
| `strict_refusal_exits_6` | Exit code 6, `error.kind == "GuardFail"` no `--json` |
| `compat_warns_on_stderr_only` | Exit 0, stdout == golden da `task-001`, aviso em stderr |

## Segurança

| Risco | Controle |
|---|---|
| Achar que está protegido sem estar (A05) | Preflight falha uma vez com código próprio; aviso compat nunca afirma validade |
| Mudança silenciosa de comportamento | `strict` default `false`; caminho default não executa código novo |
| Fail-open em estrito | `guard_fail` é propagado; nenhum `unwrap_or_default` na avaliação |
| Vazamento em mensagem | Toda mensagem passa pelo `redact` do renderer ([`output.rs`](../../../../crates/dare-cli/src/output.rs)) |
| Exit code novo observável por quem não optou | Impossível: exit 6 só é alcançável com `strict == true` |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum `unwrap_or(false)` que transforme erro de avaliação em "aceito"
- [ ] `CliParentEvidence::decide` delega a `evaluate_parent` — não reimplementa checagem
- [ ] A matriz de precedência tem **6 testes separados**, não um agregado
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-009` sem findings

## Próxima Task

`task-010` depende desta (mesmo arquivo `execute.rs`, serializado de propósito).
