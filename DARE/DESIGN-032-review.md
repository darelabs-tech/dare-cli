# DESIGN: Review — análise estática anti-stub (Microplano 032)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (ciclo autorizado sem pausa humana)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/032-review.md`  
> **Referência:** Documento Mestre §28 Ciclo 10 · skill `/dare-review` · TASK-SPEC Anti-Stub · microplanos **024** (`dare-ai`) · **025** (blueprint) · **029** (Ralph / DEC-030) · baseline TS 3.18.1  
> **Posição:** 32 de 56  
> **Arquivo:** `DARE/DESIGN-032-review.md`  
> **Escopo deste ciclo:** `dare review <task-id>` — scan estático determinístico (stubs/mocks/TODOs), severidades, flags, formatos human/json/github, merge `--from-agent`, enrichment opcional (Classe B se stub), capability `dare-review`. **Não** refine/sub-DAG (**033**). **Não** guard (**034**). **Não** mutation/formal/best-of-N (**049**).

---

## 1. DESCRIÇÃO

Portar o gate de qualidade que impede marcar task DONE com stubs, mocks de produção ou TODOs. A camada estática (regex/padrões) vive no binário nativo; a camada semântica continua no agente IDE via `/dare-review` + `--from-agent`.

Quem consome: Definition of Done das specs; CI; `dare execute --complete` (hook futuro `review.onComplete`); agentes. Entrega: crate **`dare-review`** + `crates/dare-cli/src/commands/review.rs` + docs + **DEC-034**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-review` | Workspace member; deps `dare-core` (+ serde); sem ciclo com `dare-cli` | Compile |
| O-02 | Detectar TODO/FIXME/XXX/HACK | Achado `todo_marker` severidade error | Unit |
| O-03 | Detectar stubs / unimplemented | `unimplemented!`, `todo!`, placeholders | Unit |
| O-04 | Detectar mocks fora de testes | `jest.fn` / `sinon.stub` / etc. só fora de test paths | Unit |
| O-05 | Severidades | `error` \| `warning` tipados | Unit |
| O-06 | `--strict` | Warnings elevam falha (`ok=false`) | Unit + smoke |
| O-07 | `--errors-only` | Output só errors (cálculo fail-on inalterado) | Unit |
| O-08 | `--files` | Override lista de ficheiros (jail) | Unit + smoke |
| O-09 | `--from-agent` | Merge semantic JSON; unmet → fail | Unit |
| O-10 | Formatos | human / json / github válidos | Unit + golden |
| O-11 | `--comment` | Bloco markdown resumo (PR comment body) | Unit |
| O-12 | `--fail-on` | Exit 1 conforme threshold | Unit + smoke |
| O-13 | Determinismo | Mesma input → mesma ordem de findings | Unit |
| O-14 | Docs + DEC + matriz | `cli-review.md` + DEC-034 + status Concluído | Exit 0 |
| O-15 | Ralph | fmt + clippy -D warnings + test workspace | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Gate anti-stub no ciclo Execute |
| Tech Lead | Escopo estático; enrich Classe B se preciso |
| Engenheiro CLI | `dare-review` + wire `main.rs` |
| CI / agentes | Exit codes + GitHub annotations |
| Compat | Diffs classificados vs TS 3.18.1 |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-review` | MUST | Member workspace; **não** depende de `dare-cli` / `dare-dag` |
| RF-02 | API `run_review(root, opts) -> ReviewReport` | MUST | Report tipado camelCase serde |
| RF-03 | Resolver ficheiros da spec | MUST | Parse secção 3 de `DARE/EXECUTION/<id>.md` (paths em backticks na tabela) |
| RF-04 | `--files` override | MUST | Substitui lista da spec; paths jail `ProjectRoot` |
| RF-05 | Padrões anti-stub | MUST | Ver §4.1; regras aplicadas linha a linha |
| RF-06 | Mocks só fora de testes | MUST | Test paths: `*.test.*`, `*.spec.*`, `__tests__/`, `/tests/`, `/spec/`, `*_test.rs`, `tests.rs` |
| RF-07 | Severidade | MUST | `error` (markers/stubs/mocks prod); `warning` (placeholder soft) |
| RF-08 | `--strict` | MUST | `ok = errors==0 && (!strict \|\| warnings==0)` antes de fail-on semantic |
| RF-09 | `--errors-only` | MUST | Filtra emissão; não apaga warnings do contador interno para `--fail-on warning` |
| RF-10 | Formato `human` | MUST | Default; lista file:line rule message |
| RF-11 | Formato `json` | MUST | `ReviewReport` schema 1 + envelope CLI `--json` |
| RF-12 | Formato `github` | MUST | Linhas `::error` / `::warning` GitHub Actions |
| RF-13 | `--from-agent PATH` | MUST | JSON `{passed, unmetCriteria[], notes?}`; unmet → findings `semantic` error |
| RF-14 | `--comment` | MUST | Acrescenta secção markdown `## DARE review` no human (e campo `commentMarkdown` no JSON) |
| RF-15 | `--fail-on` | MUST | Valores exact: `error` (default), `warning`, `never` |
| RF-16 | Exit codes | MUST | 0 pass; 1 fail-on; 2 usage; 3 spec/task not found; 4 invalid; 5 io |
| RF-17 | Enrichment `--ai` | SHOULD | Soft-fail Classe B: warning estável + `enriched=false`; scan estático sempre corre |
| RF-18 | `--provider` | SHOULD | Só com `--ai`; sem provider real neste ciclo (soft stub) |
| RF-19 | Capability | MUST | `cli_commands: ["review"]` + README `assets/capabilities/dare-review` |
| RF-20 | Docs | MUST | `docs/compatibility/cli-review.md` + DEC-034 |
| RF-21 | Mensagens en-US | MUST | Domínio em inglês |
| RF-22 | Path safety | MUST | Spec path + files sob root; id path-safe |
| RF-23 | Cap leitura | MUST | `read_limited` / limite bytes por ficheiro (007) |
| RF-24 | Skip binários / não-texto | SHOULD | Extensões allowlist de texto; outros ignorados sem error |
| RF-25 | Spec ausente | MUST | NotFound exit **3** |
| RF-26 | Task id path-unsafe | MUST | InvalidInput **4** |
| RF-27 | Ordenação | MUST | findings sort: path, line, col, ruleId |
| RF-28 | Smoke CLI | MUST | clean pass; TODO fail; github format; fail-on never |

### 4.1 Regras estáticas (congelar no Blueprint)

| ruleId | Severity | Padrão (resumo) |
|--------|----------|-----------------|
| `todo_marker` | error | `\b(TODO\|FIXME\|XXX\|HACK)\b` |
| `unimplemented_macro` | error | `unimplemented!` / `todo!` |
| `stub_comment` | error | `// stub`, `# stub`, `implement later`, `not implemented` em comentário |
| `placeholder_soft` | warning | `coming soon`, `placeholder` (fora de strings de teste) |
| `mock_outside_test` | error | `jest.fn(`, `sinon.stub(`, `mockReturnValue`, `mockResolvedValue`, `vi.fn(` — só se **não** test path |
| `empty_ok_stub` | warning | corpo `{ Ok(()) }` / `{ Ok(None) }` / `{ None }` em linha única (heurística) |

### Superfície CLI

```text
dare review <task-id>
  [--strict] [--errors-only] [--files PATH...]
  [--from-agent PATH] [--format human|json|github]
  [--comment] [--fail-on error|warning|never]
  [--ai] [--provider ID]
  # globais --json / --no-color
```

> Nota: `--format json` vs global `--json`: Blueprint congela (preferir `--format` para body do review; global `--json` envelope ADR-002).

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Tipo | Requisito |
|----|------|-----------|
| RNF-01 | Perf | Scan ≤5s para ≤50 ficheiros ≤200KB cada em SSD típico |
| RNF-02 | Det | Mesma árvore → mesmo JSON (sem timestamps no report; `schemaVersion` fixo) |
| RNF-03 | Sec | Sem shell; sem secrets em findings (redact se path contém `.env`) |
| RNF-04 | UX | Exit 1 = “não pode DONE”; mensagem clara |

---

## 6. RESTRIÇÕES E FORA DE ESCOPO

- **Não** `dare refine` / spliceSubDag (**033**)
- **Não** verdito semântico LLM nativo (só merge `--from-agent` + soft stub `--ai`)
- **Não** integrar `review.onComplete` em execute neste ciclo (doc aponta futuro)
- **Não** AST/tree-sitter — só line scan
- **Não** alterar contrato state.json

---

## 7. STACK TÉCNICA

| Camada | Escolha |
|--------|---------|
| Linguagem | Rust 1.85 / edition 2021 |
| Crate | `dare-review` |
| CLI | clap 4.5.40 em `dare-cli` |
| JSON | serde / serde_json camelCase |
| Path | `dare-core` ProjectRoot / SafeRelativePath |
| AI | `dare-ai` só se enrich real; senão Classe B stub |
| Testes | unit `dare-review` + smoke `dare-cli` |

---

## 8. SEGURANÇA (OWASP)

- A01: path jail em todos os reads
- A03: validar task-id + fail-on enum
- A06: sem nova dep com CVE HIGH (preferir std + workspace)
- A09: sem log de conteúdo completo de ficheiros

---

## 9. CRITÉRIOS DE ACEITE (microplano)

- [ ] Exit codes correspondem ao `--fail-on`
- [ ] Formato GitHub válido (`::error` / `::warning`)
- [ ] Resultados estáticos determinísticos
- [ ] `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`
- [ ] Diffs vs TS classificados (DEC-034)
- [ ] Matriz 032 → Concluído

---

## 10. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Falso positivo em comentários legítimos | Allowlist rule + `--fail-on`; docs |
| Merge conflict `main.rs` | Variante `Review` isolada; reportar no retorno |
| Enrichment complexo | Classe B soft stub documentado |

---

## 11. APROVAÇÃO

Ciclo Design→Blueprint→Tasks→Execute **autorizado** pelo utilizador sem pausa humana (chat microplano 032).
