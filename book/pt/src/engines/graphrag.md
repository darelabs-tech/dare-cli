# GraphRAG

O **GraphRAG** é a engine de recuperação de contexto do DARE CLI — um grafo de conhecimento que conecta **requisitos ↔ código ↔ tasks**, implementada nativamente em Rust com **SQLite + FTS5**.

## O que é o GraphRAG?

GraphRAG (Graph Retrieval Augmented Generation) é uma abordagem que, ao invés de busca vetorial pura, usa um grafo de conhecimento estruturado para recuperar contexto relevante de forma determinística.

No DARE CLI, o grafo conecta:

```
[Requisito RF-01] ──implementado_por──► [task-003]
                                             │
                                    toca_arquivo──► [src/auth/jwt.rs]
                                             │
                                    relacionado_a──► [Requisito RF-05]
```

## Camadas de busca

O GraphRAG usa uma estratégia híbrida de 3 camadas:

| Camada | Tecnologia | Velocidade |
|---|---|---|
| **Keyword** | SQLite FTS5 (BM25) | < 1ms |
| **BFS** | Graph traversal | < 5ms |
| **Semântico** (opcional) | Embedding local/remoto | 10–100ms |

Os resultados são ranqueados com **RRF** (Reciprocal Rank Fusion) antes de retornar ao agente.

## Storage

Por padrão, o grafo é armazenado em **SQLite** no diretório `.dare/`:

```
.dare/
├── graph.db          ← grafo principal (SQLite)
├── graph.json        ← export legível (opcional)
└── embeddings.bin    ← vetores f32 LE (modo semântico)
```

> **Contrato de migração**: o formato interno nunca é migrado silenciosamente. Mudanças no schema do `graph.db` exigem migração explícita e estão documentadas em ADR-006.

## Backend alternativo: Neo4j

Para projetos muito grandes, o Neo4j pode ser usado como backend:

```json
// dare.config.json
{
  "graphrag": {
    "backend": "neo4j",
    "neo4j_uri": "bolt://localhost:7687"
  }
}
```

## Comandos

```bash
dare graph query "autenticação JWT"     # busca no grafo
dare graph show task-003                # detalha um nó
dare graph ingest                       # reindexar o projeto
dare graph stats                        # estatísticas do grafo
dare graph export --format json         # exporta para JSON
```

## `--json` para agentes

```bash
dare graph query "JWT" --json
```

```json
{
  "schemaVersion": 1,
  "query": "JWT",
  "results": [
    {
      "node_id": "req:RF-01",
      "type": "requirement",
      "title": "Autenticação via JWT",
      "score": 0.94,
      "related": ["task-003", "src/auth/jwt.rs"]
    }
  ]
}
```

## Economia de tokens com MCP

Quando combinado com o [MCP Server](mcp-server.md), o GraphRAG provê contexto arquitetural sob demanda via Model Context Protocol — economizando até **95% dos tokens** em relação a expor o `BLUEPRINT.md` completo.

## Semântico opcional

O modo semântico usa embeddings para busca por similaridade. Pode ser habilitado com:

```json
// dare.config.json
{
  "graphrag": {
    "semantic": {
      "enabled": true,
      "provider": "openai",   // ou "local" (fastembed)
      "model": "text-embedding-3-small"
    }
  }
}
```

> Com `provider: "local"`, os embeddings rodam offline via `fastembed-rs`, sem API key.
