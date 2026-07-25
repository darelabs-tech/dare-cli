# GraphRAG advanced + Neo4j (experimental) — microplano 043

Advanced GraphRAG queries on the local store, plus an optional **read-only** Neo4j HTTP backend. Builds on [graphrag-ingest.md](graphrag-ingest.md) (041), [graphrag-semantic.md](graphrag-semantic.md) (042), and [graphrag-storage.md](graphrag-storage.md) (040). Decision: [DEC-046](../DECISION-LOG.md).

## CLI

```bash
dare graph locate <query> [-d DIR] [--limit N] [--max-hops H] [--fanout F]
dare graph owners <seed> [-d DIR]
dare graph impact <seeds> [-d DIR] [--limit N] [--max-hops H] [--fanout F]
dare graph trace --from <id> --to <id> [-d DIR] [--max-hops H] [--fanout F] [--limit N]
dare graph drift [-d DIR] [--strict] [--threshold N]
```

Store paths unchanged: `.dare/graph.db` (default) / `.dare/graph.json` via `dare-graph.yml`. Caps align with 041: `maxHops≤5`, `fanout≤200`, `limit` default 20 (max 100 for domain APIs).

| Command | Behavior |
|---------|----------|
| `locate` | Keyword seeds (hop 0, score `1.0`) + BFS neighbors; score = `1.0 * 0.7^hop`; keep **max** score per id; sort score DESC, id ASC |
| `owners` | Metadata `owner` (trimmed string, if present) + source ids of inbound `contains` edges; unique, sort ASC |
| `impact` | BFS **Out** on `depends_on\|uses\|contains\|affects\|implements`; exclude seeds; sort ASC; apply limit |
| `trace` | All shortest paths `from→to` within `max_hops` (unweighted); sort path length ASC, then path ids ASC |
| `drift` | Classify orphan requirements/code + stale; always report (domain `Ok`); CLI may exit **7** with `--strict` |

## Locate decay

Constant **`LOCATE_DECAY = 0.7`**. For hop `h ≥ 1`, candidate score is `1.0 * 0.7.powi(h)`. Seeds stay at `1.0`. Aggregation keeps the maximum score seen per node id.

## Drift buckets + `--strict`

| Bucket | Condition |
|--------|-----------|
| `orphanRequirements` | `node_type == requirement` with **zero** outbound `implements` edges |
| `orphanCode` | `file` or `code_symbol` with **zero** inbound `implements` edges |
| `stale` | `metadata.stale` is JSON `true` **or** string `"true"` (ASCII case-insensitive) |

`violations = len(orphanRequirements) + len(orphanCode) + len(stale)`. Default `--threshold` is **1**.

| Mode | Exit |
|------|------|
| `dare graph drift` (no `--strict`) | **0** — print report even when violations ≥ threshold |
| `dare graph drift --strict` and `violations >= threshold` | **7** — stderr/human includes **`DRIFT_THRESHOLD`** (`DRIFT_THRESHOLD exceeded`) |
| threshold `0` | Exceeds when `violations > 0` (CLI strict helper) |

Other exit codes follow the shared CLI table (usage 2, not found 3, invalid input/config 4, IO 5). Exit **7** is reserved for drift threshold breach under `--strict`.

## Neo4j opt-in (feature `neo4j`)

Cargo feature **`neo4j`** on `dare-graph` (and `dare-cli` → `dare-graph/neo4j`) is **off by default**. Without the feature, `backend: neo4j` → `InvalidInput` `"neo4j backend requires the neo4j feature"`.

```bash
cargo build -p dare-cli --features neo4j
cargo test -p dare-graph --features neo4j
```

| Aspect | Detail |
|--------|--------|
| Transport | HTTP `POST {url}/db/{database}/tx/commit` (Neo4j 5 transactional HTTP) via workspace **`ureq`**; Basic auth |
| KG surface | **Read-only** subset: `get_node`, `query_nodes`, `get_edges` (+ stats/migrate no-ops as documented in code); mutations → `InvalidInput` `"neo4j writes not supported in 043"` |
| Timeout / retry | timeout **5 s**; retries **2** on timeout/5xx; backoff **`100ms * attempt`** |
| URL allowlist | scheme `http` \| `https` only; non-empty host |

### Config + env

```yaml
# dare-graph.yml (feature neo4j required to open)
backend: neo4j
neo4j:
  url: http://localhost:7474
  database: neo4j
  # user/password: prefer env
```

| Env | Role |
|-----|------|
| `NEO4J_URL` | Base URL (required via yaml or env) |
| `NEO4J_USER` | Basic auth user (default `neo4j` when unset in resolver) |
| `NEO4J_PASSWORD` | Basic auth password — **never** logged or shown in `Debug`/`Display` |
| `NEO4J_DATABASE` | Database name (default `neo4j`) |

## Diffs vs TypeScript 3.18.1

| Item | Class | Note |
|------|-------|------|
| locate / owners / impact / trace / drift | A | Intent parity with advanced GraphRAG surfaces |
| Locate decay **0.7** | A | Deterministic hop attenuation |
| Drift `--strict` → exit **7** + `DRIFT_THRESHOLD` | A | Aligns with master exit table |
| Stale via `metadata.stale` | B | Local rule when no TS golden in-repo |
| Neo4j HTTP + Cargo feature (not Bolt driver) | B | Experimental opt-in; read-only in 043 |
| Neo4j not in default binary | B | Same pattern as `semantic` (042) |

## Out of scope

- Making `neo4j` default on stable release
- Neo4j writes / Bolt driver
- Dashboard / MCP graph UIs
- `execute --policy decay` (agent decay ≠ locate decay)
