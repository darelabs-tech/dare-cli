# Capabilities canónicas (ADR-007 / microplano 010)

Decisão: [DEC-011](../DECISION-LOG.md) · [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md)

## Fonte

- `assets/capability-matrix.yml` — **49** capabilities (= Claude commands baseline)
- Tipos em `dare-assets` (`capability.rs`) — extração para `dare-harness` deferida a 011+

## Campos (ADR-007)

`id`, `title`, `description`, `instructions`, `cli_commands`, `outputs` (claude/cursor/codex/antigravity), `assets`

## Trade-offs (T-01…T-12)

| # | Escolha |
|---|--------|
| T-01 | Tipos em `dare-assets` |
| T-02 | MUST 49 Claude |
| T-03/T-04 | Gaps 33/25/48 → `exceptions[]` Classe C |
| T-05 | Id kebab `^[a-z0-9]+(-[a-z0-9]+)*$` |
| T-06 | Output paths via `assert_safe_asset_path` |
| T-09 | Sem gerar `assets/capabilities/**` em massa |
| T-10 | CLI só `dare capabilities validate` |
| T-12 | Skill-pacote ≠ capability IDE |

## Exceptions (Classe C)

| id | Motivo |
|----|--------|
| `cursor-commands-full-parity` | 33 Cursor commands → adapter 012 |
| `cursor-rules-full-parity` | 25 rules não modeladas como rows |
| `agent-skills-full-parity` | 48 package skills ≠ IDE capabilities |

## API

```text
load_capability_matrix_from_str
validate_capability_matrix
render_claude_command / render_agent_skill
```

CLI: `dare capabilities validate` → `capabilities validate: ok (49 entries)`

## Segurança

RS-01…RS-08: validate ids/paths; instructions sem secrets; duplicate path fail; exceptions não desligam validate das entries.

## Ver também

- [`assets-inventory.md`](assets-inventory.md)
- Adapters 011–014
