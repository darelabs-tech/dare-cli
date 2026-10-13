# Task 005: Emissão do recibo e satisfação dos gates exigidos

> **Complexity:** HIGH · **depends_on:** `task-003`, `task-004` · **rank:** 2
> **Blueprint:** [§3.2](../BLUEPRINT.md) · **RF:** RF-03, RF-09 · **Objetivo:** O-03

## Objetivo

Emitir o `VerificationReceipt` a partir da evidência que **já existe em disco**, e checar que os gates exigidos pelo contrato realmente passaram. É aqui que a prova deixa de ser circular.

## O controle central: prova não circular

`receipt_from_verification` **lê** `.dare/verification/<task_id>.json` do disco via `load_verification`. Ela **não aceita** um `RalphReport`, nem um `bool`, nem um `&VerificationReport` como parâmetro.

Se a função aceitasse o relatório como argumento, quem a chama poderia construir um `ok: true` em memória e obter um recibo válido — a prova circular que o DESIGN §7/A04 proíbe. Ler do disco força a evidência a ter sido realmente produzida e persistida.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-verify/src/handoff.rs` | **criar** |
| `crates/dare-verify/src/lib.rs` | editar — `pub mod handoff;` + re-exports |
| `crates/dare-verify/Cargo.toml` | editar — `dare-project = { workspace = true }` |

> `task-006` cria `handoff_store.rs`, arquivo **separado**, para que as duas tasks de rank 2 nunca editem o mesmo arquivo.

## Especificação

### `gates_satisfied(report, required) -> Result<(), Vec<GateAspect>>`

`Ok(())` **se e somente se** para todo `a ∈ required` existe `step ∈ report.aspects` com `step.aspect == a`, `step.exit_code == 0` e `!step.timed_out`.

**Não confie em `report.ok`.** Um relatório com `ok == true` mas sem um aspecto exigido em `aspects` tem que devolver `Err`. O agregado é conveniência; a prova é por aspecto.

| Caso | Resultado |
|---|---|
| Todos os exigidos presentes e aprovados | `Ok(())` |
| Aspecto exigido ausente de `aspects` | `Err([faltante])` mesmo com `ok == true` |
| Aspecto presente com `exit_code != 0` | `Err` |
| Aspecto presente com `timed_out == true` | `Err` |
| Aspecto duplicado, um aprovado | `Ok(())` — basta um |
| `required` vazio | `Ok(())` (`validate_contract` já rejeita contrato assim) |

### `receipt_from_verification(root, req) -> CoreResult<VerificationReceipt>`

```rust
pub struct ReceiptRequest<'a> {
    pub task_id:    &'a str,
    pub contract:   &'a TaskContract,
    pub revision:   &'a SourceRevision,
    pub created_at: &'a str,   // Clock injetado pelo chamador
    pub driver:     &'a str,
}
```

`created_at` vem como `&str` porque o trait `Clock` mora em `dare-dag` ([`state.rs:21`](../../../../crates/dare-dag/src/state.rs#L21)), não em `dare-core` — mesmo padrão de `verification_from_ralph(task_id, ralph, updated_at)`.

| Condição | Resultado |
|---|---|
| Relatório ausente | `CoreError::NotFound` — chamador mapeia para `Insufficient`, **sem** emitir recibo |
| `report.ok == false` | `CoreError::Config("refusing to emit receipt for failed verification")` |
| Caps de digest excedidos | `CoreError::Config` propagado de `task-003` |
| Sucesso | Recibo com `verdict: Passed` |

Campos derivados:
- `verification_digest` = `digest_file` do **mesmo** path lido
- `contract_digest` = `material_digest`, `contract_full_digest` = `full_digest`
- `producer` = `{ cli_version: env!("CARGO_PKG_VERSION"), driver: req.driver }`
- `contract.owner`, se presente, passa por `dare_core::redact`

### Estratégia de artefatos (A-06)

| `revision.state` | `artifact_refs` / `artifact_digests` | `worktree_digest` |
|---|---|---|
| `Clean` | Arquivos alterados fornecidos pelo chamador ∩ globs de `allowed_paths`, ordenados, digest por arquivo | ausente |
| `Dirty` | Idem | `digest_tree` sobre `allowed_paths`, no `RepoRevision` correspondente |
| `Unattested` | vazio | `digest_tree` sobre `allowed_paths` |

Com `Unattested` o recibo **é** emitido. A insuficiência é decidida no **consumo** pela política (`task-007`), não na emissão — separar as duas é o que permite modo compat sem Git.

**Invariante:** `artifact_refs.len() == artifact_digests.len()`, mesmo índice = mesmo arquivo. Assertar antes de retornar.

## Validation Gates

- [ ] `cargo test -p dare-verify`
- [ ] `cargo clippy -p dare-verify --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `receipt_from_verification` **não** tem parâmetro de relatório/booleano em sua assinatura

## Testes

| Teste | Comportamento |
|---|---|
| `gates_all_pass` | `Ok(())` |
| `gates_missing_aspect_despite_ok_true` | `Err` — o teste central do anti-circularidade |
| `gates_failed_exit_code` | `Err` |
| `gates_timed_out` | `Err` |
| `gates_duplicate_one_passing` | `Ok(())` |
| `gates_empty_required` | `Ok(())` |
| `receipt_refuses_missing_report` | `NotFound` |
| `receipt_refuses_failed_report` | `Config` com `refusing to emit receipt` |
| `receipt_digests_match_recomputed` | Digests conferem com recomputação independente |
| `receipt_clean_revision_shape` | `refs` populados, `worktree_digest` ausente |
| `receipt_dirty_revision_shape` | `refs` + `worktree_digest` |
| `receipt_unattested_revision_shape` | `refs` vazio, `worktree_digest` presente, recibo emitido |
| `receipt_refs_and_digests_same_len` | Invariante |
| `receipt_redacts_owner` | `owner: "token=abc123"` sai redigido |

`created_at` fixo em string literal para assertions exatas.

## Segurança

| Risco | Controle |
|---|---|
| Prova circular | Relatório lido do disco; assinatura sem parâmetro de relatório |
| Recibo sobre gate reprovado | `report.ok == false` → `Config`, sem recibo |
| Segredo em `owner` | `redact` na emissão |
| OOM | Caps de `task-003` propagados |
| Revisão fabricada | `SourceRevision` vem do chamador, que usa as `Option` de `task-004` |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum `ok: true` construído em memória no caminho de produção
- [ ] Os 6 casos de `gates_satisfied` são **6 testes separados**
- [ ] Os 3 estados de revisão têm **3 testes separados**
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-005` sem findings

## Próxima Task

`task-007` depende desta e de `task-006`.
