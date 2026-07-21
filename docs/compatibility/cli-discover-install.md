# CLI discover install (`dare discover`)

> **DEC-020** · Microplano 019 · Source: `crates/dare-project/src/install.rs` + `commands/discover.rs`

## Purpose

Idempotent brownfield install of DARE methodology artifacts and the four IDE harnesses. Use `--check` (018) first to review detection; then run without `--check` to apply.

## Command / flags

```bash
dare discover --check [-d <path>]
dare discover [-d <path>]
dare discover --force [--dry-run] [--strict-conflicts] [-d <path>]
dare discover --json …
```

| Flag | Effect |
|------|--------|
| `--check` | Detect only — zero writes (018) |
| `-d` / `--dir` | Start directory (default: cwd) |
| `--force` | Overwrite managed config/templates/harness files |
| `--dry-run` | Emit InstallReport without writes |
| `--strict-conflicts` | Abort with exit 4 if stack conflicts present |
| `--json` / `--no-color` | Global (004) |

## Conflicts policy

| Mode | Behavior |
|------|----------|
| Default | Warning + install continues → exit **0** |
| `--strict-conflicts` | No apply → InvalidInput exit **4** |
| `--check` | Report only |

## Exit codes (004)

| Code | When |
|------|------|
| 0 | check Ok or install Ok (warnings allowed) |
| 1 | Internal / severe rollback failure |
| 2 | Usage |
| 3 | `--dir` missing |
| 4 | InvalidInput / path safety / strict conflicts |
| 5 | Io |

> **vs 018 stub:** without `--check` is no longer Internal exit 1; it installs (class B vs prior stub).

## InstallReport schema 1

CamelCase: `schemaVersion`, `mode` (`"install"`), `projectRoot`, `steps[]`, `created`, `updated`, `skipped`, `backedUp`, `harnessesValidated`, `conflicts`, `warnings`, `dryRun`.

Human output ends with: `mode: install`.

## Steps (order)

ensure_dirs → write_config → materialize_templates → write_graph → merge_gitignore → four harness installs → ensure_capability_discover → validate_harnesses.

`.gitignore` uses `# BEGIN DARE` / `# END DARE` with `.dare/` and `.dare/backups/`.

## Preserve / force / rollback

- Existing `dare.config.json` / templates skipped unless `--force` (with backup).
- Existing `dare-graph.yml` never overwritten in MUST.
- On step failure: restore backups and remove session-created files.

## Diff vs TypeScript 3.18.1

| Area | Class |
|------|-------|
| Exit codes 004 | B |
| Conflicts warn+continue | A (aligned) |
| InstallReport schema 1 | C |

## Local verify

```bash
docker compose -f docker-compose.ci.yml config
```

Verified exit 0 (mp019-001).

## DEC-020

See [`docs/DECISION-LOG.md`](../DECISION-LOG.md).
