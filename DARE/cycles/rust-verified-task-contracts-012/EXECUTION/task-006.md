# Task 006: Store do sidecar — escrita atômica, lock e supersede

> **Complexity:** MED · **depends_on:** `task-003` · **rank:** 2
> **Blueprint:** [§3.2 I/O do sidecar, §2.2](../BLUEPRINT.md) · **RF:** RF-04, RF-08 parcial, RF-10 · **Decisões:** D-02

## Objetivo

Persistir o `HandoffRecord` em `.dare/handoff/<task-id>.json` com as mesmas garantias que `.dare/state.json` já tem: escrita atômica, lock fail-fast, JSON canônico e redação de segredos.

## Contexto: por que arquivo separado (D-02 / RF-10)

O npm `packages/cli/src/dag-runner/state-store.ts::saveState` reconstrói `.dare/state.json` com campos conhecidos e descarta extensões. Um sidecar em arquivo próprio sobrevive a isso por construção — é o que `task-011` prova no round-trip.

**Arquivo separado de `handoff.rs` também por outra razão:** `task-005` e esta rodam no mesmo rank. Cada uma é dona de um arquivo, então nunca há conflito de merge em execução paralela.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-verify/src/handoff_store.rs` | **criar** |
| `crates/dare-verify/src/lib.rs` | editar — `pub mod handoff_store;` |

## Especificação

### `write_handoff(root, record) -> CoreResult<()>`

Ordem de operações normativa:

1. Rejeitar id inseguro → `CoreError::invalid_input("unsafe handoff task id: {id}")`
2. Rejeitar `record.schema_version != 1` → `CoreError::Config`
3. Rejeitar `kind == FollowUpHandoff` com `open_items` vazio → `CoreError::config("follow-up handoff requires at least one open item")` — um follow-up sem pendência declarada é um `DONE` disfarçado (D-05)
4. `redact` em `deliverable` e em **cada** entrada de `next_steps` e `open_items`, sobre um **clone** do record — exatamente como `write_verification` redige `stdout_tail`/`stderr_tail` ([`verification.rs:63`](../../../../crates/dare-verify/src/verification.rs#L63))
5. `serde_json::to_value` + `dare_core::to_canonical_json_string`
6. `FileLock::try_acquire` em `.dare/handoff/<id>.json.darelock`
7. `atomic_write` em `.dare/handoff/<id>.json`

**Contenção de lock é fail-fast.** Deixe o `CoreError::io("file lock held")` propagar. Sem fila, sem retry, sem backoff — mesmo contrato de `.dare/state.json` documentado em [`dag-runtime.md`](../../../../docs/compatibility/dag-runtime.md).

### `load_handoff(root, task_id) -> CoreResult<HandoffRecord>`

`read_limited` (2 MiB) + `from_json_slice`. `schema_version != 1` → `CoreError::config("unsupported handoff schema version")`.

### `handoff_exists(root, task_id) -> CoreResult<bool>`

Checagem sem desserializar. Usada pela checagem 1 de `task-007`.

### `supersede_handoff(root, task_id, reason, at) -> CoreResult<()>`

1. Carregar o record. **Ausente → `Ok(())`** — idempotente, para poder ser chamada sem condicional no `Reset`.
2. Empilhar em `superseded`: `SupersededReceipt { receipt_digest: digest do recibo canônico atual, created_at: at, reason: redact(reason) }`
3. Truncar FIFO (descarta o **mais antigo**) em `MAX_SUPERSEDED` (16)
4. Regravar atomicamente sob o mesmo lock

É o que cumpre RF-03 ("reexecução não sobrescreve evidência anterior silenciosamente") e o caso 5 do aceite.

## Validation Gates

- [ ] `cargo test -p dare-verify`
- [ ] `cargo clippy -p dare-verify --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `git diff --name-only` **não** mostra `crates/dare-verify/src/handoff.rs` (arquivo da `task-005`)

## Testes

| Teste | Comportamento |
|---|---|
| `write_load_roundtrip` | Round-trip exato; arquivo em disco com chaves em ordem canônica |
| `write_redacts_deliverable` | `"token=abc123"` → `[REDACTED]` no disco |
| `write_redacts_next_steps_and_open_items` | Cada entrada redigida |
| `write_rejects_unsafe_id` | `a/b`, `../x` → `InvalidInput` |
| `write_rejects_bad_schema_version` | `2` → `Config` |
| `write_rejects_followup_without_open_items` | `Config` com `requires at least one open item` |
| `load_rejects_oversize` | > 2 MiB → `Config` com `size limit` |
| `load_rejects_bad_schema_version` | `Config` |
| `supersede_missing_is_ok` | Record ausente → `Ok(())` |
| `supersede_twice_accumulates` | 2 entradas em `superseded` |
| `supersede_fifo_cap` | Na 17ª, `len() == 16` e a **mais antiga** saiu |
| `supersede_redacts_reason` | Segredo no `reason` sai redigido |
| `write_fails_fast_when_lock_held` | Lock já adquirido → `Io` mencionando lock, sem travar |

## Segurança

| Risco | Controle |
|---|---|
| Segredo no sidecar | `redact` em 4 campos antes de serializar |
| Path traversal | `task_id_is_path_safe` + `SafeRelativePath` + `ProjectRoot::resolve` |
| Escrita parcial / corrupção | `atomic_write` (tmp + rename) |
| Corrida entre processos | `FileLock` fail-fast |
| OOM na leitura | `read_limited` 2 MiB |
| Evidência apagada em silêncio | `supersede_handoff` preserva histórico; nada sobrescreve `receipt` sem registrar |
| `follow-up` passando por `DONE` | Rejeitado na escrita quando não há pendência |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum `fs::write` direto — somente `atomic_write`
- [ ] O `redact` é aplicado a um **clone**; o record do chamador não é mutado
- [ ] `supersede_handoff` é idempotente e tem teste provando isso
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-006` sem findings

## Próxima Task

`task-007` depende desta e de `task-005`.
