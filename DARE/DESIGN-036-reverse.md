# DESIGN: Reverse — engenharia reversa brownfield (Microplano 036)

> **Versão:** v1.0 | **Data:** 2026-07-23 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/036-reverse.md`  
> **Referência:** Documento Mestre §31 Ciclo 13 · skill `dare-reverse` · Microplanos **018** (discover) · **024** (dare-ai) · **035** (dare-ast) · path safety **005**  
> **Posição:** 36 de 56  
> **Arquivo:** `DARE/DESIGN-036-reverse.md`  
> **Escopo:** `dare reverse` + módulo `dare-project::reverse` + capability `dare-reverse`. **Não** dna (**037**), patterns (**038**), migrate (**039**).

---

## 1. DESCRIÇÃO

Portar a engenharia reversa Fase 0 (brownfield): varrer o projeto, inventariar módulos, gerar `DARE/IDEIA.md` + specs em `DARE/REVERSE/`, report determinístico, AST opcional (`dare-ast`), Excalidraw opcional, enrichment soft-fail (`dare-ai`), e `--check` zero-write.

O CLI produz a camada **determinística**; a skill `/dare-reverse` preenche marcadores `<!-- AGENT -->`. Entrega verificável sem depender de 037+.

---

## 2. OBJETIVOS E MÉTRICAS

| # | Objetivo | Meta |
|---|----------|------|
| O-01 | Analisar módulos (crates/src/dirs) | Unit + smoke |
| O-02 | Flags `--deep`, `--modules`, `--check`, `--ast`, `--no-excalidraw`, `--report`, `--ai` | CLI smoke |
| O-03 | Gerar `IDEIA.md` + `module-*.md` + `reverse-facts.json` | Snapshot/unit |
| O-04 | `--check` não escreve | Smoke before/after |
| O-05 | AST opcional: merge estável endpoints/entities | Unit |
| O-06 | Excalidraw opcional (default on; `--no-excalidraw` off) | Unit |
| O-07 | Enrichment soft-fail (`--ai`) | Soft warning, exit 0 |
| O-08 | Capability `dare-reverse` + `cli_commands: ["reverse"]` | Matrix + asset README |
| O-09 | Docs + **DEC-038** + matriz 036 Concluído | Artefatos |

---

## 3. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Aceite |
|----|-----------|------------|--------|
| RF-01 | Módulo `crates/dare-project/src/reverse.rs` | MUST | Compile + export |
| RF-02 | CLI `dare reverse` em `commands/reverse.rs` + `main.rs` | MUST | Help + smoke |
| RF-03 | Resolver root via `find_project_root` / `-d` | MUST | Exit 4 se sem root; 3 se dir missing |
| RF-04 | Inventário de módulos determinístico (sort lex) | MUST | Unit |
| RF-05 | `--modules a,b` filtra por id | MUST | Unit |
| RF-06 | `--deep` gera stubs Fase 3 (`erd.md`, `domain-rules.md`, `c4/…`) | MUST | Unit |
| RF-07 | Escrever `DARE/IDEIA.md` com mapa determinístico + markers AGENT | MUST | Conteúdo |
| RF-08 | Escrever `DARE/REVERSE/module-<id>.md` + `reverse-facts.json` | MUST | Paths canónicos |
| RF-09 | `--report` gera `confidence-report.md` | MUST | File + report field |
| RF-10 | `--ast` usa `dare_ast::analyze_source` com caps; merge estável | MUST | Unit |
| RF-11 | Excalidraw default; `--no-excalidraw` omite | MUST | Unit |
| RF-12 | `--check` analisa e reporta **sem** mutações | MUST | Smoke |
| RF-13 | `--ai` / `--provider` soft-fail (não corrompe artefato) | MUST | Soft warning |
| RF-14 | Capability asset + matrix `cli_commands: ["reverse"]` | MUST | Validate |
| RF-15 | Docs `docs/compatibility/cli-reverse.md` + DEC-038 | MUST | Append-only log |
| RF-16 | Mensagens domínio en-US | MUST | Strings |
| RF-17 | Path jail `ProjectRoot` + `atomic_write`; redact secrets | MUST | RS |

### Fora de escopo

- `dare dna` / `patterns` / `migrate`
- GraphRAG ingest via reverse
- Inferência semântica completa no CLI (só esqueletos + facts)

---

## 4. REQUISITOS NÃO FUNCIONAIS

| ID | Requisito |
|----|-----------|
| RNF-01 | Caps: max files AST, max bytes/file, max modules |
| RNF-02 | Skip dirs: `.git`, `node_modules`, `target`, `vendor`, `.dare`, `DARE` (ao varrer código) |
| RNF-03 | Saída JSON schemaVersion 1 camelCase |
| RNF-04 | Diffs vs TS 3.18.1 classificados A/B/C em DEC-038 |
| RNF-05 | Sem shell concatenado |

---

## 5. STACK

- `dare-project` (+ deps `dare-ast`, `dare-core`, serde)
- `dare-cli` (+ `dare-ai` para enrichment soft-fail)
- Sem crate novo (módulo em dare-project)

---

## 6. CRITÉRIOS DE ACEITE

- [ ] `--check` zero writes
- [ ] AST×regex merge estável quando `--ast`
- [ ] `IDEIA.md` gerado no happy path
- [ ] Capability presente
- [ ] Smokes: happy + check + bad input
- [ ] Ralph: fmt, clippy `-p dare-project -p dare-cli`, tests
- [ ] Matriz 036 → Concluído
- [ ] DEC-038

---

## 7. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Escopo TS enorme | MVP Classe B; documentar deferrals |
| Conflito main.rs | Branch isolada; wire fino |
| AST lento em monorepos | Caps + skip dirs |
| Enrichment flaky | Soft-fail (blueprint pattern) |
