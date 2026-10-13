# Task 008: Porta `ParentEvidence` e `compose_task_prompt_verified`

> **Complexity:** MED · **depends_on:** `task-003` · **rank:** 2
> **Blueprint:** [§3.3](../BLUEPRINT.md) · **RF:** RF-05 · **Decisões:** A-02, A-03 · **Objetivo:** O-04

## Objetivo

Inserir o gate de verificação na composição do prompt **sem** `dare-dag` ganhar uma dependência nova e **sem** alterar a função existente.

## Contexto: a inversão de dependência (A-02)

`compose_task_prompt` mora em `dare-dag`, que depende **apenas** de `dare-core` e `dare-contracts` ([`Cargo.toml`](../../../../crates/dare-dag/Cargo.toml)). Ler recibos exigiria `dare-verify`, que depende de `dare-agent` e `globset` — a camada de processo. Isso inverteria o layering do crate de domínio.

A porta resolve: o trait é declarado em `dare-dag`, implementado em `dare-cli` (`task-009`). `HandoffDecision` mora em `dare-contracts`, visível aos dois.

**Proibido nesta task:** adicionar qualquer linha a `crates/dare-dag/Cargo.toml`.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-dag/src/execution.rs` | editar — adicionar trait, struct e a função nova |
| `crates/dare-dag/src/lib.rs` | editar — re-export |

## Especificação

```rust
pub trait ParentEvidence {
    fn decide(&self, parent_id: &str) -> HandoffDecision;
}

pub struct ComposedPrompt {
    pub prompt: String,
    pub imported: Vec<String>,
    pub refused: Vec<HandoffDecision>,
}

pub fn compose_task_prompt_verified(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    task_id: &str,
    ev: &dyn ParentEvidence,
) -> Result<ComposedPrompt, DagGraphError>
```

Espelhe [`compose_task_prompt`](../../../../crates/dare-dag/src/execution.rs#L115) linha a linha e insira o gate:

1. Task não encontrada → `DagGraphError::InvalidDag { message: format!("task not found: {task_id}") }` — **a mesma string de hoje**, caractere por caractere
2. `limit = parent_context_limit(doc)`; `deps = meta.depends_on` **ordenado**
3. Por dep: ausente de `state` → skip; `status != DONE` → skip (inalterado); então `ev.decide(dep_id)`
4. `state == Valid` → `imported.push(dep_id)` e anexar a seção usando o `deliverable` da decisão:
   `"### From parent: {dep_id} — {title}\n{unicode_tail(deliverable, limit)}\n"`
   (trim antes; `deliverable` vazio → pai omitido, como hoje)
5. Qualquer outro estado → `refused.push(decision)` e **nada** é anexado
6. Nenhuma seção anexada → `prompt == meta.subtask_prompt`, **sem** o cabeçalho `## Upstream context`. Caso contrário `format!("{}\n\n## Upstream context\n\n{}", subtask_prompt, upstream)`

`imported` e `refused` em ordem lexicográfica das deps.

### O que não pode ser tocado (A-03)

`compose_task_prompt` e `build_next_report` ficam **intactas**. Saída legada byte-idêntica é contrato, não aspiração.

> `build_next_report` ([`execution.rs:277`](../../../../crates/dare-dag/src/execution.rs#L277)) também chama `compose_task_prompt` — ou seja, `dare execute --next` compõe prompts. O DESIGN só havia citado o caminho do `--agent`. Os dois estão no escopo, mas a mudança de chamador é da `task-009`.

## Validation Gates

- [ ] `cargo test -p dare-dag`
- [ ] `cargo clippy -p dare-dag --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `git diff crates/dare-dag/Cargo.toml` **vazio**
- [ ] `git diff` em `execution.rs` não altera o corpo de `compose_task_prompt` nem de `build_next_report`

## Testes

Todos com um `ParentEvidence` **falso**, sem tocar o filesystem — é o ganho da porta.

| Teste | Comportamento |
|---|---|
| `verified_matches_legacy_no_parents` | `assert_eq!` entre as duas funções |
| `verified_matches_legacy_one_parent` | `assert_eq!` entre as duas funções |
| `verified_matches_legacy_two_parents_truncating` | `assert_eq!`, com `parent_context_chars` forçando `unicode_tail` |
| `all_refused_yields_bare_prompt` | `prompt == subtask_prompt`, sem cabeçalho, `refused.len() == 2` |
| `mixed_accept_refuse_keeps_only_accepted` | Só a seção do pai aceito |
| `done_parent_with_empty_deliverable_skipped` | Pai omitido, como hoje |
| `legacy_dag_behaves_as_today` | `subtask_prompt` vazio → mesma saída da função legada |
| `task_not_found_exact_legacy_message` | String idêntica |
| `imported_and_refused_are_sorted` | Ordem lexicográfica |

O stub de `ParentEvidence` que devolve `Valid` com `deliverable == state.output` para todo pai `DONE` é o que prova a equivalência byte a byte nos 3 primeiros testes. Esse é o teste que sustenta O-04 no nível de biblioteca; `task-011` sustenta no nível de CLI.

## Segurança

- Pai recusado **não entra** no prompt — é a mitigação de prompt injection (OWASP A03) no ponto de consumo.
- `unicode_tail` + `parent_context_limit` continuam limitando volume; nenhum log completo entra no prompt.
- Nenhum I/O nesta função: todo acesso a disco fica atrás da porta, no implementador. Isso mantém a superfície de ataque de `dare-dag` inalterada.

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()`
- [ ] `compose_task_prompt` tem **zero** linhas alteradas
- [ ] Os 3 testes de equivalência usam `assert_eq!` entre as duas funções reais, não snapshots
- [ ] Mocks apenas em `#[cfg(test)]` (o `ParentEvidence` falso vive lá)
- [ ] `dare review task-008` sem findings

## Próxima Task

`task-009` depende desta e de `task-007`.
