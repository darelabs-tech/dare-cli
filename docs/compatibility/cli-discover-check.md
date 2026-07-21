# CLI discover --check (`dare discover --check`)

> **DEC-019** · Microplano 018 · Source: `crates/dare-project` + `crates/dare-cli/src/commands/discover.rs`

## Purpose

Read-only brownfield detection: project root, Git root, stacks (node/rust/python), conflicts, monorepo heuristics, and IDE harness presence. **Zero filesystem mutations.** Installation without `--check` is deferred to microplano **019**.

## Command / flags

```bash
dare discover --check [--dir|-d <path>]
dare discover --check --json [--dir|-d <path>]
dare discover [-d <path>]   # without --check → exit 1 (install not implemented)
```

| Flag | Effect |
|------|--------|
| `--check` | Detect only — do not install DARE files |
| `-d` / `--dir <path>` | Start directory (default: cwd) |
| `--json` | Envelope via output renderer (004); `data` = DetectionReport schema 1 |
| `--no-color` | Global; no ANSI in human mode |

## Exit codes (aligned to microplano 004)

| Code | Kind | When |
|------|------|------|
| 0 | — | `--check` succeeded |
| 1 | Internal | `discover` **without** `--check` (install → 019) |
| 2 | Usage | invalid args / clap |
| 3 | NotFound | `--dir` missing / not a directory |
| 4 | InvalidInput | path safety reject |
| 5 | Io | unexpected I/O while reading the tree |

> **Note vs Design 018 Apêndice D:** the Design draft used 3=path, 4=I/O, 5=install. Blueprint 018 **corrects** to the frozen 004 map (install stub = Internal/1, NotFound=3, Io=5). Classification: intentional Class B/C drift — see DEC-019.

## JSON schema version 1 (frozen)

CamelCase fields:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `mode` | string | Always `"check"` in this microplan |
| `projectRoot` | string\|null | Absolute display path |
| `gitRoot` | string\|null | Absolute display path |
| `stacks` | `StackHit[]` | Sorted by `id`; families `node`\|`rust`\|`python` |
| `conflicts` | `StackConflict[]` | One entry if ≥2 families; else `[]` |
| `monorepo` | bool | |
| `monorepoEvidence` | string[] | Relative POSIX paths; sorted |
| `harnesses` | `HarnessHit[]` | Always 4 ids sorted: antigravity, claude, codex, cursor |
| `dareAlreadyPresent` | bool | `dare.config.json` **or** `DARE/` at root |

Example (Node fixture, paths illustrative):

```json
{
  "schemaVersion": 1,
  "mode": "check",
  "projectRoot": "/tmp/existing-node-project",
  "gitRoot": null,
  "stacks": [
    { "id": "node", "family": "node", "confidence": "high", "evidence": ["package.json"] }
  ],
  "conflicts": [],
  "monorepo": false,
  "monorepoEvidence": [],
  "harnesses": [
    { "id": "antigravity", "present": false, "evidence": [] },
    { "id": "claude", "present": false, "evidence": [] },
    { "id": "codex", "present": false, "evidence": [] },
    { "id": "cursor", "present": false, "evidence": [] }
  ],
  "dareAlreadyPresent": false
}
```

## Markers / stacks / monorepo / conflicts

**Project root walk-up** stops at first ancestor with any of: `dare.config.json`, `DARE/`, `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements.txt`, `setup.py`. `.git` alone is not a project-root marker.

| Family | Root markers | Confidence |
|--------|--------------|------------|
| node | `package.json` (+ lockfiles in evidence) | high |
| rust | `Cargo.toml` | high |
| python | `pyproject.toml` \| `requirements.txt` \| `setup.py` | high if pyproject; else medium |

**Conflicts:** ≥2 distinct families at root → one `StackConflict` with sorted `kinds` and evidence. Check still exits **0**.

**Monorepo:** true if `pnpm-workspace.yaml` \| `lerna.json` \| `nx.json`, or Cargo `[workspace]`, or ≥2 child manifests within depth ≤3 (max 64 dirs; skip `node_modules`/`target`/`.git`/`vendor`/`.dare`).

Manifest reads capped at **262_144** bytes (`MANIFEST_READ_CAP`).

## Harness mapping

| id | `present` if | Evidence candidates |
|----|--------------|---------------------|
| claude | `claude_md \|\| claude_dir` | `CLAUDE.md`, `.claude` |
| cursor | `cursor_dir \|\| cursorrules` | `.cursor`, `.cursorrules` |
| codex | `agents_md \|\| codex_dir \|\| agents_skills` | `AGENTS.md`, `.codex`, `.agents/skills` |
| antigravity | rules/dir/skills/workflows | `.antigravityrules`, `.antigravity`, `.agents/skills`, `.agents/workflows` |

## Zero mutations

`--check` must not create, modify, or delete files. Human output ends with exact line:

```text
mode: check (zero mutations)
```

Without `--check`, the CLI returns Internal (exit 1) **without** calling `detect` and without writing.

## Diff vs TypeScript `@dewtech/dare-cli@3.18.1`

| Area | TS 3.18.1 | Native 018 | Class |
|------|-----------|------------|-------|
| `--check` surface | Combined discover/install | Check-only; install = 019 | B |
| Exit codes | Historical TS map | Frozen 004 (`CoreError`) | B |
| Schema | Ad-hoc / evolving | `schemaVersion: 1` camelCase | C |
| Stacks MUST | Broader heuristics | node/rust/python only | C |

## Local verify (container)

```bash
docker compose -f docker-compose.ci.yml config
```

Verified exit 0 in microplano 018 (mp018-001). Inherits images from microplanos 003/015 (`Dockerfile.rust`).

## DEC-019

See [`docs/DECISION-LOG.md`](../DECISION-LOG.md) entry **DEC-019**.
