# DESIGN: Patterns — mineração determinística brownfield (Microplano 038)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/038-patterns.md`  
> **Referência:** Documento Mestre §5.6 Patterns · §32 Ciclo 14 · Microplanos **035** (AST) · **037** (DNA) · baseline TS 3.18.1  
> **Posição:** 38 de 56  
> **Arquivo:** `DARE/DESIGN-038-patterns.md`  
> **Escopo deste ciclo apenas:** `dare patterns` + domínio em `dare-project` + capability `dare-patterns` + `DARE/PATTERNS.md` / `patterns-facts.json`. **Não** reverse/dna/migrate/graph CLI lifecycle. Graph indexing **soft**. DEC **041** apenas.

---

## 1. DESCRIÇÃO

Minerar padrões recorrentes em código brownfield por frequência e coocorrência (kinds fechados), materializar `DARE/PATTERNS.md` + `DARE/patterns-facts.json`, com scores estáveis e evidência. Flags: `--check` (zero write), `--modules`, `--inject` (preserva AGENT), `--ast` (opt-in), `-d/--dir`.

Quem consome: agentes IDE (`/dare-patterns`), blueprint trade-offs, steering. Entrega: `crates/dare-project/src/patterns.rs`, CLI `dare patterns`, docs + **DEC-041**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Kinds fechados | Só `inferred-layer`, `naming-idiom`, `structural-idiom`, `call-idiom`, `implicit-decision` | Unit |
| O-02 | Frequência/coocorrência | Score + pares cooccur estáveis (sort determinístico) | Unit |
| O-03 | `PATTERNS.md` + facts JSON | Paths canónicos + schema 1 | Unit + smoke |
| O-04 | `--check` zero writes | Nenhum ficheiro criado/alterado | Smoke |
| O-05 | `--modules` filtra | Só módulos pedidos (ou InvalidInput) | Unit + smoke |
| O-06 | `--inject` preserva AGENT | Corpos entre markers preservados | Unit |
| O-07 | `--ast` opt-in | call-idiom enriquecido via dare-ast; caps | Unit |
| O-08 | Capability `dare-patterns` | `cli_commands: ["patterns"]` | Matrix |
| O-09 | Graph soft-index | Pattern nodes só se store existir; soft-fail | Unit |
| O-10 | Docs + DEC-041 | `cli-patterns.md` + append DEC | Artefatos |
| O-11 | Ralph | fmt/clippy/test `-p dare-project -p dare-cli` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Ciclo 14 Patterns sem bloquear migrate 039 |
| Tech Lead | Não tocar reverse/dna além de enum CLI aditivo; DEC-041 |
| Engenheiro | API `run_patterns` tipada |
| Compat | Diffs Classe A/B vs TS 3.18.1 |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Domínio `dare-project::patterns` | MUST | Módulo + re-exports; sem crate novo |
| RF-02 | Kinds fechados (5) | MUST | Enum/const estável |
| RF-03 | Frequência | MUST | `frequency` ≥ 1; score derivado estável |
| RF-04 | Coocorrência | MUST | Pares `(a,b)` com count; sort lex |
| RF-05 | `DARE/PATTERNS.md` | MUST | Markdown determinístico + markers AGENT |
| RF-06 | `DARE/patterns-facts.json` | MUST | JSON camelCase schema 1 |
| RF-07 | `--check` | MUST | Zero writes; `mode=check` |
| RF-08 | `--modules` CSV | MUST | Filtra módulos; empty/no-match → InvalidInput |
| RF-09 | `--inject` | MUST | Preserve corpos AGENT existentes em PATTERNS.md |
| RF-10 | `--ast` | MUST | Opt-in amostra dare-ast → call-idiom |
| RF-11 | `-d/--dir` | MUST | Start dir (default cwd) |
| RF-12 | CLI `dare patterns` | MUST | Wire `Commands::Patterns` aditivo em main.rs |
| RF-13 | Capability | MUST | `assets/capabilities/dare-patterns` + matrix `cli_commands:["patterns"]` |
| RF-14 | Graph soft | SHOULD | `NodeType::Pattern` via `canonical_pattern_node_id`; soft-fail |
| RF-15 | Path safety | MUST | ProjectRoot / SafeRelativePath / atomic_write |
| RF-16 | Redact | MUST | Evidence/values redigidos |
| RF-17 | Mensagens en-US | MUST | Domínio + help |
| RF-18 | Docs + DEC-041 | MUST | `docs/compatibility/cli-patterns.md` + append |
| RF-19 | Matriz 038 | MUST | Status → Concluído |

### API de domínio (esboço — Blueprint congela)

```text
pub struct PatternsOptions { dir, check, inject, ast, modules: Vec<String> }
pub struct DiscoveredPattern { id, kind, title, frequency, score, evidence, modules }
pub struct Cooccurrence { left, right, count }
pub struct PatternsReport { schema_version, mode, patterns, cooccurrences, written, … }
pub fn run_patterns(opts: &PatternsOptions) -> CoreResult<PatternsReport>;
```

---

## 5. REQUISITOS NÃO FUNCIONAIS

| ID | Categoria | Requisito |
|----|-----------|-----------|
| RNF-01 | Segurança | Path jail; sem shell concat; redact |
| RNF-02 | Performance | Caps walk/AST; score O(n log n) sort |
| RNF-03 | Compat | Diffs vs TS em DEC-041 |
| RNF-04 | Portabilidade | Linux/macOS/Windows |
| RNF-05 | Determinismo | Sort patterns `(kind, id)`; cooccur `(left, right)` |

---

## 6. FORA DE ESCOPO

- Editar `reverse.rs` / `dna.rs` (só CLI enum aditivo)
- `dare migrate` (**039**), graph ingest CLI (**041+**)
- Skill lifecycle / refine
- `--ai` no CLI Rust (enrichment via skill IDE)

---

## 7. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Conflito com dna/reverse em main | Só adicionar variante `Patterns` |
| Graph store ausente | Soft-fail warning |
| Inject apaga enrichment humano | Preservar AGENT bodies |

---

## 8. CRITÉRIOS DE ACEITE (resumo)

- [ ] Resultados explicáveis (kind + evidence + score)
- [ ] Inject preserva conteúdo AGENT existente
- [ ] Ordenação e scores estáveis
- [ ] Ralph `-p dare-project -p dare-cli` + smokes write/check/help
- [ ] DEC-041 + matriz Concluído
