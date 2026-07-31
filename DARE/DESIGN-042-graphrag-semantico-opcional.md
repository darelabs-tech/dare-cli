# DESIGN: GraphRAG — semântico opcional (Microplano 042)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (blueprint autorizado via `/dare-blueprint`)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/042-graphrag-semantico-opcional.md`  
> **Referência:** Documento Mestre §5.1 GraphRAG · storage **040** (DEC-036) · ingest/keyword/BFS/RRF **041** (DEC-042) · ADR-006 · baseline TS `@dewtech/dare-cli@3.18.1`  
> **Posição:** 42 de 56  
> **Arquivo:** `DARE/DESIGN-042-graphrag-semantico-opcional.md`  
> **Escopo deste ciclo apenas:** feature Cargo **`semantic`** + `crates/dare-graph/src/semantic.rs` + embeddings locais (**all-MiniLM-L6-v2** quantizado) + cache `~/.dare/models/**` + cosine + fusão RRF vetorial + fallback keyword+grafo + (opcional) `dare graph enable|doctor` + docs + **DEC-045**.  
> **Não** Neo4j / locate / impact / owners / drift (**043**). **Não** obrigar modelo no binário base. DEC proposto: **DEC-045**.

---

## 1. DESCRIÇÃO

Adicionar um **canal semântico opcional** ao GraphRAG já entregue em **040–041**: embeddings locais (sem API cloud) que enriquecem `dare graph query` via ranking vetorial fundido com keyword+BFS pelo mesmo RRF (`k=60`). O binário CLI **default** continua leve — sem modelo embutido; o modelo só existe após opt-in (feature + download confirmado).

O problema: keyword+BFS sozinho falha em queries por intenção (“onde autenticamos JWT?”). Quem usa: engenheiros e agentes que já têm `.dare/graph.db` indexado. Entrega: `semantic.rs`, feature flag, cache em `~/.dare/models`, fusão RRF de 3 listas (keyword, grafo, vetorial) com **fallback automático** se semantic indisponível.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | CLI base sem modelo | Build default **sem** feature `semantic`; binário não inclui pesos ONNX | CI / `cargo tree -p dare-cli` |
| O-02 | Feature `semantic` | `cargo build -p dare-graph --features semantic` compila | Exit 0 |
| O-03 | Modelo MiniLM quantizado | Id congelado **all-MiniLM-L6-v2** (quantizado); dim documentada | Unit + docs |
| O-04 | Download com confirmação | Prompt (ou flag `--yes` em CI) + tamanho bytes exibido **antes** do GET | Smoke |
| O-05 | Cache compartilhado | Artefactos sob `~/.dare/models/**` (ou `%USERPROFILE%\.dare\models` no Windows); reuso entre projetos | Unit + integ |
| O-06 | Cosine O(n·d) | Score = cosine(query, node); complexidade linear no nº de candidatos | Unit + bench leve |
| O-07 | RRF 3 canais | Keyword + BFS + vector fused com `k=60`; tie-break `id ASC` | Golden |
| O-08 | Fallback automático | Sem modelo / download fail / feature off → query = comportamento **041** (keyword+grafo); exit **0** | Smoke |
| O-09 | Enable/doctor (se aprovado) | `dare graph doctor` reporta feature/modelo/cache; enable dispara download | Smoke |
| O-10 | Docs + DEC-045 | `graphrag-semantic.md` + DECISION-LOG; matriz 042 → Concluído | Review |
| O-11 | Ralph | fmt/clippy/test `-p dare-graph -p dare-cli` (com e sem feature) | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Busca por intenção sem inflar install default |
| Tech Lead | DARE CLI Rust | Feature gate; DEC-045; sem ciclo crate; paridade RRF |
| Engenheiro | Consumidor CLI | Opt-in claro; fallback nunca quebra `query` |
| Compat | Baseline TS 3.18.1 | Diffs Classe A/B/C (download UX, lib embeddings) |
| Segurança | — | Path jail em `~/.dare/models`; HTTPS download; sem secrets em logs |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `dare-graph::semantic` | MUST | `crates/dare-graph/src/semantic.rs` + `pub mod` sob `#[cfg(feature = "semantic")]` (API stub/`Unavailable` sem feature) |
| RF-02 | Feature Cargo `semantic` | MUST | Em `dare-graph` (e wire opcional em `dare-cli`); **não** default; CLI base sem modelo |
| RF-03 | Runtime embeddings | MUST | Escolher **uma** lib: 🟡 preferência **`fastembed`** (fallback documentado: `ort` + tokenizers). Blueprint congela crate+versão |
| RF-04 | Modelo | MUST | **all-MiniLM-L6-v2** quantizado; nome canónico + dim (ex. 384) em constante; mismatch → InvalidInput/Internal classificado |
| RF-05 | Download sob confirmação | MUST | Antes do download: mostrar URL (allowlist) + tamanho estimado; exige confirmação interativa **ou** `--yes` / `DARE_GRAPH_SEMANTIC_YES=1` em não-TTY/CI |
| RF-06 | Cache `~/.dare/models` | MUST | Path via home dir + `.dare/models/<model-id>/…`; atomic write; reuso entre projetos; path safety (sem escape) |
| RF-07 | Embed texto | MUST | `embed_query(text) -> Vec<f32>`; `embed_passages` para nós candidatos (label+description e/ou snippet); truncagem com limiar documentado |
| RF-08 | Índice / candidatos | MUST | Semantic rank sobre subconjunto: top-K keyword∪BFS **ou** nós file/code_symbol já ingestados — Blueprint congela estratégia (evitar O(N) full-DB sem cap) |
| RF-09 | Cosine | MUST | Similaridade cosseno; NaN/zero-norm → score 0; ordenação estável |
| RF-10 | Ranking vetorial | MUST | Lista de ids ordenada por score DESC, tie `id ASC` |
| RF-11 | Fusão RRF | MUST | Estender hybrid 041: `rrf_fuse([keyword, graph, vector], k=60)` quando semantic OK; sem vector → 2 listas (paridade 041) |
| RF-12 | Fallback automático | MUST | Feature off / modelo ausente / download cancelado / embed fail → **não** falha a query; warning em stderr/report; ranking = keyword+grafo |
| RF-13 | Persistência embeddings (opcional) | SHOULD | Se armazenar vetores no store: BLOB f32 LE alinhado a 040; invalidar por `contentHash`; ausência de cache em DB = recompute. Blueprint decide in-memory vs persist |
| RF-14 | CLI `dare graph query` | MUST | Com feature: tenta canal semantic; flags possíveis `--no-semantic` (força 041) e/ou `--semantic` (exige modelo, senão warning+fallback ou exit 4 — Blueprint congela) |
| RF-15 | `dare graph doctor` | SHOULD | Report: feature compiled?, model present?, cache path, dim, last error; exit 0 sempre (informativo) |
| RF-16 | `dare graph enable` / download | SHOULD | Dispara download+cache sob confirmação; idempotente se já presente |
| RF-17 | Mensagens en-US | MUST | Help/erros domínio em inglês |
| RF-18 | Docs | MUST | `docs/compatibility/graphrag-semantic.md` |
| RF-19 | DEC-045 | MUST | Append-only DECISION-LOG |
| RF-20 | Matriz 042 | MUST | `000A` → ✅ Concluído no closeout |
| RF-21 | Capability / matrix | SHOULD | Atualizar docs graph capability se existir row; sem nova capability IDE obrigatória |
| RF-22 | Smokes | MUST | query sem semantic = 041; com modelo (fixture mock ou skip CI sem download) fallback; doctor; `--no-semantic` |

### API de domínio (esboço — Blueprint congela)

```rust
#[cfg(feature = "semantic")]
pub struct SemanticOptions {
    pub yes: bool,           // skip interactive confirm
    pub model_id: String,    // default ALL_MINILM_L6_V2_Q
    pub max_candidates: usize,
}

#[cfg(feature = "semantic")]
pub fn ensure_model(opts: &SemanticOptions) -> CoreResult<ModelHandle>;

#[cfg(feature = "semantic")]
pub fn embed_texts(handle: &ModelHandle, texts: &[String]) -> CoreResult<Vec<Vec<f32>>>;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64;

/// Extends 041 hybrid: when semantic available, fuse 3 rankings.
pub fn hybrid_query_v2(store: &mut dyn GraphStore, opts: &SearchOptions) -> CoreResult<Vec<RankedHit>>;
```

### Fora de escopo (ver §10)

- Neo4j, locate, impact, owners, drift (**043**)
- API cloud (OpenAI embeddings, etc.)
- GPU obrigatória / batch server
- Alterar schema storage breaking sem ADR

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | Cosine sobre candidatos capped | `max_candidates` default ≤ **512** (Blueprint) |
| RNF-02 | Performance | Query sem semantic ≤ latência 041 (sem regressão) | Smoke/bench relativo |
| RNF-03 | Disponibilidade | Falha de download **nunca** quebra busca | Exit 0 + warning |
| RNF-04 | Instalação | Binário default sem pesos de modelo | Feature off |
| RNF-05 | Determinismo | Mesmos textos → mesmos embeddings (seed/modelo fixo); RRF tie-break estável | Golden |
| RNF-06 | Observabilidade | Warnings: `semantic unavailable: …` (redacted); doctor JSON opcional | Review |
| RNF-07 | Manutenibilidade | `cfg(feature = "semantic")` isolado; testes 041 passam **sem** feature | CI matrix 2 jobs |
| RNF-08 | Cross-platform | Cache path Windows/macOS/Linux | Unit path join |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar query/text length e `max_candidates` (caps) antes de embed | OWASP A03 |
| RS-02 | Não logar conteúdos de código completos nem PII em erros de embed; `redact` | OWASP A02 / privacy |
| RS-03 | Downloads só de URLs allowlist (host+path do modelo); HTTPS; verificar tamanho/hash se disponível | Supply chain |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL **novo** nas deps `semantic` | OWASP A06 |
| RS-05 | Sem API keys no código; env só para flags (`DARE_GRAPH_SEMANTIC_YES`) | Secrets |
| RS-06 | Path jail: writes só sob `~/.dare/models/**` (e store do projeto já jailado); rejeitar `..` | Path safety |
| RS-07 | Processos filhos (se ort/runtime) via argv separado — sem shell concatenado | RS CLI |
| RS-08 | Confirmação explícita antes de download (tamanho exibido) — sem silent multi‑MB | UX/security |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | workspace `rust-version` |
| Crate | `dare-graph` | + feature `semantic` |
| CLI | `dare-cli` | Wire opcional `--features semantic` |
| Storage | SQLite/JSON | Inalterado (040); embeddings opcionais |
| Search base | keyword + BFS + RRF | 041 |
| Embeddings | 🟡 `fastembed` **ou** `ort`+tokenizers | Blueprint congela |
| Modelo | all-MiniLM-L6-v2 quantizado | Cache `~/.dare/models` |
| Testes | cargo test / tempfile | Unit + smoke |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Model CDN / HuggingFace (ou mirror allowlisted) | Download artefacto | HTTPS GET | Saída | Pesos ONNX/quantizados | Tech Lead (allowlist) |
| Filesystem home | Cache local | FS | RW | `~/.dare/models/**` | CLI |
| Graph store projeto | Já existente | SQLite/JSON | RW | nodes/edges; opcional BLOB embed | dare-graph |

Nenhuma API LLM cloud neste microplano.

---

## 9. RESTRIÇÕES

- **Pré-requisito:** microplano **041** ✅ (já concluído).
- **Binário base** não inclui modelo (critério de aceite microplano).
- **Sem breaking** de contrato `dare graph query` sem feature: comportamento 041 preservado.
- **DEC** apenas **DEC-045** (próximo livre após DEC-044 migrate).
- **Não** Neo4j neste ciclo.
- **Confirmado 🟢:** RRF `k=60` e caps BFS 041 reutilizados.
- **Inferido 🟡:** escolha `fastembed` vs `ort`; persistir embeddings no DB vs só runtime; flags `--semantic` / `--no-semantic` exactas.
- **Lacuna 🔴:** URL oficial + hash checksum do artefacto quantizado — Blueprint deve fixar fonte allowlist.

---

## 10. FORA DO ESCOPO (v1 / este microplano)

- Neo4j, locate, impact, owners, drift, advanced GraphRAG (**043**)
- Embeddings cloud / API keys de vendor
- Treino ou fine-tune de modelo
- UI gráfica / dashboard embeddings
- Mudança de IDs canónicos de nós 040 sem ADR
- Tornar `semantic` default no release estável (fica opt-in)

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Dep `fastembed`/`ort` pesada / CVE | Média | Alto | Feature-gated; audit no job `semantic`; pin versão |
| R-02 | Download falha / rede | Alta | Médio | Fallback 041; doctor explica; retry manual `enable` |
| R-03 | Latência O(N) full embed | Média | Alto | Cap candidatos; opcional cache por contentHash |
| R-04 | Diff vs TS 3.18.1 (lib diferente) | Alta | Baixo | Classe **B**; documentar; golden RRF local |
| R-05 | Disk fill em `~/.dare/models` | Baixa | Médio | Mostrar tamanho; um modelo pinado; docs uninstall path |
| R-06 | Não-determinismo float | Média | Médio | Tolerância em testes de score; ordem por id no tie |
| R-07 | Windows path / home | Média | Médio | Testes path; dirs crate `dirs`/`home` já usado no monorepo |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF priorizados (MUST semantic opt-in + fallback; SHOULD doctor/enable)
- [ ] Feature **não** default confirmada (O-01)
- [ ] Modelo **all-MiniLM-L6-v2** quantizado confirmado
- [ ] 🟡 Runtime (`fastembed` vs `ort`) — PO/TL escolhe no Blueprint
- [ ] 🔴 URL/hash allowlist do artefacto a congelar no Blueprint
- [ ] Segurança RS-01…08 OK (download confirm + path jail + audit)
- [ ] Fora de escopo **043** alinhado
- [ ] DEC-045 reservado; sem colidir com DEC-044
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-042-…`

---

## 13. ESTRATÉGIA DE TESTES (resumo)

| Tipo | Casos |
|------|-------|
| Unit (sempre) | `cosine_similarity`; RRF 2 vs 3 listas; fallback path sem feature |
| Unit (`semantic`) | embed mock/fixture; candidate cap; model path resolve |
| Integração | cache dir tempfile (= home fake); download **mocked** (httpmock / file local) |
| Smoke CLI | `graph query` sem feature ≡ 041; `--no-semantic`; `doctor` |
| Negativo | URL não allowlist; path traversal em model id; texto vazio |
| Compat | Tabela diffs vs TS (Classe A/B/C) em docs |

Ralph: `cargo test -p dare-graph`; `cargo test -p dare-graph --features semantic`; clippy ambos os modos; `cargo audit`.

---

## 14. ENTREGÁVEIS

- `crates/dare-graph/src/semantic.rs` (+ wiring `search` / `lib`)
- Feature `semantic` em `Cargo.toml` (+ opcional `dare-cli`)
- Cache contract `~/.dare/models/**`
- CLI: query estendido + `doctor` / `enable` (SHOULD)
- `docs/compatibility/graphrag-semantic.md` + **DEC-045**
- Matriz 042 Concluído
- Testes + smokes

**Próximo microplano após aceite:** [`043-graphrag-avancado-e-neo4j.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/043-graphrag-avancado-e-neo4j.md)
