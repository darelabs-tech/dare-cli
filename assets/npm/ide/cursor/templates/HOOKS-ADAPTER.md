# Hooks adapter — Cursor (v3.6.0)

Hooks **nativos por evento** no Cursor estão **adiados para versão futura** (API de gatilho não confirmada — BLUEPRINT §0).

## Fallback disponível hoje

1. **git pre-commit (universal):** instale `packages/cli/templates/hooks/pre-commit-dare-validate` em `.git/hooks/pre-commit`.
2. **Manual:** `dare hooks run on-save --file <rel>` | `on-file-create` | `on-task-complete` | `pre-commit` (com `--trust` ou `hooks.trusted: true`).

## Steering

Steering files funcionam via MCP: `GET /steering?file=<rel>` ou `dare steering show <file>`.

Não há configuração de gatilho nativo Cursor nesta versão.
