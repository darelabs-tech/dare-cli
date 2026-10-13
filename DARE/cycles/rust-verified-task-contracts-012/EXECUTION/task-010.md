# Task 010: Emissão no `--complete`, supersede no `--reset` e campos JSON

> **Complexity:** HIGH · **depends_on:** `task-009` · **rank:** 5
> **Blueprint:** [§4.3](../BLUEPRINT.md) · **RF:** RF-02, RF-03, RF-04, RF-09, RF-10 · **Objetivo:** O-02, O-03

## Objetivo

Fechar o lado produtor: a conclusão de uma task passa a emitir recibo e handoff, a mudança material de contrato passa a ser recusada com caminho de recuperação, e o reset preserva a evidência anterior.

## Por que depende da `task-009`

Edita o **mesmo arquivo** (`commands/execute.rs`). A dependência serializa as duas para evitar conflito de merge em execução paralela com worktrees. Não é dependência de output lógico — é de propriedade de arquivo, declarada explicitamente.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-cli/src/commands/execute.rs` | editar — emissão, supersede, JSON |

## Especificação

### Emissão no `run_complete_inner`

**Depois** do veredito avançado passar **e** da `Transition::Complete` ser aplicada — para que o recibo descreva uma task que realmente alcançou `DONE`:

1. `load_contract(root, dag_rel, task_id)`. **`None` → não emite nada**, nenhum campo JSON novo, run idêntico ao de hoje (contratos são opcionais — D-06).
2. Recomputar `material_digest` e comparar com o digest gravado no `Start`. Divergência → abortar com:
   ```
   error: contract changed after task start: task-004 (HANDOFF_CONTRACT_DIGEST_MISMATCH)
          material fields (goal/scope/acceptance/exitCriteria) differ from the recorded digest
          to adopt the new contract: dare execute --reset task-004
   ```
   O digest do `Start` vai no `extra` da task em `.dare/state.json`, sob `"handoffContractDigest"`. `TaskRuntimeState` já preserva chaves desconhecidas via `#[serde(flatten)] extra` ([`state.rs:26`](../../../../crates/dare-contracts/src/state.rs#L26)). **Não adicionar campo obrigatório** ao `state.json` (RF-10) — e, como o npm descarta extras no `saveState`, a ausência do digest após um ciclo pelo npm é tratada como "sem digest registrado", não como divergência.
3. Montar `SourceRevision` com as primitivas da `task-004`: `head_sha` + `worktree_is_dirty` + `changed_files(start_sha, None)` por entrada de `contract.starting_context.repositories`, com alias `"self"` quando a lista é vazia. Sem Git → `RevisionState::Unattested`.
4. `receipt_from_verification` com `driver: "cli"` e `created_at` do clock existente.
5. `write_handoff` com `kind` de `contract.exit_criteria.kind`, `deliverable` = o mesmo output já truncado pela transição, `context_bounds = parent_context_limit(doc)`, `next_steps`/`open_items` do contrato quando houver.

**Falha nos passos 3–5 com a task já `DONE`:** **não** reverter o status. Surfaçar erro cuja mensagem diga que a task está `DONE` mas o recibo não foi escrito, para o operador reexecutar. Reverter um `DONE` legítimo porque a escrita do sidecar falhou seria pior que o problema.

### Supersede

- No caminho de `Reset`: `supersede_handoff(root, task_id, "task reset", clock.now_rfc3339())` **sem condicional** — é idempotente em record ausente.
- Em re-execução do `complete` sobre record existente: mesma chamada com `reason: "re-executed"`.

É o que cumpre RF-03 e o caso 5 do aceite.

### Campos JSON — aditivos

`complete_json` ([`execute.rs:686`](../../../../crates/dare-cli/src/commands/execute.rs#L686)) ganha, **somente quando um recibo foi escrito**:

```json
{ "handoffPath": ".dare/handoff/task-004.json", "receipt": { … } }
```

`--next --json`: cada item de `ready` ganha

```json
{ "handoff": { "strict": false, "imported": ["task-003"],
    "refused": [ { "parentId": "task-002", "state": "missing",
                   "code": "HANDOFF_RECEIPT_MISSING", "detail": "…" } ] } }
```

**Aditivo é normativo:** nada já emitido é renomeado, reordenado ou removido. Política de disco e JSON ([`disk-and-json-policy.md`](../../../../docs/compatibility/disk-and-json-policy.md)): "novo campo opcional permitido com default seguro".

## Validation Gates

- [ ] `cargo test -p dare-cli`
- [ ] `cargo clippy -p dare-cli --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `dare execute --complete` sem contrato produz stdout **idêntico** ao golden da `task-001`
- [ ] `.dare/state.json` não ganhou campo obrigatório — diff do schema vazio

## Testes

| Teste | Comportamento |
|---|---|
| `complete_with_contract_writes_valid_sidecar` | Sidecar existe; digests conferem ao recomputar |
| `complete_without_contract_writes_nothing` | Nenhum sidecar; stdout == golden `task-001` |
| `complete_refuses_material_change` | Exit ≠ 0; mensagem contém `HANDOFF_CONTRACT_DIGEST_MISMATCH` **e** `dare execute --reset` |
| `complete_allows_non_material_change` | Alterar `owner` entre Start e Complete → passa; `contractFullDigest` difere |
| `reset_supersedes_receipt` | `superseded.len() == 1`, `reason == "task reset"` |
| `recomplete_supersedes_with_reason` | `reason == "re-executed"` |
| `complete_json_has_handoff_fields` | `handoffPath` e `receipt` presentes |
| `complete_json_omits_fields_without_contract` | Campos ausentes, não `null` |
| `next_json_has_handoff_block` | Bloco `handoff` em cada `ready` |
| `receipt_failure_after_done_reports_honestly` | Status permanece `DONE`; mensagem diz que o recibo não foi escrito |
| `unattested_revision_when_no_git` | `sourceRevision.state == "unattested"`, sem SHA fabricado |
| `start_digest_stored_in_state_extra` | Chave `handoffContractDigest` em `extra`, e `state.json` carrega por `load_runtime_state` |

## Segurança

| Risco | Controle |
|---|---|
| Contrato alterado para aparentar sucesso (RF-02) | Recomputação do digest material no complete |
| Evidência anterior apagada em silêncio | `supersede_handoff` no reset e na re-execução |
| Revisão fabricada | `Option` das primitivas da `task-004` → `Unattested` |
| Campo novo quebrando o npm | Digest no `extra` do `state.json`; ausência tratada como "sem digest" |
| Segredo no sidecar | `redact` já aplicado na `task-006` |
| `DONE` revertido por falha de I/O | Proibido — erro honesto, status preservado |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum campo JSON existente renomeado, reordenado ou removido
- [ ] Nenhum campo obrigatório novo em `.dare/state.json`
- [ ] A mensagem de mudança material **nomeia `dare execute --reset`** (teste verifica a substring)
- [ ] `supersede_handoff` é chamada sem condicional no reset
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-010` sem findings

## Próxima Task

`task-011` depende desta e de `task-001`.
