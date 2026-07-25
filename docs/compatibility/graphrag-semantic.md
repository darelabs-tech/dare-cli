# GraphRAG semantic (optional) — microplano 042

Optional local MiniLM embeddings for `dare graph query`. Builds on [graphrag-ingest.md](graphrag-ingest.md) (041 keyword+BFS+RRF) and [graphrag-storage.md](graphrag-storage.md) (040). Decision: [DEC-045](../DECISION-LOG.md).

## Feature gate (not default)

Cargo feature **`semantic`** on `dare-graph` (and `dare-cli` → `dare-graph/semantic`) is **off by default**. The release binary does **not** ship model weights. Enable at build time:

```bash
cargo build -p dare-cli --features semantic
cargo test -p dare-graph --features semantic
```

Runtime embeddings use **`fastembed`** (feature-gated). Model id: **`all-MiniLM-L6-v2`**; embedding dim **`384`**.

## Cache

| Path | Role |
|------|------|
| `~/.dare/models/all-minilm-l6-v2` | Shared model cache (`%USERPROFILE%\.dare\models\all-minilm-l6-v2` on Windows) |
| `FASTEMBED_CACHE_PATH` | Set to that dir before fastembed init |

Path jail: model id segment must not contain `..`, `/`, `\`, or be absolute. No writes outside `{home}/.dare/models/**`.

## CLI

```bash
dare graph query <q> [-d DIR] [--limit N] [--max-hops H] [--fanout F] [--no-semantic]
dare graph doctor [-d DIR]
dare graph enable [-d DIR] [--yes]
```

| Surface | Behavior |
|---------|----------|
| `--no-semantic` | Force 041 hybrid (keyword+BFS RRF only); skip vector channel even if feature+model OK |
| `dare graph doctor` | Report: `semanticCompiled`, model id, dim 384, cache path, `modelPresent`, expected ~22 MB, allowlist hosts; exit **0** (informational) |
| `dare graph enable` | Confirm + download/init model into cache; idempotent if already present |
| `DARE_GRAPH_SEMANTIC_YES=1` | Non-TTY / CI: skip interactive confirm (same as `--yes` on `enable`) |

TTY: prompt `y/N` after showing allowlisted hosts + expected size. Without TTY: require `--yes` or `DARE_GRAPH_SEMANTIC_YES=1`. Cancel → typed message, exit **0** on enable.

## Ranking & fallback

| Mode | Channels | Notes |
|------|----------|-------|
| Semantic OK | Keyword + BFS + vector | Cosine on candidates (keyword∪BFS, cap **512**); fuse with RRF `k=60` |
| Unavailable / `--no-semantic` / feature off / download fail / embed fail | Keyword + BFS | Same as microplano **041**; warning `semantic unavailable: …` on soft-fail; query exit **0** |

Embeddings are **runtime-only** — no new graph.db schema / no persisted vectors in this cycle.

## Download allowlist (HTTPS)

Hosts only:

- `huggingface.co`
- `cdn-lfs.huggingface.co`
- `cdn-lfs-us-1.huggingface.co`

## Diffs vs TypeScript 3.18.1

| Item | Class | Note |
|------|-------|------|
| Optional MiniLM + soft-fail to keyword | A | Intent parity with TS optional `@huggingface/transformers` |
| Runtime: **fastembed** vs transformers.js | **B** | Different stack; same model family + dim 384; RRF local golden |
| Feature Cargo vs npm optionalDependency | B | Build-time gate; default CLI binary without weights |
| Cache under `~/.dare/models` | A | Shared across projects |
| Trust download (HTTPS + host allowlist + size UX) | B | No mandatory artefact sha256 if fastembed manages fetch |

## Out of scope

- Neo4j, locate, impact, owners, drift (**043**)
- Cloud embedding APIs
- Making `semantic` default on stable release
