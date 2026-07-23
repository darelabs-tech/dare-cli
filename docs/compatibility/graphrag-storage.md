# GraphRAG storage (microplano 040)

Library-first crate `dare-graph`: trait `KnowledgeGraph`, backends SQLite + JSON, IDs canônicos, BLOB f32 LE, migrations explícitas. Complementa [ADR-006](../adr/ADR-006-compatibilidade-migracao-graph-db.md) e [DEC-032](../DECISION-LOG.md).

## Paths

| Path | Backend |
|------|---------|
| `.dare/graph.db` | SQLite (default) |
| `.dare/graph.json` | JSON |
| `dare-graph.yml` | Seleção de backend (`backend: sqlite\|json`) |

Neo4j → `InvalidInput` `"not implemented"` até o microplano **043**.

## Schema SQLite (versão 1)

Idêntico ao baseline `@dewtech/dare-cli@3.18.1`:

- `nodes(id, type, label, description, vector BLOB, metadata, created_at, updated_at)`
- `edges(id, source_id, target_id, type, weight, metadata)`
- Índices: `idx_nodes_type`, `idx_edges_source`, `idx_edges_target`, `idx_edges_type`
- `vector` = sequência de `f32` **little-endian**

Tabela `dare_schema_migrations` só é escrita por `KnowledgeGraph::migrate()` — **nunca** na abertura (ADR-006).

## IDs canônicos

| Tipo | Formato |
|------|--------|
| task | `task:{id}` |
| file | `file:{posixPath}` |
| code_symbol | `code_symbol:{posixPath}::{symbol}` |
| requirement | `requirement:{reqId}` |
| pattern | `pattern:{patternId}` |
| edge | `{kind}:{from}->{to}` |

## Diffs vs TypeScript 3.18.1

| Item | Classe | Nota |
|------|--------|------|
| Trait sync (sem `Promise`) | B | Alinhado a crates 024/030 |
| Persistência SQLite nativa (sem rewrite sql.js) | B | Schema/BLOB/IDs idênticos |
| Sem `ensureVectorColumn` silencioso no open | B | Exige `migrate()` (ADR-006) |
| Storage-only (sem search/BFS/RRF) | — | Escopo 041+ |
| Sem CLI `dare graph` | — | Escopo 041+ |

## API rápida

```rust
use dare_graph::{load_graph_config, open_graph, KnowledgeGraph, GraphNode, NodeType};

let cfg = load_graph_config(&root, None)?;
let mut g = open_graph(&root, &cfg)?;
g.add_node(GraphNode::new("task:t1", NodeType::Task, "T1"))?;
```

## Fora de escopo (040)

- `dare graph ingest|query|stats|viz`
- Keyword / BFS / RRF / hybrid
- Embeddings / semantic feature
- Neo4j
