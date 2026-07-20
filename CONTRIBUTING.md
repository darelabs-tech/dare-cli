# Contributing to DARE CLI (Rust workspace)

Obrigado por contribuir com o rewrite nativo do DARE CLI.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Tipos comuns

| Tipo | Uso |
|------|-----|
| `feat` | Nova funcionalidade |
| `fix` | Correção de bug |
| `docs` | Documentação |
| `refactor` | Refatoração sem mudança de comportamento |
| `test` | Testes |
| `chore` | Manutenção (deps, CI, tooling) |

### Escopos sugeridos

`cli`, `core`, `contracts`, `config`, `assets`, `workspace`, `ci`, `docs`

### Exemplos

```
feat(cli): add --version flag via clap
fix(core): reject empty names in validate_nonempty_name
chore(workspace): bump clap to 4.5.40
docs(compatibility): document MSRV upgrade path
```

## Toolchain

O projeto usa Rust **1.85.0** via `rust-toolchain.toml`. Com `rustup` instalado, o pin é aplicado automaticamente ao entrar no repositório.

## Gates locais (Ralph Loop)

Antes de abrir PR:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

## Licença

Ao contribuir, você concorda que suas contribuições serão licenciadas sob [Apache-2.0](LICENSE).
