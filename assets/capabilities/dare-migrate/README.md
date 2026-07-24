# dare-migrate

CLI command: `migrate`

Harness output paths (from `assets/capability-matrix.yml`):

| Harness     | Path                                        |
|-------------|---------------------------------------------|
| Claude      | `.claude/commands/dare-migrate.md`          |
| Cursor      | `.cursor/commands/dare-migrate.md`          |
| Codex       | `.codex/skills/dare-migrate/SKILL.md`       |
| Antigravity | `.antigravity/commands/dare-migrate.md`     |

## Command

```text
dare migrate --to <stack> [--check] [--ai] [-d DIR]
```

- `--to <stack>` — target stack (required)
- `--check` — analyze and report; zero writes
- `--ai` — optional soft-fail enrichment of AGENT sections
- `-d DIR` — project root (default: cwd)

## Prerequisites

Run `dare reverse` first so `DARE/IDEIA.md` and `DARE/REVERSE/` modules exist.
Optional inputs: `PROJECT-DNA.md` / patterns artifacts (warnings if absent, not hard-fail).

## Artifacts (`DARE/MIGRATION/**`)

| Path | Role |
|------|------|
| `DARE/MIGRATION/MIGRATION.md` | Phased migration plan (+ AGENT markers) |
| `DARE/MIGRATION/migration-facts.json` | Deterministic facts JSON |
| `DARE/MIGRATION/parity/*.feature` | Gherkin parity skeletons per reverse module |

## Safety

**No destructive source rewrite.** The CLI never deletes, moves, or rewrites application source (`src/`, `crates/`, `app/`, etc.). Writes are limited to `DARE/MIGRATION/`.

Static layer: `dare migrate --to <stack>`. Semantic layer: IDE `/dare-migrate` or `dare migrate --ai` (soft-fail).
