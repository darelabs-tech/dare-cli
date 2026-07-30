# CLI AI enrich (`dare ai`)

> **DEC-051** · Microplano 050 · Library: `crates/dare-ai` · CLI: `commands/ai.rs`

## Purpose

Expose diagnosis and execution of AI enrich providers for DARE markdown (`design` / `blueprint`). Domain logic lives in `dare-ai`; the CLI is a thin clap/I/O shell. Provider ids are **not** the same as `dare-agent` `--driver` ids (DEC-037).

| Surface | Role |
|---------|------|
| `dare ai doctor` | PATH/override probe; never calls `enrich()` |
| `dare ai providers` | Capability list (`schemaVersion` 1) |
| `dare ai run` | Run enrich; default **no-write**; `--write` opt-in |
| `dare ai prompt` | Redacted prompt preview (`envLeaked` must stay false) |

## Providers (frozen order)

| Id | Env override | Notes |
|----|--------------|-------|
| `mock` | — | Always `ready`; in-process; CI smokes |
| `codex` | `DARE_CODEX_COMMAND` | Default when `--provider` omitted |
| `claude-code` | `DARE_CLAUDE_COMMAND` | Text CLI via stdin (SHOULD) |
| `cursor-cli` | `DARE_CURSOR_COMMAND` | Text CLI via stdin (SHOULD) |
| `antigravity-cli` | `DARE_ANTIGRAVITY_COMMAND` | Text CLI via stdin (SHOULD) |

`DoctorStatus`: `ready` \| `missing` \| `invalid` \| `not_implemented`. Doctor resolves the program only (no `--help` spawn).

## Commands (`--command`)

| Command | AGENT section ids |
|---------|-------------------|
| `design` | `description`, `objectives`, `functional-requirements`, `stack` |
| `blueprint` | `architecture-overview`, `execution-phases`, `api-contracts`, `data-model` |

Unknown command → usage exit **2**.

## Flags

```text
dare ai doctor   [--provider <id>] [--json] [-d <dir>]
dare ai providers [--json] [-d <dir>]
dare ai run --command <design|blueprint>
            [--provider <id>] [--facts <rel>] [--markdown <rel>]
            [--write] [--json] [-d <dir>]
dare ai prompt --command <design|blueprint>
            [--provider <id>] [--facts <rel>] [--markdown <rel>]
            [--json] [-d <dir>]
```

| Flag | Default | Effect |
|------|---------|--------|
| `--provider` | `codex` | Provider id (smokes use `mock`) |
| `--facts` | — | Project-relative facts JSON (`title`/`description` required) |
| `--markdown` | — | Project-relative markdown; wins over facts body |
| `--write` | false | Requires `--markdown`; `inject_sections` + `atomic_write`; schema fail → no write |
| `--json` | false | Envelope on stdout |
| `-d` / `--dir` | cwd | `ProjectRoot` |

`--facts` or `--markdown` required for `run`/`prompt` (`MSG_FACTS_REQUIRED`).  
`--write` without `--markdown` → exit **2** (`MSG_WRITE_NEEDS_MARKDOWN`).

## Constants

| Const | Value |
|-------|-------|
| `AI_REPORT_SCHEMA` / `schemaVersion` | **1** (camelCase JSON) |
| `ENRICH_TIMEOUT` | **1200** s |
| `STDOUT_CAP` | 1_048_576 |
| `STDERR_CAP` | 65_536 |
| `DEFAULT_PROVIDER` | `codex` |

## Exit codes

| Code | When |
|------|------|
| 0 | Success (doctor may report `missing`/`invalid` statuses) |
| 1 | Provider exe missing on `run` |
| 2 | Usage (unknown command flag, missing facts/markdown, `--write` without markdown) |
| 3 | Facts/markdown path not found |
| 4 | Invalid input (unknown provider, path jail, malformed enrich JSON/sections) |
| 5 | IO write |
| 124 | Provider timed out |

## Reports

All reports use **schemaVersion 1** (camelCase):

- `DoctorReport` — `ok`, `providers[]` (`id`, `status`, `implemented`, `program`, `envOverride`, `reason`, `defaultTimeoutSecs`)
- `ProvidersReport` — `providers[]` (`id`, `enrich`, `implemented`, `envOverride`, `defaultTimeoutSecs`, `commands`)
- `RunReport` — `ok`, `command`, `provider`, `enriched`, `written`, `writePath`, `sections`, `durationMs`, `warnings`
- `PromptReport` — `command`, `provider`, `promptPreview` (redacted), `promptChars`, `envLeaked`

`promptPreview` never includes env override values / `DARE_*_COMMAND` / `PATH`.

## Capability

`dare-ai` → `cli_commands: ["ai"]` in `assets/capability-matrix.yml` (existing id; no duplicate).

## Examples

```bash
dare ai doctor --json
dare ai doctor --provider mock --json
dare ai providers --json
dare ai prompt --command design --provider mock --markdown DARE/DESIGN.md --json
dare ai run --command design --provider mock --markdown DARE/DESIGN.md --json
dare ai run --command design --provider mock --markdown DARE/DESIGN.md --write --json
```

## Out of scope (050)

Dashboard/REST (**051**), MCP (**052**), rewriting `dare design --ai` / `dare blueprint --ai`, `dare-agent` / execute `--agent`, cloud SDKs.
