# REVIEW — rust-init-npm-parity-011

## Escopo entregue
- Menus do `dare init` idênticos ao npm 3.18.1 (estrutura, MVC, MCP, backend, frontend,
  workspace Rust, IDE, GraphRAG, MCP Server, toolchain) e flags não interativas iguais.
- Scaffold igual ao npm para todas as stacks e estruturas da matriz, usando os templates
  do pacote publicado; Vite/Rails/Leptos via toolchain auto/native/docker.
- `dare bootstrap` compatível com o npm (configs novas) e com a 4.0 (configs legadas).

## Desvios intencionais
B-01..B-10 (ver `DESIGN.md` e `docs/compatibility/cli-init-bootstrap.md`), versão do
binário em `dare.config.json`, ausência de Dockerfile/compose no init (igual ao npm),
extras Rust (`--dir`, `--force`, `--check`, `--json`, aliases).

## Breaking vs Rust 4.0
`--fullstack <react|vue>` e `--toolchain none` no init foram removidos (mensagem orienta
o modo interativo / Monorepo). Configs 4.0 (`stack`) continuam suportadas pelo bootstrap.

## Itens para acompanhamento (fora do escopo)
- Monorepo NestJS: `.claude/settings.json` libera `Read(src/**)` embora o código fique em
  `backend/` (comportamento do npm, mantido; candidato a correção futura nos dois CLIs).
- Portar as correções B-xx para o npm em PR separado.

Aprovação humana obrigatória antes do merge na `main`.
