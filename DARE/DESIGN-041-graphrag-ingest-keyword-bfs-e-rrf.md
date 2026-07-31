# DESIGN: GraphRAG — ingest, keyword, BFS e RRF (Microplano 041)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (ciclo autorizado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/041-graphrag-ingest-keyword-bfs-e-rrf.md`  
> **Referência:** Documento Mestre §5.1 GraphRAG · §33 Ciclo 15 · ADR-006 · storage 040 · AST 035 (não usado no code-index) · baseline TS `@dewtech/dare-cli@3.18.1`  
> **Posição:** 41 de 56  
> **Arquivo:** `DARE/DESIGN-041-graphrag-ingest-keyword-bfs-e-rrf.md`  
> **Escopo deste ciclo apenas:** `ingest.rs` + `search.rs` (contentHash, símbolos regex, keyword LIKE/FTS5, BFS 2 hops, RRF k=60, limites maxHops/fanout, golden rankings) + CLI `dare graph ingest|query|stats|viz` + docs + **DEC-042**. **Não** embeddings/semantic (→ **042**). **Não** Neo4j / locate / impact / drift (→ **043**). **Não** refine/patterns/skills.

---

## 1. DESCRIÇÃO

Entregar busca híbrida **básica sem modelo semântico**: indexação incremental de arquivos por `contentHash` (sha256), símbolos por regex, keyword (LIKE com FTS5 opcional no SQLite), expansão BFS (default 2 hops) e fusão RRF (`k=60`). Superfície CLI `dare graph` additive em `dare-cli`.

Pré-requisitos: microplanos **035** (AST nativo — disponível mas **não** usado no code-index GraphRAG) e **040** (storage `dare-graph`).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Ingest incremental | Mesmo contentHash → skip; mudança → reindex | Unit |
| O-02 | Símbolos regex | `code_symbol` + edge `contains` file→symbol | Unit |
| O-03 | Keyword | LIKE (JSON+SQLite); FTS5 opcional SQLite | Unit |
| O-04 | BFS | Default 2 hops; caps maxHops≤5, fanout≤200 | Unit |
| O-05 | RRF | `1/(60+rank)`; ranking determinístico | Golden |
| O-06 | Hybrid query | Keyword + BFS fused by RRF (sem semantic) | Unit |
| O-07 | CLI | `dare graph ingest\|query\|stats\|viz` | Smoke |
| O-08 | Docs + DEC + matriz | `graphrag-ingest.md` + DEC-042; 041 Concluído | Review |
| O-09 | Ralph | fmt + clippy `-D warnings` + test `-p dare-graph -p dare-cli` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Ciclo 15 busca híbrida sem download de modelo |
| Tech Lead | Paridade RRF/BFS; CLI additive |
| Compat | Diffs classificados vs TS 3.18.1 |
| Segurança | Path jail; caps traverse; sem secrets em logs |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `ingest.rs` | MUST | Walk fontes; hash sha256; upsert file nodes |
| RF-02 | contentHash | MUST | Metadata `contentHash`; skip se igual |
| RF-03 | Símbolos regex | MUST | Extrair nomes; IDs `code_symbol:{path}::{sym}` |
| RF-04 | Keyword LIKE | MUST | Match case-insensitive em id/label/description |
| RF-05 | FTS5 | SHOULD | SQLite: tabela FTS5 opcional; fallback LIKE |
| RF-06 | BFS | MUST | Default 2 hops; direção Both; ordem estável |
| RF-07 | Caps | MUST | maxHops clamp ≤5; fanout clamp ≤200 |
| RF-08 | RRF | MUST | k=60; tie-break por id ASC |
| RF-09 | Hybrid | MUST | Sem embeddings; keyword+graph only |
| RF-10 | Golden rankings | MUST | Fixture fixa → ordem de ids assertada |
| RF-11 | CLI ingest | MUST | `dare graph ingest` → abre store, migrate, indexa |
| RF-12 | CLI query | MUST | `dare graph query <q>` → hits RRF |
| RF-13 | CLI stats | MUST | Delega `get_statistics` |
| RF-14 | CLI viz | MUST | Mermaid subset nós/arestas (cap) |
| RF-15 | Additive CLI | MUST | `Commands::Graph` em main.rs; sem remover cmds |
| RF-16 | Path safety | MUST | ProjectRoot / SafeRelativePath |
| RF-17 | Docs | MUST | `docs/compatibility/graphrag-ingest.md` |
| RF-18 | DEC-042 | MUST | Decision log append-only |
| RF-19 | Matriz | MUST | `000A` 041 → ✅ Concluído |
| RF-20 | Mensagens en-US | MUST | Erros/help domínio em inglês |

### Fora de escopo

- Semantic / MiniLM / feature `semantic` (042)
- Neo4j, locate, impact, owners, drift (043)
- Code-index via tree-sitter / dare-ast
- refine / patterns / skills lifecycle

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Requisito | Meta |
|----|-----------|------|
| RNF-01 | Determinismo | Ordenação estável (id ASC / score DESC+id) |
| RNF-02 | Sem modelo | Query funciona offline sem embeddings |
| RNF-03 | Incremental | Reingest sem mudanças = zero writes de conteúdo |
| RNF-04 | Segurança | Skip dirs perigosos; caps; redact |

---

## 6. RESTRIÇÕES E RISCOS

| Risco | Mitigação |
|-------|-----------|
| Walk enorme | Caps MAX_WALK_ENTRIES / MAX_FILE_BYTES |
| FTS5 vs LIKE divergência | LIKE é SoT de ranking; FTS5 só acelera SQLite |
| Regex incompleta vs AST | Documentado; GraphRAG não usa dare-ast (paridade TS) |

---

## 7. CRITÉRIOS DE ACEITE

- [ ] Funciona sem modelo semântico
- [ ] Ranking determinístico (golden)
- [ ] Reindexação sem mudança é incremental
- [ ] `cargo fmt --check`, `clippy -D warnings`, `test -p dare-graph -p dare-cli`
- [ ] Diffs de compat classificados
- [ ] Matriz 041 Concluído + DEC-042
