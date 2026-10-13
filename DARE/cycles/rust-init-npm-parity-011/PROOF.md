# PROOF — rust-init-npm-parity-011

Executado localmente (Windows 11, toolchain `1.88.0`) na branch `dare/rust-init-npm-parity-011`.

| Gate | Comando | Resultado |
|---|---|---|
| Format | `cargo fmt --all -- --check` | ok |
| Build | `cargo build --workspace` | ok |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| Test | `cargo test --workspace --no-fail-fast` | 968 passed, 0 failed |
| Audit | `cargo audit` | ok (2 allowed warnings pré-existentes; nenhuma dependência nova) |
| Deny | `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |

## Paridade com o npm

`init_npm_parity_tests::init_matches_npm_goldens` — **27/27 casos** iguais ao golden do npm
3.18.1 (árvore de perguntas, config resolvida, árvore de arquivos incl. diretórios vazios,
hash de todos os arquivos, conteúdo dos arquivos gerados por código, argv dos comandos
externos e stdout), com as correções B-01..B-10 aplicadas como patches explícitos.

Caso reportado pelo usuário (`monorepo-node-nestjs-vue-claude-code`, `api-nest-validations`):
`backend/` com o scaffold NestJS completo e `frontend/` com Vite vue-ts — idêntico ao npm.

## Observações
- `serde_json/preserve_order` foi descartado: por unificação de features mudaria `Map`
  no workspace inteiro (e quebrava o clippy `large_enum_variant` em `dare-contracts`).
  A ordem de chaves do npm é reproduzida localmente com `project::ordered_json`.
- O commit `5b46dc0` inclui as correções locais pré-existentes do usuário
  (`cargo fmt` em `e2e_recovery.rs` + `Cargo.lock` com `fs4`) que corrigem o gate de
  formatação quebrado na `main` (run 38325937835).
- CI do PR ainda não executado no momento deste registro.
