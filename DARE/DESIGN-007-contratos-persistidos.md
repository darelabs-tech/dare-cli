# DESIGN: Contratos persistidos (Microplano 007)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/007-contratos-persistidos.md`  
> **Referência:** Microplanos 002+004+005 · Documento Mestre §13 · ADR-002 flatten · `disk-and-json-policy.md`  
> **Posição:** 7 de 56  
> **Arquivo:** `DARE/DESIGN-007-contratos-persistidos.md` (não substitui Designs 001–006)

---

## 1. DESCRIÇÃO

Este Design cobre os **contratos de disco** do DARE CLI nativo: tipos serde e readers/writers canônicos na crate `dare-contracts`. Sem modelos estáveis para `dare.config.json`, DAG, state, grafo YAML, skills, verification e update manifest, os microplanos 008+ (config/migrations) e comandos de produto não conseguem garantir paridade com o baseline TypeScript 3.18.1 nem round-trip sem perda de chaves desconhecidas.

A entrega são structs tipadas (`DareConfig` com flatten, `DagV21`/`LegacyDag`, `RuntimeStateV1`, `GraphNode`/`GraphEdge`, `SkillsManifest`, `VerificationBaseline`, `UpdateManifestV1`, `TelemetrySnapshot`), fixtures golden, I/O via path safety (005) e writers JSON/YAML determinísticos. Quem usa são engenheiros dos ciclos 008–056; o usuário final ganha persistência compatível — ler legado, regravar sem destruir extensões customizadas.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Desserializar fixtures legadas | Suite carrega fixtures TS/`fixtures/` sem panic | 100% fixtures MUST |
| O-02 | Round-trip unknown keys | Config com chave extra sobrevive serialize→deserialize→serialize | Bit-igual ou Classe A documentada |
| O-03 | DAG v2.1 + legado | Ambos os shapes parseiam; detecção de variante estável | Testes dedicados |
| O-04 | Writers canônicos | JSON keys lexicográficas (ADR-002); YAML estável | Asserts golden |
| O-05 | Paths internos `/` | Campos path em modelos usam POSIX | Asserts |
| O-06 | Erros tipados | Malformed → `CoreError` kind estável (Config/InvalidInput) | Exit 4 alinhado |
| O-07 | Sem I/O fora do jail | Readers/writers usam `ProjectRoot` (005) | 0 PathBuf absoluto soltos na API pública |
| O-08 | Desbloquear 008 | Checklist MUST do 007 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade de disco com CLI npm |
| Tech Lead | Time DARE CLI Rust | Flatten, versão de schema, writers |
| Engenheiro CLI | Time implementação | Tipos reutilizáveis em 008+ |
| Usuário Final | Devs / agentes | Customizações em config não apagadas |
| Compatibilidade | Tech Lead | Matriz Classe A / ADR-002 |
| Segurança | Tech Lead + AppSec | Path jail + sem secrets em fixtures |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `DareConfig` + `#[serde(flatten)]` / mapa de extras | MUST | Round-trip preserva unknown keys na raiz (e nested conforme Blueprint); ADR-002 |
| RF-02 | `DagV21` | MUST | Campos: `title`, `version`, `limits`, `models`, `tasks[]` (id, title, depends_on, complexity, subtask_prompt, spec_file) |
| RF-03 | `LegacyDag` | MUST | Shape flat legado parseável; API de detecção v2.1 vs legado |
| RF-04 | `RuntimeStateV1` | MUST | Modelo de `.dare/state.json` v1 (tasks/attempts/`failureSignature` — campos fechados no Blueprint a partir de fixtures) |
| RF-05 | `GraphNode` / `GraphEdge` | MUST | IDs canônicos (`task:…`, `file:…`, `edge kind:from->to`); tipos para `dare-graph.yml` / JSON — **não** SQLite neste ciclo |
| RF-06 | `SkillsManifest` | MUST | Modelo de `.dare/skills.yml` (header + skills) alinhado a fixtures |
| RF-07 | `VerificationBaseline` | MUST | Modelo de `.dare/verification/<id>.json` (aspectos/proofs — campos Blueprint) |
| RF-08 | `UpdateManifestV1` | MUST | Modelo de `templates/UPDATE-MANIFEST.json` schemaVersion 1 |
| RF-09 | `TelemetrySnapshot` | MUST | Snapshot tipado para dashboard futuro (dag/gates/cost/… — campos mínimos Blueprint) |
| RF-10 | Readers canônicos | MUST | `read_*` / `from_str` por artefato; path via `ProjectRoot` + `SafeRelativePath` |
| RF-11 | Writers canônicos | MUST | JSON: keys lexicográficas (`dare_core::to_canonical_json_string` ou equivalente); YAML: crate pin + ordenação documentada |
| RF-12 | Fixtures + golden | MUST | Pelo menos 1 fixture por artefato MUST; testes round-trip / parse |
| RF-13 | Documentação | MUST | `docs/compatibility/persisted-contracts.md` + índice de tipos ↔ paths |
| RF-14 | DEC no decision log | SHOULD | DEC-008 (serde_yaml pin, flatten strategy, YAML formatting) |
| RF-15 | Paridade golden TS | SHOULD | Divergências classificadas (Classe A/B/C) |
| RF-16 | Migrations de config | COULD | **Fora** — microplano **008** (`dare-config`) |
| RF-17 | SQLite `graph.db` / BLOB f32 | COULD | **Fora** — microplanos **040+** |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contratos de disco (este ciclo)

| Path | Tipo principal |
|------|----------------|
| `dare.config.json` | `DareConfig` |
| `DARE/dare-dag.yaml` (e variantes `dare-dag-*.yaml`) | `DagV21` / `LegacyDag` |
| `.dare/state.json` | `RuntimeStateV1` |
| `dare-graph.yml` | nós/arestas (`GraphNode`/`GraphEdge` + container) |
| `.dare/skills.yml` | `SkillsManifest` |
| `.dare/verification/*.json` | `VerificationBaseline` |
| `templates/UPDATE-MANIFEST.json` | `UpdateManifestV1` |
| (in-memory / futuro `.dare`) | `TelemetrySnapshot` |

Alteração de schema/ID/exit ⇒ ADR + migration note (política `disk-and-json-policy.md`).

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Compatibilidade | Leitura de legado obrigatória | Fixtures 3.18.1 green |
| RNF-02 | Determinismo | Ordenação independente de locale | Mesmo bytes em CI multi-OS (JSON) |
| RNF-03 | Performance | Parse de DAG/config típicos | < 50 ms orientativo em SSD |
| RNF-04 | Observabilidade | Erros de schema via `CoreError` + redact | Sem dump de secrets em fixtures |
| RNF-05 | Manutenibilidade | Módulos por artefato em `dare-contracts` | Clippy limpo; sem `unwrap` em prod |
| RNF-06 | Dependências | `serde`/`serde_json`/`serde_yaml` pins workspace | audit/deny verdes |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths e inputs antes de I/O; rejeitar escape | OWASP A03 |
| RS-02 | Fixtures sem tokens/PII reais; redact em mensagens de erro de parse | OWASP A02 |
| RS-03 | Toda leitura/escrita sob `ProjectRoot` (005) | OWASP A01 |
| RS-04 | `cargo audit` + `cargo deny` após novas deps | OWASP A06 |
| RS-05 | Sem secrets hardcoded; paths via env só se documentado | Supply chain |
| RS-06 | Não executar conteúdo de YAML/JSON como código | Injection |
| RS-07 | Limites de tamanho de ficheiro no reader (cap documentado no Blueprint) | DoS |
| RS-08 | Writers atómicos via `dare_core::fs::atomic_write` quando gravam em disco | Integridade (005) |
| RS-09 | Sem `Command`/shell neste ciclo | Escopo |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | 1.85.0 | pin existente |
| Crate | `dare-contracts` | workspace |
| Erros / JSON canónico | `dare-core` (004) | existente |
| Path / atomic write | `dare-core` path+fs (005) | existente |
| Serde | `serde` + `serde_json` | pins workspace (já há) |
| YAML | `serde_yaml` | **pin no Blueprint** |
| Validação extra | `garde` / `validator` — **A confirmar** | Blueprint (opcional se serde basta) |
| Testes | fixtures em `crates/dare-contracts/tests/fixtures/` | — |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem local | I/O | OS | Entrada+saída | Artefatos de contrato | Time CLI |
| Baseline TS 3.18.1 | Referência | fixtures | Entrada | Golden JSON/YAML | Compat |
| CI 003 | Test | GHA | Entrada | Suite | Time CLI |
| Microplano 008 | Consumidor | API Rust | Entrada | `DareConfig` + loaders | `dare-config` |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** Microplanos **002**, **004** e **005** DONE.
- **Prazo:** Bloqueia **008** (config/migrations) e qualquer comando que persista estado.
- **Limitações:**
  - Não implementar CLI `dare config` / migrations dry-run (008).
  - Não SQLite GraphRAG / `graph.db` (040+).
  - Não alterar schemas públicos sem ADR.
  - Não “corrigir” bugs de formato legado sem classificação (ADR-001).
  - Capabilities IDE / skills registry lifecycle (010, 044+) fora — só modelo de manifest de skills em disco.
- **Idioma:** mensagens en-US; docs pt-BR.
- **Breaking:** mudança de ID canónico ou shape ⇒ ADR + migration.

---

## 10. FORA DO ESCOPO (v1)

- Microplano **008** — precedence CLI/env/config, migration plan, backups de migration.
- Microplanos 016+ — comandos de produto que *consomem* estes tipos.
- Persistência SQLite do grafo e embeddings (040–043).
- Servidor dashboard HTTP (051) — só o tipo `TelemetrySnapshot`.
- Reescrever `UPDATE-MANIFEST` releases 3.9+ (bug legado — 015/021; aqui só ler schema 1).
- Fuzzing exaustivo (SHOULD futuro).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Flatten nested incompleto vs Zod TS | Alta | Alto | Fixtures reais + testes por bloco; DEC |
| R-02 | `serde_yaml` formatação diverge do js-yaml | Alta | Médio | Writer golden; Classificar whitespace se preciso |
| R-03 | Ambiguidades DAG v2.1 vs flat | Média | Alto | Detector explícito + erros claros |
| R-04 | Campos de state.json subdocumentados | Média | Médio | Extrair de fixture TS; campos `Option`/extras |
| R-05 | Ciclo `dare-contracts` ↔ `dare-core` | Baixa | Alto | contracts depende de core; core **não** depende de contracts |
| R-06 | Escopo inchado (todos os manifests) | Média | Médio | RF MUST = tipos+IO+fixtures; deep validation defer 008 |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-17 priorizados (008/040 fora)
- [ ] Estratégia flatten (raiz vs nested) aceite ou marcada para Blueprint
- [ ] Pin `serde_yaml` e política de writer YAML a fechar no Blueprint
- [ ] Cap de tamanho de ficheiro aceite como risco/DoS
- [ ] RS-01…RS-09 validados
- [ ] Pré-requisitos 002+004+005 confirmados
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-007-contratos-persistidos.md`

---

## Apêndice A — Crates / paths (microplano)

| Path | Papel |
|------|-------|
| `crates/dare-contracts/src/` | Tipos + read/write |
| `crates/dare-contracts/tests/fixtures/` | Golden legado |
| `crates/dare-core` | erros, path, fs, JSON canónico |

## Apêndice B — Tipos mínimos (Documento Mestre §13.1)

```rust
DareConfig, DagV21, LegacyDag, RuntimeStateV1,
GraphNode, GraphEdge, SkillsManifest,
VerificationBaseline, UpdateManifestV1, TelemetrySnapshot
```

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `DARE/BLUEPRINT-007-contratos-persistidos.md`.  
3. Após closeout → [`008-configuracao-e-migrations.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/008-configuracao-e-migrations.md).
