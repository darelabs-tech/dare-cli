# DESIGN: Engine AST nativo (Microplano 035)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/035-engine-ast-nativo.md`  
> **Referência:** Documento Mestre §5.3 AST · §31 Ciclo 13 · Microplanos **005** (path safety) · **009** (assets) · baseline TS 3.18.1  
> **Posição:** 35 de 56  
> **Arquivo:** `DARE/DESIGN-035-engine-ast-nativo.md`  
> **Escopo deste ciclo apenas:** crate **`dare-ast`** — tree-sitter nativo + fallback regex + dedupe + corpus. **Não** `dare reverse` / `dna` / `patterns` / `migrate` (→ **036+**). **Não** GraphRAG code-index (→ **040+**). **Não** tocar `crates/dare-cli/src/main.rs` (biblioteca pura).

---

## 1. DESCRIÇÃO

Este Design inicia o Ciclo 13 (brownfield) com a **engine AST** reutilizável: extrair endpoints HTTP e entidades (classes/models) de código-fonte via tree-sitter nativo, com fallback regex transparente e merge determinístico.

O problema: sem `dare-ast`, `reverse`/`dna`/`patterns` (036–038) não têm motor de extração. A baseline TS usa web-tree-sitter + WASM opcional; em Rust ganhamos performance com gramáticas nativas. Quem consome: crates futuros e testes de corpus. Entrega: `crates/dare-ast` + fixtures + docs + **DEC-032**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-ast` | Member no workspace; deps sem `dare-cli` | Compile |
| O-02 | 8 linguagens | TS/TSX/JS/Python/PHP/Go/Ruby/Rust com grammar feature | Unit + fixtures |
| O-03 | Extrair endpoints | Método HTTP + path por linguagem | Corpus golden |
| O-04 | Extrair entities | Classes/models/structs/interfaces | Corpus golden |
| O-05 | Fallback regex | Sem grammar / feature off → regex ainda extrai | Unit |
| O-06 | Dedupe AST×regex | Merge estável; sem duplicatas | Unit |
| O-07 | Output determinístico | Ordenação canónica; JSON estável se serializado | Golden |
| O-08 | Feature flags | Flags por linguagem; default = all | Compile matrix |
| O-09 | Docs + DEC | `ast-engine.md` + DEC-032; Ralph Loop | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Ciclo 13: motor AST sem CLI reverse ainda |
| Tech Lead | Evitar ciclos crate; não puxar 036+ |
| Engenheiro | API tipada `analyze_*` |
| Compat | Diffs Classe A/B/C vs TS 3.18.1 documentados |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-ast` | MUST | Workspace member; **não** depende de `dare-cli` |
| RF-02 | Gramáticas | MUST | TS, TSX, JS, Python, PHP, Go, Ruby, Rust via crates tree-sitter |
| RF-03 | Feature flags | MUST | `lang-*` por linguagem; default habilita todas; build sem default usa só regex |
| RF-04 | Detect language | MUST | Por extensão de path (`.ts`/`.tsx`/`.js`/`.py`/`.php`/`.go`/`.rb`/`.rs`) |
| RF-05 | Parse | MUST | `parse(source, lang)` → tree ou erro tipado; parse fail → fallback regex |
| RF-06 | Extract endpoints | MUST | `HttpEndpoint { method, path, line, source: Ast\|Regex }` |
| RF-07 | Extract entities | MUST | `Entity { name, kind, line, source }` kinds: class/struct/interface/model/enum |
| RF-08 | DataModel | MUST | `DataModel { language, endpoints, entities, warnings }` |
| RF-09 | Regex fallback | MUST | Padrões por linguagem; sempre disponível |
| RF-10 | Merge/dedupe | MUST | Key endpoint=`METHOD\0path`; entity=`kind\0name`; prefer AST sobre Regex |
| RF-11 | API pública | MUST | `analyze_source(path, source)`; `analyze_bytes`; ordenação determinística |
| RF-12 | Corpus | MUST | ≥1 fixture por linguagem (+ tsx); testes golden |
| RF-13 | Limites | MUST | Cap tamanho fonte (ex. 2 MiB); rejeitar NUL |
| RF-14 | Path safety | SHOULD | Se API file-based: jail via `dare-core` ProjectRoot |
| RF-15 | Docs + DEC | MUST | `docs/compatibility/ast-engine.md` + DEC-032 |
| RF-16 | Sem CLI | MUST | Sem mudanças obrigatórias em `dare-cli` / `main.rs` |
| RF-17 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-18 | Determinismo | MUST | Sort endpoints por (method, path, line); entities por (kind, name, line) |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### API de domínio (esboço — Blueprint congela)

```text
// crates/dare-ast
pub enum Language { TypeScript, Tsx, JavaScript, Python, Php, Go, Ruby, Rust }
pub fn detect_language(path: &str) -> Option<Language>;
pub fn analyze_source(path: &str, source: &str) -> CoreResult<DataModel>;
pub struct DataModel { /* endpoints, entities, language, warnings */ }
```

---

## 5. REQUISITOS NÃO FUNCIONAIS

| ID | Categoria | Requisito |
|----|-----------|-----------|
| RNF-01 | Performance | Parse single-file síncrono; sem threads obrigatórias |
| RNF-02 | Segurança | Sem shell; sem eval de fonte; redact paths em logs |
| RNF-03 | Compat | Diffs vs TS classificados em DEC-032 |
| RNF-04 | Portabilidade | Compila Linux/macOS/Windows (MSVC/clang para grammars C) |
| RNF-05 | Build | Feature flags permitem CI parcial se grammars falharem |

---

## 6. FORA DE ESCOPO

- `dare reverse`, `dare dna`, `dare patterns`, `dare migrate`
- GraphRAG indexing via AST
- CLI subcommand dedicado
- Incremental parse / watch
- WASM tree-sitter

---

## 7. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Versões tree-sitter incompatíveis entre grammars | Pin único `tree-sitter` 0.25.x + grammars compatíveis; feature flags |
| Build C no Windows | Documentar; CI matrix já tem windows-latest |
| Extração incompleta vs TS | Classe B em DEC; corpus mínimo MUST; regex cobre gaps |

---

## 8. CRITÉRIOS DE ACEITE (resumo)

- [ ] Cada linguagem possui fixture
- [ ] Fallback funciona sem grammar
- [ ] Output determinístico
- [ ] `cargo fmt --check && cargo clippy --workspace --all-features -- -D warnings && cargo test --workspace`
- [ ] DEC-032 + docs compat
- [ ] Matriz 035 → Concluído
