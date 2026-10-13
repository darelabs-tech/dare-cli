# Task 002: `dare-contracts::handoff` — tipos, parse, validate e paths

> **Complexity:** HIGH · **depends_on:** — · **rank:** 0
> **Blueprint:** [§2.1, §2.2, §2.3, §3.1](../BLUEPRINT.md) · **RF:** RF-01, RF-10 · **Decisões:** A-01, A-04

## Objetivo

Criar a fundação de tipos e validação do Task Contract e do Handoff no crate que **os dois lados já veem** (`dare-dag` e `dare-verify` ambos dependem de `dare-contracts`). Nenhuma lógica de digest aqui — é `task-003`.

## Contexto arquitetural

A-01: não existe crate novo. Os tipos moram em `dare-contracts` porque é o crate de contratos persistidos (`dag.rs`, `state.rs`, `config.rs`) e porque `dare-dag` **não pode** depender de `dare-verify` sem inverter o layering.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-contracts/src/handoff.rs` | **criar** |
| `crates/dare-contracts/src/lib.rs` | editar — `pub mod handoff;` + re-exports |
| `crates/dare-contracts/Cargo.toml` | editar — `sha2 = { workspace = true }` |

## Especificação

Tipos, constantes e assinaturas completas estão no `subtask_prompt` desta task no [`dare-dag.yaml`](../dare-dag.yaml) e em [BLUEPRINT §2.1–2.3 e §3.1](../BLUEPRINT.md). Resumo das decisões que não podem ser reinterpretadas:

- **`parse` e `validate` são separados.** `parse_contract_yaml` só falha em YAML malformado ou `schemaVersion != 1`. Regras de negócio ficam em `validate_contract`. Isso espelha `parse_dag_yaml` vs `validate_dag` ([`validate.rs:352`](../../../../crates/dare-dag/src/validate.rs#L352)) e é o que permite o preflight reportar tudo de uma vez.
- **`validate_contract` retorna `Vec`, nunca `Err`.** Acumula violações em ordem determinística (ordem de campo declarado, depois índice).
- **`load_contract` devolve `Ok(None)` para arquivo ausente.** Ausência de contrato é estado legítimo (D-06), não erro.
- **A-04:** `contract_dir_for_dag(dag_rel) = parent(dag_rel)/"contracts"`. DAG na raiz → `contracts/`. `dag_rel == "."` → `InvalidInput`.
- **Não adicionar `dare-verify` como dependência.** O predicado de id seguro é copiado localmente (poucas linhas, mesma regra de `task_id_is_path_safe`).

### Constraints de campo

Tabela completa em [BLUEPRINT §2.1](../BLUEPRINT.md). Os limites numéricos são normativos: `goal` 1..=500, `allowedPaths` 1..=64, `acceptance` 1..=64, `requiredGates` 1..=3, `requirementRefs` 0..=32, `repositories` 0..=8.

### `extra` em todo tipo persistido

`TaskContract` e `HandoffRecord` carregam `#[serde(flatten)] extra: Map<String, Value>`, como `DagTask` ([`dag.rs:44`](../../../../crates/dare-contracts/src/dag.rs#L44)) e `DareConfig` já fazem. Round-trip preservando chaves desconhecidas é requisito, não cortesia.

## Validation Gates

- [ ] `cargo build -p dare-contracts`
- [ ] `cargo test -p dare-contracts`
- [ ] `cargo clippy -p dare-contracts --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo tree -p dare-contracts` não mostra `dare-verify` nem `dare-dag`

## Testes

| Teste | Comportamento |
|---|---|
| `parse_contract_roundtrip_preserves_extra` | Chave desconhecida sobrevive a parse → serialize |
| `parse_rejects_unsupported_schema_version` | `schemaVersion: 2` → `CoreError::Config` |
| `parse_rejects_malformed_yaml` | Mensagem contém `invalid contract yaml` |
| `validate_produces_every_violation` | Cada variante de `ContractViolation` é produzida por ≥ 1 caso |
| `validate_is_deterministic` | Mesma entrada → mesma ordem de violações, 2 execuções |
| `validate_accepts_minimal_valid_contract` | Contrato mínimo válido → `vec![]` |
| `contract_dir_cycle_dag` | `DARE/cycles/x/dare-dag.yaml` → `DARE/cycles/x/contracts` |
| `contract_dir_root_dag` | `dare-dag.yaml` → `contracts` |
| `contract_dir_dot_rejected` | `.` → `InvalidInput` |
| `unsafe_task_ids_rejected` | `a/b`, `../x`, `-bad`, `""` → `InvalidInput` |
| `load_contract_missing_is_none` | Arquivo ausente → `Ok(None)` |
| `load_contract_oversize_rejected` | > 2 MiB → `Config` com `size limit` |
| `refusal_code_roundtrip` | `as_str`/`parse` exato nos **13** variantes |

## Segurança

- Todo path construído passa por `SafeRelativePath::new`; nenhum `format!` de path vai direto ao filesystem.
- `load_contract` usa `read_limited` (2 MiB) — arquivo malicioso barrado antes do parse, prevenindo OOM.
- `task_id` validado antes de virar segmento de path (mesma regra que já protege `.dare/verification/`).
- `owner` é apenas armazenado aqui; a redação acontece na emissão (`task-005`).

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`, `panic!("not implemented")`
- [ ] Nenhum `Ok(Default::default())` ou `return None` como único corpo de função pública
- [ ] Todas as 13 variantes de `RefusalCode` têm string de wire e são cobertas pelo teste de round-trip
- [ ] Toda variante de `ContractViolation` é **produzível** — um teste a produz
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-002` sem findings

## Próxima Task

`task-003` (digests) depende desta. `task-008` também.
