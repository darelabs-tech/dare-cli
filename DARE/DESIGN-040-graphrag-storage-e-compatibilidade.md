# DESIGN: GraphRAG — storage e compatibilidade (Microplano 040)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (ciclo autorizado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/040-graphrag-storage-e-compatibilidade.md`  
> **Referência:** Documento Mestre §5.1 GraphRAG · §33 Ciclo 15 · ADR-006 · contratos 007 · path safety 005 · baseline TS `@dewtech/dare-cli@3.18.1`  
> **Posição:** 40 de 56  
> **Arquivo:** `DARE/DESIGN-040-graphrag-storage-e-compatibilidade.md`  
> **Escopo deste ciclo apenas:** crate **`dare-graph`** com trait `KnowledgeGraph`, backends **SQLite (rusqlite bundled)** + **JSON**, schema nodes/edges, BLOB **f32 LE**, IDs canônicos, migrations **explícitas** versionadas, fixtures/contract tests. **Não** ingest/keyword/BFS/RRF (→ **041**). **Não** semantic (→ **042**). **Não** Neo4j / queries avançadas (→ **043**). **Não** CLI `dare graph *`.

---

## 1. DESCRIÇÃO

Portar a camada de **persistência** do GraphRAG antes da busca híbrida. Projetos brownfield já têm `.dare/graph.db` / `.dare/graph.json`; o Rust MUST abrir e mutar cópias desses stores sem reindexação prévia, preservando schema SQL, encoding de vetores e IDs canônicos idênticos ao legado 3.18.1.

Entrega: library-first em `crates/dare-graph/src/storage/**` + tipos/IDs/migrations; docs de compatibilidade + DEC; atualização da matriz 040 → Concluído. Sem superfície CLI neste ciclo.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-graph` | Member do workspace; deps sem ciclo com `dare-cli` | Compile |
| O-02 | Trait `KnowledgeGraph` | API tipada storage (open/migrate/CRUD/export/vectors/stats/close) | Unit |
| O-03 | Backend SQLite | `rusqlite` **bundled**; paths `.dare/graph.db` | Integração |
| O-04 | Schema nodes/edges | SQL idêntico ao TS 3.18.1 + 4 índices | Fixture |
| O-05 | BLOB f32 LE | Round-trip byte-identical Float32 little-endian | Unit |
| O-06 | Backend JSON | `.dare/graph.json`; contract tests vs SQLite | Integração |
| O-07 | IDs canônicos | `task:`, `file:`, `code_symbol:`, `requirement:`, `pattern:`, edge `{kind}:{from}->{to}` | Unit |
| O-08 | Migrations | Versionadas; **nunca** na abertura (ADR-006) | Unit |
| O-09 | Legacy DB | Abrir+mutar cópia de DB legado; IDs/schema estáveis | Integração |
| O-10 | Docs + DEC + matriz | `graphrag-storage.md` + DEC-036; matriz 040 Concluído | Review |
| O-11 | Ralph | `fmt --check` + `clippy -D warnings` + `test --workspace` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Ciclo 15 storage antes de ingest 041 |
| Tech Lead | Paridade ADR-006; sem CLI prematura |
| Compat | Baseline 3.18.1; diffs classificados |
| Segurança | Path jail; sem secrets em metadata/logs |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-graph` | MUST | Workspace member; `dare-core` + `dare-contracts` + serde/rusqlite |
| RF-02 | Trait `KnowledgeGraph` | MUST | Storage-only sync (Classe B vs `init(): Promise`); search/traverse defer 041 |
| RF-03 | Tipos | MUST | 12 `NodeType`, 13 `EdgeType` = TS `graphrag/types` |
| RF-04 | SQLite backend | MUST | Open/create `.dare/graph.db` sob `ProjectRoot` |
| RF-05 | Schema SQL | MUST | `nodes`/`edges` + 4 índices byte-compatíveis com TS `SCHEMA_SQL` |
| RF-06 | Vector BLOB | MUST | f32 LE; deserialize rejeita length % 4 ≠ 0 |
| RF-07 | JSON backend | MUST | Persist `.dare/graph.json` via `atomic_write` |
| RF-08 | Upsert | MUST | `add_node`/`add_edge` upsert por `id` |
| RF-09 | Delete node | MUST | Remove nó + arestas incidentes |
| RF-10 | IDs canônicos | MUST | Helpers idênticos ao legado |
| RF-11 | Config | MUST | Ler `backend` de `dare-graph.yml`; default sqlite → `.dare/graph.db` |
| RF-12 | Neo4j | MUST | `backend: neo4j` → InvalidInput `"not implemented"` (043) |
| RF-13 | Migrations | MUST | `schema_version` + `migrate()` explícito; open **não** ALTER |
| RF-14 | Detect legacy | MUST | DB sem coluna `vector` → version 0; exige migrate |
| RF-15 | load_vectors | MUST | Rows `{id, v: Vec<f32>}` só nós com vector não-vazio |
| RF-16 | export/import JSON | MUST | `{nodes,edges}` camelCase `sourceId`/`targetId`/`createdAt` |
| RF-17 | Stats | MUST | totals + counts por tipo (zeros para ausentes) |
| RF-18 | Path safety | MUST | Só `SafeRelativePath` / `ProjectRoot` |
| RF-19 | Contract tests | MUST | Mesmo dataset JSON↔SQLite (ids, vectors, edges) |
| RF-20 | Legacy fixture test | MUST | Cópia mutável de DB schema legado |
| RF-21 | Docs | MUST | `docs/compatibility/graphrag-storage.md` + DEC-036 |
| RF-22 | Matriz | MUST | `000A-MATRIZ-DE-STATUS.md` 040 → ✅ Concluído |
| RF-23 | Sem CLI | MUST | Não tocar `dare-cli` `main`/commands graph |
| RF-24 | Mensagens en-US | MUST | Erros de domínio em inglês |

### Contratos de disco

| Path | Papel |
|------|-------|
| `.dare/graph.db` | SQLite store (default) |
| `.dare/graph.json` | JSON store |
| `dare-graph.yml` | Backend selection (não é o store de nós) |

### Fora de escopo

- `dare graph ingest|query|stats|viz` (041+)
- keyword / BFS / RRF / hybrid / embeddings (041–042)
- Neo4j (043)
- Traverse / locate (041/043)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Requisito | Meta |
|----|-----------|------|
| RNF-01 | Compat binária schema+BLOB | Round-trip fixture |
| RNF-02 | Sem migração silenciosa | ADR-006 |
| RNF-03 | Determinismo | Ordenação estável em query/export (id ASC) |
| RNF-04 | Segurança | Jail paths; metadata não loga secrets |
| RNF-05 | Stats zero-filled | Tipos ausentes = 0 (legado RNF-05) |

---

## 6. RESTRIÇÕES E RISCOS

| Risco | Mitigação |
|-------|-----------|
| sql.js rewrite vs rusqlite nativo | DEC Classe B; schema/IDs/BLOB idênticos |
| TS `ensureVectorColumn` silencioso | Rust exige `migrate()` — Classe B / ADR-006 |
| Licença sqlite em `deny.toml` | Ajustar allowlist se necessário (blessing/Unlicense) |
| Merge com 041 | Trait storage estável; search em módulo futuro |

---

## 7. CRITÉRIOS DE ACEITE

- [ ] Rust abre e muta cópia do DB legado
- [ ] IDs permanecem idênticos ao TS
- [ ] JSON e SQLite passam contract tests
- [ ] Migrations explícitas; open sem ALTER
- [ ] Ralph Loop verde
- [ ] Matriz 040 Concluído; DEC + docs

---

## 8. DECISÕES 🟡 (Blueprint congela)

1. Trait **sync** (sem async/Promise).
2. Trait **storage-only** neste ciclo.
3. Persistência SQLite **nativa** (não export/rewrite sql.js).
4. Tabela `dare_schema_migrations` só escrita por `migrate()`.
5. DEC id **DEC-036**.
