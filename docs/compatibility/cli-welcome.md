# CLI welcome (`dare welcome`)

> **DEC-017** · Microplano 016 · Source: `crates/dare-cli/src/commands/welcome.rs`

## Purpose

First UX surface of the native binary: optional TTY banner + quick-start for Design → Architecture → Review → Execute. Fixes legacy CI-005 (`dare new` mentioned a nonexistent command).

## Command

```bash
dare welcome [--no-banner]
# global flags also apply:
dare welcome --no-color
dare welcome --json
```

| Flag | Effect |
|------|--------|
| `--no-banner` | Skip banner even on TTY; still prints quick-start |
| `--no-color` | If banner would show, use plain `DARE Framework` instead of ASCII art |
| `--json` | Envelope via output renderer (004); body = welcome text |

## Environment

| Variable | Values that disable banner | Notes |
|----------|----------------------------|-------|
| `DARE_NO_BANNER` | `1`, `true`, `TRUE`, `yes`, `YES` | Other/unset → banner allowed if TTY |
| `NO_COLOR` | any set value | Forces plain banner when banner is shown |

## TTY policy

| Condition | Banner |
|-----------|--------|
| Non-TTY (pipes/CI) | Off |
| `--no-banner` or truthy `DARE_NO_BANNER` | Off |
| TTY + color allowed | ASCII art + tagline |
| TTY + `--no-color` / `NO_COLOR` | `DARE Framework` + tagline |

Quick-start is **always** printed.

## CI-005 — no `dare new`

Welcome output must **never** contain the substring `dare new`. Covered by unit `debug_assert!`, unit tests, and CLI smoke.

## Quick start (canonical steps)

1. `dare design` — `/dare-design` → `DARE/DESIGN.md`
2. `dare blueprint` — `/dare-blueprint` → architecture + tasks
3. `dare tasks` — `/dare-tasks` → `TASKS.md` + `dare-dag.yaml`
4. `dare execute` — `/dare-dag-run-parallel` — Ralph Loop

Also mentioned: `dare info`, `dare harness claude detect`, `dare assets verify`.

## Local verify (container)

```bash
docker compose -f docker-compose.ci.yml config
# optional: docker build -f Dockerfile.rust .
```

Inherits microplan 003/015 images — no new image for welcome.

## Tests

```bash
cargo test -p dare-cli welcome
cargo test -p dare-cli --test cli_smoke -- welcome
```

## Related

- Decision log: **DEC-017**
- Output/errors: [`cli-output-and-errors.md`](cli-output-and-errors.md) (DEC-005)
- Classification: CI-005 Classe B in [`classification-matrix.md`](classification-matrix.md)
