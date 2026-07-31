# DESIGN: GraphRAG — avançado + Neo4j experimental (Microplano 043)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (blueprint autorizado via `/dare-blueprint`)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/043-graphrag-avancado-e-neo4j.md`  
> **Referência:** Documento Mestre §2.2 exit **7** · §34 Ciclo 16 · storage **040** (DEC-036) · ingest/search **041** (DEC-042) · semantic **042** (DEC-045) · ADR-006 · baseline TS `@dewtech/dare-cli@3.18.1`  
> **Posição:** 43 de 56  
> **Arquivo:** `DARE/DESIGN-043-graphrag-avancado-e-neo4j.md`  
> **Escopo deste ciclo apenas:** `advanced.rs` (locate/owners/impact/trace/drift) + CLI subcomandos + exit **7** em `drift --strict` + `neo4j.rs` HTTP **opt-in experimental** + timeouts/retries + docs + **DEC-046**.  
> **Não** skills/init/bootstrap (**044+** já parcialmente feitos fora desta sequência). **Não** tornar Neo4j default. DEC proposto: **DEC-046**.

---

## 1. DESCRIÇÃO

Completar o GraphRAG além de ingest/query/viz/semantic: comandos de **navegação e higiene do grafo** (`locate`, `owners`, `impact`, `trace`, `drift`) sobre o store SQLite/JSON já existente, com limites de traverse e **exit code 7** quando drift em modo strict ultrapassa threshold. Em paralelo, desbloquear o backend **Neo4j** hoje rejeitado em `config.rs` via cliente HTTP experimental (feature/opt-in), com timeout e retries.

O problema: sem locate/impact/drift, o grafo não responde a perguntas de ownership, blast-radius e inconsistências requisito↔código. Quem usa: engenheiros e o orquestrador (`execute --next` já menciona graph-locate opcional). Entrega: `crates/dare-graph/src/advanced.rs`, `neo4j.rs`, wire CLI `dare graph …`, docs + DEC-046.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | `dare graph locate` | Seeds + decay → ranking determinístico de nós | Unit + smoke |
| O-02 | `dare graph owners` | Lista owners/módulos por nó ou path | Unit + smoke |
| O-03 | `dare graph impact` | Blast-radius / dependentes a partir de seed; respeita caps hops/fanout | Unit + smoke |
| O-04 | `dare graph trace` | Caminho/trace entre seeds (ou até tipo); ordenação estável | Unit + smoke |
| O-05 | `dare graph drift` | Detecta orphan-requirement, orphan-code, stale | Unit + golden |
| O-06 | Drift strict | `--strict` + threshold estourado → exit **7** | Smoke |
| O-07 | Traverse caps | maxHops/fanout/timeout respeitados (sem hang) | Unit |
| O-08 | Neo4j opt-in | Feature/backend experimental; default build **sem** exigir Neo4j up | Smoke config |
| O-09 | HTTP timeouts/retries | Cliente Neo4j com timeout + retry limitado documentado | Unit (mock HTTP) |
| O-10 | Docs + DEC-046 | `graphrag-advanced.md` (+ neo4j) + DECISION-LOG; matriz 043 Concluído | Review |
| O-11 | Ralph | fmt/clippy/test `-p dare-graph -p dare-cli` (± feature neo4j) | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Fechar Ciclo 16 GraphRAG avançado |
| Tech Lead | DARE CLI Rust | Exit 7; Neo4j experimental; ADR-006; DEC-046 |
| Engenheiro | Consumidor CLI | locate/impact/drift acionáveis no SQLite default |
| Compat | Baseline TS 3.18.1 | Diffs Classe A/B/C (Neo4j HTTP vs driver) |
| Segurança | — | Sem secrets Neo4j em logs; path jail; caps traverse |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `dare-graph::advanced` | MUST | `crates/dare-graph/src/advanced.rs` + exports |
| RF-02 | `locate` com decay | MUST | Input: query/seeds; score com decay por hop (fórmula Blueprint); output ranked ids |
| RF-03 | `owners` | MUST | Resolve ownership a partir de edges/metadata (ex. contains/module); lista determinística |
| RF-04 | `impact` | MUST | A partir de seed(s), lista nós impactados (BFS/DFS capped); direction In/Out/Both configurável |
| RF-05 | `trace` | MUST | Caminho(s) entre origem e destino (ou até tipo); se múltiplos, ordenar por comprimento depois id |
| RF-06 | `drift` orphan/stale | MUST | Tipos mínimos: `orphan-requirement`, `orphan-code`, `stale` (alinhar TS/Mestre); report JSON/human |
| RF-07 | Threshold + exit 7 | MUST | `drift --strict` (e/ou `--threshold N`): se violações ≥ threshold → process exit **7**; sem `--strict` → exit 0 + report |
| RF-08 | Caps traverse | MUST | Reusar/estender caps 041: maxHops ≤5 (ou const avançada documentada), fanout ≤200; timeout wall opcional |
| RF-09 | CLI subcomandos | MUST | `dare graph locate\|owners\|impact\|trace\|drift` additive em `Commands::Graph` |
| RF-10 | Flags comuns | MUST | `-d/--dir`; `--json`; `--limit`; hops/fanout onde aplicável; drift: `--strict`, `--threshold` |
| RF-11 | Mensagens en-US | MUST | Help/erros domínio em inglês |
| RF-12 | Neo4j HTTP experimental | MUST | `neo4j.rs`; backend `neo4j` deixa de ser hard-reject **quando** feature/opt-in ativo |
| RF-13 | Neo4j opt-in | MUST | Feature Cargo `neo4j` **não** default **ou** config + env; sem servidor Neo4j, SQLite/JSON intactos |
| RF-14 | Auth Neo4j | MUST | URL + user/pass via env (`NEO4J_URL` / `NEO4J_USER` / `NEO4J_PASSWORD` ou bloco yaml) — **nunca** logar password |
| RF-15 | Timeout/retries | MUST | Timeout HTTP default (ex. 5s) + retries (ex. 2) com backoff; configuráveis; sem loop infinito |
| RF-16 | KnowledgeGraph Neo4j | SHOULD | Implementar subset read/query necessário aos advanced cmds; writes podem ser limitadas (documentar) |
| RF-17 | Docs | MUST | `docs/compatibility/graphrag-advanced.md` (+ secção Neo4j ou ficheiro irmão) |
| RF-18 | DEC-046 | MUST | Append-only DECISION-LOG |
| RF-19 | Matriz 043 | MUST | `000A` → ✅ Concluído |
| RF-20 | Smokes | MUST | locate/owners/impact/trace/drift happy; drift strict → 7; neo4j sem feature → 4 ou mensagem clara |
| RF-21 | Compat ADR | SHOULD | Diffs vs TS classificados; apontar ADR-006 se schema/migrate |

### API de domínio (esboço — Blueprint congela)

```rust
pub struct LocateOptions { /* seeds/query, max_hops, fanout, decay */ }
pub struct DriftOptions { pub strict: bool, pub threshold: u32, /* … */ }
pub struct DriftReport {
    pub orphans_requirement: Vec<String>,
    pub orphans_code: Vec<String>,
    pub stale: Vec<String>,
    pub violations: u32,
    pub threshold: u32,
}

pub fn locate(g: &dyn KnowledgeGraph, opts: &LocateOptions) -> CoreResult<Vec<RankedHit>>;
pub fn owners(g: &dyn KnowledgeGraph, seed: &str) -> CoreResult<Vec<String>>;
pub fn impact(g: &dyn KnowledgeGraph, seeds: &[String], opts: &TraverseOptions) -> CoreResult<Vec<String>>;
pub fn trace(g: &dyn KnowledgeGraph, from: &str, to: &str, opts: &TraverseOptions) -> CoreResult<Vec<Vec<String>>>;
pub fn drift(g: &dyn KnowledgeGraph, opts: &DriftOptions) -> CoreResult<DriftReport>;

/// CLI maps: if opts.strict && report.violations >= threshold → exit 7
```

### Fora de escopo (ver §10)

- Semantic model download (já 042)
- Dashboard MCP graph endpoints (ciclo separado)
- Neo4j como default de produção
- Correção de skills/init (outros microplanos)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | Traverse capped; sem full-graph unbounded | caps 041 + timeout |
| RNF-02 | Determinismo | Rankings/paths estáveis (id ASC / length ASC) | Golden |
| RNF-03 | Disponibilidade | SQLite/JSON funcionam sem Neo4j | Smoke default |
| RNF-04 | Observabilidade | Drift report conta violations; Neo4j errors redacted | Review |
| RNF-05 | Manutenibilidade | `advanced` independente de `semantic`; feature `neo4j` isolada | CI dual |
| RNF-06 | Compat | Exit **7** preservado (Mestre §2.2) | Smoke |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar seeds/paths/query length e caps hops/fanout/threshold | OWASP A03 |
| RS-02 | Redact passwords/tokens Neo4j e PII em erros/logs | OWASP A02 |
| RS-03 | Sem executar Cypher arbitrário do utilizador sem allowlist/param binding | Injection |
| RS-04 | `cargo audit` sem HIGH/CRITICAL novo (deps HTTP) | OWASP A06 |
| RS-05 | Credenciais Neo4j só via env/yaml local gitignored — nunca hardcoded | Secrets |
| RS-06 | Path jail no project graph; Neo4j URL scheme allowlist `http`/`https` | Path/SSRF |
| RS-07 | Timeouts obrigatórios em HTTP (sem hang) | Availability |
| RS-08 | Retries com teto; sem amplify DoS contra Neo4j | Abuse |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | workspace |
| Crate | `dare-graph` | + `advanced.rs` + `neo4j.rs` |
| CLI | `dare-cli` | `graph` subcommands additive |
| Store default | SQLite / JSON | 040 |
| Search/BFS | `search.rs` | Reusar `bfs_expand` / caps |
| Neo4j | HTTP (🟡 `ureq` workspace) | Feature `neo4j` opcional |
| Testes | tempfile + httpmock/wiremock 🟡 | Blueprint congela mock |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Neo4j Server | Graph DB experimental | HTTP/Bolt-via-HTTP 🟡 | Saída | Cypher parametrizado / JSON | Tech Lead |
| Filesystem `.dare/graph.*` | Store local | FS | RW | nodes/edges | dare-graph |

Sem cloud LLM neste microplano.

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** 040–042 ✅ (matriz).
- **Exit 7** reservado a drift strict (não reutilizar para outros erros).
- **Neo4j** experimental: pode ser incomplete vs SQLite (documentar subset).
- **DEC** apenas **DEC-046** (próximo após DEC-045).
- **Confirmado 🟢:** `GraphBackend::Neo4j` já existe e é rejeitado até 043; exit 7 no Mestre.
- **Inferido 🟡:** fórmula exacta de decay do locate; mapping owners a partir de quais edge types; Cypher vs HTTP transactional endpoint.
- **Lacuna 🔴:** golden fixtures TS exactos para drift types — Blueprint deve fixar enums e exemplos mínimos se TS não estiver no repo.

---

## 10. FORA DO ESCOPO (v1 / este microplano)

- Tornar Neo4j backend default
- UI dashboard / MCP `/graph/*` HTTP server
- Semantic ingest persistido em DB (042 já decidiu runtime-only)
- `dare execute --policy decay` (política de agente — outro ciclo)
- Microplanos 044+ (skills já entregues noutro DEC)

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Paridade TS incompleta em drift/locate | Alta | Médio | Classe B + goldens mínimos locais |
| R-02 | Neo4j flaky em CI | Alta | Médio | Feature off default; mocks HTTP; `#[ignore]` integrações reais |
| R-03 | Traverse explode em grafos grandes | Média | Alto | Caps + timeout |
| R-04 | Leak de password Neo4j | Baixa | Alto | redact + env-only |
| R-05 | Exit 7 confundido com erro genérico | Média | Médio | Docs + smoke dedicado; stderr mensagem `DRIFT_THRESHOLD` |
| R-06 | Cypher injection | Média | Alto | Só queries internas parametrizadas |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF MUST (advanced cmds + exit 7 + Neo4j opt-in) OK
- [ ] Fórmula decay / enums drift a congelar no Blueprint
- [ ] Neo4j feature não-default confirmada
- [ ] Segurança RS-01…08 OK
- [ ] Fora de escopo alinhado (sem dashboard/MCP)
- [ ] DEC-046 reservado
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-043-…`

---

## 13. ESTRATÉGIA DE TESTES (resumo)

| Tipo | Casos |
|------|-------|
| Unit | decay scores; impact caps; drift classification; exit mapping helper |
| Golden | fixture graph → locate/impact/drift ids estáveis |
| Smoke CLI | cada subcomando; `drift --strict` → 7 |
| Neo4j | config reject sem feature; mock HTTP timeout/retry |
| Negativo | seed inexistente; threshold 0; URL neo4j inválida |

Ralph: `cargo test -p dare-graph`; `cargo test -p dare-cli --test cli_smoke -- graph_`; clippy; audit.

---

## 14. ENTREGÁVEIS

- `crates/dare-graph/src/advanced.rs`
- `crates/dare-graph/src/neo4j.rs` (+ feature/opt-in)
- CLI: `locate|owners|impact|trace|drift`
- Exit **7** em drift strict
- `docs/compatibility/graphrag-advanced.md` + **DEC-046**
- Matriz 043 Concluído
- Testes + smokes

**Nota:** o microplano aponta “próximo = 044 skills”, mas skills registry/lifecycle **já estão concluídos** na matriz — após 043, seguir a matriz (ex. próximos pendentes reais).
