# DESIGN: DNA — extração de convenções brownfield (Microplano 037)

> **Versão:** v1.0 | **Data:** 2026-07-23 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/037-dna.md`  
> **Referência:** Documento Mestre §5.6 DNA · §32 Ciclo 14 · Microplanos **018** (discover) · **024** (AI foundation) · **035** (AST) · baseline TS 3.18.1  
> **Posição:** 37 de 56  
> **Arquivo:** `DARE/DESIGN-037-dna.md`  
> **Escopo deste ciclo apenas:** `dare dna` + domínio em `dare-project` + capability `dare-dna` + `DARE/PROJECT-DNA.md` / `dna-facts.json`. **Não** reverse (**036**), patterns (**038**), migrate (**039**). Graph indexing **soft** (quando `dare-graph` disponível).

---

## 1. DESCRIÇÃO

Extrair fatos determinísticos de convenção de um projeto brownfield (tooling, naming, arquitetura, testes, libraries, commits Git limitados) e materializar `DARE/PROJECT-DNA.md` + `DARE/dna-facts.json`, com evidência de origem em cada fato.

Quem consome: agentes IDE (`/dare-dna`), steering futuro, feature-design. Entrega: biblioteca em `crates/dare-project/src/dna.rs`, CLI `dare dna`, docs + **DEC-039**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Coletar fatos com evidência | Todo fato tem `evidence[]` não-vazio ou warning explícito | Unit |
| O-02 | `--check` zero writes | Nenhum ficheiro criado/alterado | CLI smoke |
| O-03 | Sem Git continua | Projeto sem `.git` → exit 0; commits vazios | Unit + smoke |
| O-04 | Gerar PROJECT-DNA.md | Path canónico + secções determinísticas + markers AGENT | Unit |
| O-05 | Flags `--check` / `--ast` / `-d` | Help + comportamento | Smoke |
| O-06 | Capability `dare-dna` | `cli_commands: ["dna"]` + README assets | Matrix |
| O-07 | Graph soft-index | Falha graph não falha comando | Unit |
| O-08 | Docs + DEC-039 | `cli-dna.md` + append DEC | Artefatos |
| O-09 | Ralph | fmt / clippy / test verdes | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Ciclo 14 DNA sem bloquear 036 paralelo |
| Tech Lead | Evitar editar reverse; DEC-039 (não 038) |
| Engenheiro | API tipada `run_dna` |
| Compat | Diffs Classe A/B/C vs TS 3.18.1 |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Domínio `dare-project::dna` | MUST | Módulo + re-exports; **não** crate novo |
| RF-02 | Coletar tooling | MUST | Manifests: `package.json`, `Cargo.toml`, `pyproject.toml`, lockfiles, toolchain |
| RF-03 | Analisar naming | MUST | Heurística de filenames sob `src/`/`crates/`/`app/`/`lib/` |
| RF-04 | Analisar arquitetura | MUST | Diretórios-camada conhecidos + opcional AST entities/endpoints |
| RF-05 | Analisar testes | MUST | Dirs `tests/`/`__tests__`/`spec` + deps de teste |
| RF-06 | Libraries | MUST | Top deps de manifests (cap + sort estável) |
| RF-07 | Git log limitado | MUST | `git log` via SafeCommand; cap N commits; soft-fail sem git |
| RF-08 | `DARE/PROJECT-DNA.md` | MUST | Markdown determinístico + markers `<!-- AGENT:… -->` |
| RF-09 | `DARE/dna-facts.json` | MUST | JSON camelCase schema 1 |
| RF-10 | `--check` | MUST | Zero writes; report `mode=check` |
| RF-11 | `--ast` | MUST | Opt-in; usa `dare-ast::analyze_source` em amostra limitada |
| RF-12 | `-d/--dir` | MUST | Start dir (default cwd) |
| RF-13 | CLI `dare dna` | MUST | Wire em `main.rs` (variante Dna apenas) |
| RF-14 | Capability | MUST | `assets/capabilities/dare-dna` + matrix `cli_commands:["dna"]` |
| RF-15 | Graph soft | SHOULD | Index concept nodes; soft-fail se open/migrate falhar |
| RF-16 | Path safety | MUST | Leituras/escritas sob ProjectRoot; sem shell concat |
| RF-17 | Redact | MUST | Secrets/tokens redigidos em evidence/logs |
| RF-18 | Mensagens en-US | MUST | Domínio e CLI help em inglês |
| RF-19 | Docs + DEC-039 | MUST | `docs/compatibility/cli-dna.md` + append DEC |
| RF-20 | Matriz 037 | MUST | Status → Concluído |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### API de domínio (esboço — Blueprint congela)

```text
pub struct DnaOptions { check: bool, ast: bool, dir: PathBuf }
pub struct DnaFact { category, key, value, evidence: Vec<String> }
pub struct DnaReport { schema_version, mode, project_root, git_root, facts, written, … }
pub fn run_dna(opts: &DnaOptions) -> CoreResult<DnaReport>;
```

---

## 5. REQUISITOS NÃO FUNCIONAIS

| ID | Categoria | Requisito |
|----|-----------|-----------|
| RNF-01 | Segurança | Path jail; SafeCommand argv; redact |
| RNF-02 | Performance | Caps: ficheiros lidos, bytes/ficheiro, commits Git, AST sample |
| RNF-03 | Compat | Diffs vs TS classificados em DEC-039 |
| RNF-04 | Portabilidade | Linux/macOS/Windows |
| RNF-05 | Determinismo | Sort facts por (category, key); evidence sorted |

---

## 6. FORA DE ESCOPO

- `dare reverse` / edição de `reverse.rs` (**036**)
- `dare patterns` (**038**)
- `dare migrate` (**039**)
- Enrichment LLM `--ai` completo (hooks 024; enrichment semântico fica na skill IDE)
- GraphRAG search/ingest CLI (**041+**)

---

## 7. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Conflito paralelo com 036 em `main.rs` | Só adicionar variante `Dna`; sem tocar reverse |
| DEC-038 reservado a reverse | Usar **DEC-039** para DNA |
| Graph não migrado | Soft-fail; warning em report |
| Git ausente / lento | Soft-fail; timeout SafeCommand |

---

## 8. CRITÉRIOS DE ACEITE (resumo)

- [ ] Fatos possuem evidência de origem
- [ ] `--check` não escreve
- [ ] Projetos sem Git funcionam
- [ ] `PROJECT-DNA.md` gerado no write mode
- [ ] `cargo fmt --check`, clippy `-p dare-project -p dare-cli`, tests + smokes
- [ ] Matriz 037 → Concluído
- [ ] DEC-039 + `cli-dna.md`

---

## 9. STACK

- Rust workspace; `dare-project` + `dare-cli`
- Deps: `dare-core`, `dare-ast` (opt-in `--ast`), `dare-graph` (soft)
- serde / serde_json

---

## 10. PRÓXIMAS ETAPAS

1. Blueprint `DARE/BLUEPRINT-037-dna.md`
2. Tasks + DAG + EXECUTION-037
3. Implementação + Ralph
