# Task 003: Digests determinísticos — contrato, arquivo e árvore

> **Complexity:** MED · **depends_on:** `task-002` · **rank:** 1
> **Blueprint:** [§3.1 Digests](../BLUEPRINT.md) · **Decisões:** A-05, A-06, A-08 · **Objetivo:** O-02

## Objetivo

A camada de integridade. É o que faz "alterar um byte invalida a importação" ser verdade mecânica em vez de intenção.

## Contexto: por que dois digests de contrato (A-05)

O DESIGN tinha uma contradição: `contractDigest` cobria "o contrato" **e** mudança não material (`owner`, `requirementRefs`) deveria apenas "deixar trilha auditável". Com um digest único sobre o contrato inteiro, mexer em `owner` invalidaria a importação.

| Digest | Cobre | Divergência significa |
|---|---|---|
| `material_digest` | Conjunto **fixo**: `schemaVersion`, `taskId`, `goal`, `scope`, `acceptance`, `verification`, `exitCriteria`, `startingContext` | Recusa — `HANDOFF_CONTRACT_DIGEST_MISMATCH` |
| `full_digest` | Contrato canônico inteiro, `extra` incluído | Trilha auditável, sem recusa |

A projeção material é um **conjunto fixo de chaves**, não "tudo menos os opcionais". Um campo novo adicionado ao tipo no futuro não entra na projeção por acidente.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-contracts/src/handoff.rs` | editar — adicionar camada de digest |
| `crates/dare-contracts/Cargo.toml` | editar — `globset = { workspace = true }` |

## Especificação

### Algoritmo de `material_digest` (normativo)

1. Projetar no conjunto fixo de 8 chaves materiais. Descartar tudo o mais, incluindo `extra`.
2. `serde_json::to_value` da projeção.
3. `dare_core::to_canonical_json_string` — ordena chaves recursivamente ([`lib.rs:35`](../../../../crates/dare-core/src/lib.rs#L35)).
4. `digest_bytes` dos bytes UTF-8 → hex lowercase, 64 chars.

`full_digest`: passos 2–4 sobre o contrato inteiro.

### `digest_file`

SHA-256 em streaming, blocos de **64 KiB**. Nunca `fs::read` do arquivo inteiro — um artefato grande não pode derrubar o processo.

### `digest_tree`

```rust
pub struct TreeDigest {
    pub digest: String,
    pub file_count: u32,
    pub total_bytes: u64,
    pub skipped_symlinks: u32,
}
```

1. Expandir `globs` com `globset` contra a raiz do projeto.
2. Coletar **somente arquivos regulares**.
3. Ordenar por path POSIX (normalizar `\` → `/` **antes** de ordenar).
4. Alimentar um único hasher com `"{path}\n{file_digest}\n"` por entrada.

**Determinismo é o ponto inteiro.** O resultado não pode depender da ordem de `read_dir`, do SO nem do separador de path. Zero matches → digest da string vazia com `file_count == 0` (determinístico, não erro).

**Symlinks não são seguidos.** Contados em `skipped_symlinks`, excluídos do digest. Seguir symlink aqui seria o vetor de escape que o §7 do DESIGN proíbe.

### Caps (A-06)

`file_count > MAX_ARTIFACT_REFS` (512) ou `total_bytes > MAX_TREE_DIGEST_BYTES` (64 MiB) → `CoreError::config` com `"size limit"` na mensagem, mapeado a `HANDOFF_SIZE_LIMIT` no consumo.

## Validation Gates

- [ ] `cargo test -p dare-contracts`
- [ ] `cargo clippy -p dare-contracts --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] Nenhuma dependência nova além de `globset` (já no workspace — A-08)

## Testes

| Teste | Comportamento |
|---|---|
| `material_digest_stable_on_non_material_change` | Alterar `owner`, `requirementRefs` e uma chave de `extra` → digest **igual** |
| `material_digest_changes_on_goal` | Digest diferente |
| `material_digest_changes_on_scope` | Digest diferente |
| `material_digest_changes_on_acceptance` | Digest diferente |
| `material_digest_changes_on_exit_criteria` | Digest diferente |
| `full_digest_changes_on_both` | Muda nos dois casos acima |
| `material_digest_stable_across_yaml_key_order` | Dois YAML com chaves em ordens diferentes → mesmo digest |
| `digest_file_known_vector` | SHA-256 de `"abc"` == vetor conhecido |
| `digest_file_streams_large` | Arquivo de 5 MiB digere sem carregar em memória |
| `digest_file_missing_is_not_found` | `CoreError::NotFound` |
| `digest_tree_stable_under_shuffle` | Criar arquivos em ordens diferentes → mesmo digest |
| `digest_tree_zero_matches` | `file_count == 0`, digest determinístico, não erro |
| `digest_tree_rejects_file_count_cap` | > 512 arquivos → `Config` com `size limit` |
| `digest_tree_rejects_bytes_cap` | > 64 MiB → `Config` com `size limit` |
| `digest_tree_skips_symlink` | `#[cfg(unix)]` — contado em `skipped_symlinks`, fora do digest |

**Proptest:** `material_digest` invariante sob permutação das chaves do mapa.

## Segurança

- Hash sobre os **bytes efetivos** do arquivo, nunca sobre metadados ou caminho só.
- Symlink não seguido — fecha o vetor de escape via glob.
- Caps de tamanho fecham OOM por arquivo/árvore maliciosa.
- Expansão de glob confinada: todo path resolvido via `ProjectRoot::resolve`, que já nega escape por symlink ([`path.rs:148`](../../../../crates/dare-core/src/path.rs#L148)).
- `digest_bytes` não é assinatura (D-04). Nenhum comentário ou doc desta task pode sugerir autenticidade.

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum digest hardcoded no código de produção (apenas vetores conhecidos em teste)
- [ ] `digest_file` usa `io::copy` / buffer de 64 KiB — `fs::read` do arquivo inteiro é reprovação
- [ ] Os 4 testes de mudança material são **4 assertions separadas**, não um teste agregado
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-003` sem findings

## Próxima Task

`task-005`, `task-006` e `task-008` dependem desta.
