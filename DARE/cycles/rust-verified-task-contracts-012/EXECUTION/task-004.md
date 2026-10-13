# Task 004: Primitivas de proveniência Git

> **Complexity:** MED · **depends_on:** — · **rank:** 0
> **Blueprint:** [§6, §3.2](../BLUEPRINT.md) · **RF:** RF-09 · **Decisões:** A-06

## Objetivo

Dar ao recibo a capacidade de apontar para uma revisão real — e de dizer honestamente quando não há revisão. Hoje [`git.rs`](../../../../crates/dare-project/src/git.rs) só descobre o toplevel; não existe helper de HEAD SHA nem de estado da working tree.

## Princípio inegociável

**Ausência de Git nunca produz revisão inventada.** Toda função devolve `Option` e degrada para `None`. O chamador traduz `None` em `RevisionState::Unattested`, que é evidência insuficiente — não uma revisão fabricada. Isso é RF-09 literal.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-project/src/git.rs` | editar — 3 funções novas |

## Especificação

Siga o estilo exato de `try_git_rev_parse` ([`git.rs:33`](../../../../crates/dare-project/src/git.rs#L33)): `SafeCommand` com `.cwd(root, rel)`, `.timeout(Duration::from_secs(5))`, `.stdout_limit(...)`, via `SystemProcessRunner`, e **nunca** `Err` por Git ausente.

### `head_sha(root) -> Option<String>`

`git rev-parse HEAD`. `Some` **somente** quando `exit_code == 0` **e** stdout trimado casa `^[0-9a-f]{40}$`. Repositório sem commits (rev-parse falha) → `None`.

### `worktree_is_dirty(root) -> Option<bool>`

`git status --porcelain`. `Some(true)` com stdout não vazio após trim, `Some(false)` vazio, `None` sem Git ou exit ≠ 0.

> Nota sobre o `stdout_limit`: uma árvore muito suja pode truncar a saída. Truncar ainda significa corretamente **dirty** — o `Some(true)` não depende de ler tudo.

### `changed_files(root, from_sha, to_sha) -> Option<Vec<String>>`

`git diff --name-only <from>..<to>`, ou `git diff --name-only <from>` quando `to_sha` é `None` (inclui a working tree).

**Guarda de injeção de argumento — obrigatória.** Validar `from_sha` e `to_sha` contra `^[0-9a-f]{7,40}$` **antes** de montar o comando, e devolver `None` em mismatch. Strings não validadas **não** chegam ao `git`: valores como `--upload-pack=evil` ou `--output=/etc/x` seriam interpretados como flags.

Retorno: paths relativos POSIX, ordenados, deduplicados. Entradas que escapam da raiz do projeto são descartadas. `stdout_limit` de **65536** só nesta chamada.

## Validation Gates

- [ ] `cargo test -p dare-project`
- [ ] `cargo clippy -p dare-project --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] Nenhuma dependência nova
- [ ] Suite passa em máquina **sem** `git` no PATH (testes de repo são skipados, não falham)

## Testes

| Teste | Comportamento |
|---|---|
| `head_sha_without_git_is_none` | tempdir sem `.git` → `None` |
| `worktree_dirty_without_git_is_none` | → `None` |
| `changed_files_without_git_is_none` | → `None` |
| `head_sha_shape_in_real_repo` | 40 hex lowercase |
| `worktree_clean_after_commit` | `Some(false)` |
| `worktree_dirty_after_touch` | `Some(true)` após alterar arquivo versionado |
| `changed_files_between_commits` | Lista exatamente o path alterado |
| `changed_files_rejects_flag_injection` | `--upload-pack=evil` → `None` **sem spawnar git** |
| `changed_files_rejects_shell_metachar` | `; rm -rf /` → `None` sem spawn |
| `changed_files_rejects_non_hex` | `zzz`, `HEAD`, `""` → `None` sem spawn |

Testes de repo real criam o repo via `SafeCommand` (`git init`, `git config user.email`, `git config user.name`, commit). Um helper `fn git_available() -> bool` faz `skip` quando `git` não está no PATH — CI sem git continua verde.

O teste de injeção precisa provar **ausência de spawn**, não só o `None`: use um `ProcessRunner` falso que falha o teste se for chamado.

## Segurança

| Risco | Controle |
|---|---|
| Injeção de argumento em `git` | Regex `^[0-9a-f]{7,40}$` antes de montar o comando; teste prova que não houve spawn |
| Comando pendurado | `timeout(5s)` como o helper existente |
| Saída gigante | `stdout_limit` explícito (4096 / 65536) |
| Path escape via `git diff` | Entradas fora da raiz descartadas |
| Revisão fabricada | Proibida por construção: `Option`, nunca default |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum SHA hardcoded fora de teste
- [ ] A validação de SHA acontece **antes** de construir o `SafeCommand`, não depois
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-004` sem findings

## Próxima Task

`task-005` depende desta e de `task-003`.
