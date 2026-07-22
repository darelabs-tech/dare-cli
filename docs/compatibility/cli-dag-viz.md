# CLI dag viz (`dare dag viz`)

> **DEC-028** · Microplano 027 · Source: `crates/dare-dag/src/viz.rs` + `crates/dare-cli/src/commands/dag.rs`

## Purpose

Deterministic visualization of `dare-dag.yaml` as Mermaid, Graphviz DOT, or Excalidraw JSON. Library-first render in `dare-dag::viz`; thin nested CLI. Does **not** mutate the DAG or runtime state unless `-o` writes an output file.

## Command

```bash
dare dag viz [--dag PATH] [-f mermaid|dot|excalidraw] [-o PATH]
# global flags:
dare dag viz --json --no-color
```

| Flag | Default | Effect |
|------|---------|--------|
| `--dag` | `DARE/dare-dag.yaml` | DAG path (project-relative or absolute under root) |
| `-f` / `--format` | `mermaid` | Exact lowercase only: `mermaid` \| `dot` \| `excalidraw` |
| `-o` / `--output` | (stdout) | Write body under project jail via `atomic_write` |

## Formats

| Format | Shape |
|--------|-------|
| Mermaid | `flowchart TB` + `subgraph rank_N["Rank N"]`; edges `dep --> task` after subgraphs |
| DOT | `digraph dare_dag { rankdir=TB; … }`; edges `dep -> task` |
| Excalidraw | `{type,version:2,source:"dare-cli",elements,appState}`; columns by rank |

Node order: rank ascending, then id lexicographic. Edge order: `(from, to)` lexicographic. Title truncate: 40 Unicode scalars + `…`. Ids sanitized (`-`→`_`) for Mermaid/DOT identifiers.

## Status / complexity (Excalidraw)

- Complexity fills: LOW `#e3f2fd`, MED `#fff3e0`, HIGH `#fce4ec`, other `#eeeeee`
- Optional `.dare/state.json`: soft-fail if missing/corrupt → all PENDING strokes
- Status strokes: PENDING `#9e9e9e`; RUNNING `#1976d2` dashed; DONE `#2e7d32`; FAILED `#c62828`; SKIPPED `#757575` dashed

## Exit codes

| Code | When |
|------|------|
| 0 | OK (stdout or file) |
| 2 | Usage (invalid `-f`, clap) |
| 3 | DAG NotFound |
| 4 | InvalidInput / Config / cycle / missing dep / jail / OUTPUT_CAP / no root |
| 5 | Io |

Cycle errors include the substring `cycle` (en-US) and never dump `subtask_prompt`.

## Caps / safety

| Control | Value |
|---------|-------|
| `OUTPUT_CAP` | 2_097_152 bytes |
| Jail | `--dag` / `-o` must resolve under project root |
| Zero writes | Without `-o`, no DAG/state mutation |
| RS-02 | Prompts never appear in viz output |

## `--json` data

```json
{
  "format": "mermaid",
  "dag": "DARE/dare-dag.yaml",
  "outputPath": null,
  "body": "flowchart TB\n…"
}
```

When `-o` is set, `body` is `null` and `outputPath` is the relative path written.

## Out of scope

- `dare execute` / Ralph orchestration (028+)
- PNG/SVG export, force-directed layout
- GraphRAG visualization

## Container

```bash
docker compose -f docker-compose.ci.yml config
```

Verified exit 0 in mp027-001 — no waiver; no new image for viz.

## Local verify

```bash
dare dag viz --dag DARE/dare-dag-027.yaml
dare dag viz -f dot -o DARE/dag-graph-027.dot
cargo test -p dare-dag -- viz
cargo test -p dare-cli --test cli_smoke -- dag_viz
```

## Related

- Decision log: **DEC-028**
- Runtime ranks/state: [`dag-runtime.md`](dag-runtime.md) (DEC-027)
- Validate: [`cli-validate.md`](cli-validate.md) (DEC-021)
- Capability matrix: `dare-dag-viz` → `cli_commands: ["dag"]` (subcomando `viz`)
