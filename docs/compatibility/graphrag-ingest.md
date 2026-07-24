# GraphRAG ingest / keyword / BFS / RRF (microplano 041)

Hybrid search **without** semantic embeddings. Builds on [graphrag-storage.md](graphrag-storage.md) (040) and [ADR-006](../adr/ADR-006-compatibilidade-migracao-graph-db.md). Decision: [DEC-042](../DECISION-LOG.md).

## CLI

```bash
dare graph ingest [-d DIR]
dare graph query <q> [-d DIR] [--limit N] [--max-hops H] [--fanout F]
dare graph stats [-d DIR]
dare graph viz [-d DIR] [-o PATH] [--max-nodes N]
```

Store paths unchanged: `.dare/graph.db` (default) / `.dare/graph.json` via `dare-graph.yml`.

## Ingest

| Step | Behavior |
|------|----------|
| Walk | Source extensions under project; skip `.git`, `node_modules`, `target`, `.dare`, `DARE`, … |
| Hash | `metadata.contentHash` = sha256 hex of file bytes |
| Incremental | Same hash → skip (no rewrite) |
| Symbols | Regex heuristics (`fn`/`function`/`class`/`def`/…); **not** `dare-ast` |
| Edges | `contains`: `file:*` → `code_symbol:*` |
| FTS5 | SQLite best-effort `nodes_fts` rebuild after ingest (acceleration only) |

## Search

| Channel | Detail |
|---------|--------|
| Keyword | Case-insensitive LIKE-style match on `id` / `label` / `description` (SoT for rankings) |
| FTS5 | Optional SQLite index; fallback LIKE |
| BFS | Default **2** hops; clamp `maxHops≤5`, `fanout≤200`; direction Both; stable neighbor order |
| RRF | `score += 1/(60 + rank)` per list; tie-break `id ASC` |
| Hybrid 041 | Keyword ranking + BFS expansion fused by RRF — **no** vector/semantic channel |

## Diffs vs TypeScript 3.18.1

| Item | Class | Note |
|------|-------|------|
| Keyword LIKE SoT | A | Parity |
| FTS5 optional (SQLite) | B | rusqlite bundled; JSON stays LIKE-only |
| No semantic rank | — | Deferred to **042** |
| Regex code-index | A | GraphRAG does not use tree-sitter |
| Sync API | B | Aligns with storage 040 |

## Golden / determinism

Unit tests in `dare-graph` assert RRF math and stable hybrid top-hit for a fixed seed graph. Re-running the same query yields identical id order.

## Out of scope

- Embeddings / feature `semantic` (042)
- Neo4j, locate, impact, owners, drift (043)
- refine / patterns / skills
