# DESIGN: Migrate — plano de migração e Gherkin de paridade (Microplano 039)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (blueprint autorizado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/039-migrate.md`  
> **Referência:** Documento Mestre § Ciclo brownfield Fase 2 · Skills `/dare-migrate` · Microplanos **024** (AI enrich) · **036** (reverse) · **037** (DNA) · **038** (patterns) · baseline TS 3.18.1  
> **Posição:** 39 de 56  
> **Arquivo:** `DARE/DESIGN-039-migrate.md`  
> **Escopo deste ciclo apenas:** CLI **`dare migrate --to <stack>`** + domínio em `dare-project::migrate` + artefatos `DARE/MIGRATION/**` + capability `dare-migrate`. **Não** executa reescrita destrutiva do código. **Não** Neo4j/semantic graph (**042+**). **Não** init/bootstrap (**046–047**). DEC proposto: **DEC-044**.

---

## 1. DESCRIÇÃO

`dare migrate` gera um **plano de migração determinístico** e **esqueletos Gherkin de paridade** para reimplementar um legado brownfield numa **stack alvo** explícita (`--to`). O comando lê evidências já produzidas por `dare reverse` / `dare dna` / (opcional) `dare patterns`, compara stack atual vs alvo, e materializa `DARE/MIGRATION/` — sem apagar, mover ou reescrever o código-fonte do projeto.

O problema: migrar stack sem contrato de paridade vira reescrita cega. Quem consome: engenheiros e agentes IDE (`/dare-migrate` preenche secções AGENT). Entrega: `crates/dare-project/src/migrate.rs`, CLI `dare migrate`, capability, docs + **DEC-044**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Validar stack alvo | `--to` inválido → exit **4** cedo | Unit + smoke |
| O-02 | Comparar origem × alvo | Report com `fromStacks` / `toStack` + gaps | Unit |
| O-03 | Plano por fases | `MIGRATION.md` com fases ordenadas + evidências | Unit + smoke |
| O-04 | Gherkin de paridade | `parity/<module>.feature` esqueleto por módulo reverse | Unit + smoke |
| O-05 | `--check` zero writes | Nenhum ficheiro criado/alterado sob `DARE/` | Smoke |
| O-06 | Sem migração destrutiva | Zero deletes/moves de `src/`/`crates/`/`app/` | Code review + teste |
| O-07 | Enrichment soft | `--ai` soft-fail (padrão blueprint); determinístico sempre ok | Smoke |
| O-08 | Capability `dare-migrate` | Matrix `cli_commands: ["migrate"]` | Matrix validate |
| O-09 | Docs + DEC-044 | `cli-migrate.md` + append DECISION-LOG | Artefatos |
| O-10 | Ralph | fmt/clippy/test `-p dare-project -p dare-cli` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Fechar Ciclo brownfield Fase 2 (migrate) sem cutover real |
| Tech Lead | DARE CLI Rust | DEC-044; não confundir com `dare-config` migrate / graph.migrate |
| Engenheiro | Consumidor CLI | Plano + Gherkin acionáveis; `--check` seguro |
| Agente IDE | `/dare-migrate` | Secções AGENT preenchíveis pós-CLI |
| Compat | Baseline TS 3.18.1 | Diffs Classe A/B/C documentados |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Domínio `dare-project::migrate` | MUST | Módulo + re-exports; sem crate novo |
| RF-02 | CLI `dare migrate --to <stack>` | MUST | Flag `--to` obrigatória; help en-US |
| RF-03 | Allowlist de stacks alvo | MUST | Conjunto fechado alinhado a scaffolds DARE (ex.: `node`, `rust`, `python`, `php-laravel`, `nestjs`, `go-gin`, `rails`, `fastapi`, `rust-axum`, `rust-leptos` — Blueprint congela lista exacta); fora → InvalidInput exit **4** |
| RF-04 | Detectar stack atual | MUST | Reusa `detect_stacks` / evidências de discover; report lista `fromStacks` |
| RF-05 | Comparar origem × alvo | MUST | Diff estrutural: same / upgrade-family / cross-stack; lista `blockingGaps[]` |
| RF-06 | Pré-condição reverse | MUST | Sem `DARE/IDEIA.md` (ou REVERSE vazio) → Usage/InvalidInput com mensagem clara (pedir `dare reverse`); **não** inventar módulos |
| RF-07 | DNA / Patterns opcionais | SHOULD | Se `PROJECT-DNA.md` / `PATTERNS.md` existirem, referenciar no plano; ausência = warning, não hard-fail |
| RF-08 | Plano por fases | MUST | Fases ordenadas (ex.: foundations → modules → cutover); cada fase cita evidência (path/módulo) |
| RF-09 | `DARE/MIGRATION/MIGRATION.md` | MUST | Markdown determinístico + markers `<!-- AGENT:BEGIN/END … -->` nas secções enrichable |
| RF-10 | `DARE/MIGRATION/migration-facts.json` | MUST | JSON camelCase `schemaVersion: 1` |
| RF-11 | Gherkin esqueleto | MUST | `DARE/MIGRATION/parity/<moduleId>.feature` com Scenario placeholders derivados de módulos reverse (não inventar fluxos) |
| RF-12 | `--check` | MUST | Analisa + reporta; **zero writes** |
| RF-13 | `-d/--dir` | MUST | Project root start (default cwd); path safety |
| RF-14 | `--ai` / `--provider` | SHOULD | Soft-fail enrich só secções AGENT de `MIGRATION.md` (padrão DEC-026); sem `--ai` = só determinístico |
| RF-15 | Não destrutivo | MUST | Nunca apaga/reescreve código de aplicação; só escreve sob `DARE/MIGRATION/` |
| RF-16 | Capability | MUST | `assets/capabilities/dare-migrate` + matrix |
| RF-17 | Exit codes | MUST | 0 ok; 2 Usage; 3 NotFound (dir); 4 InvalidInput/Config (stack); 1 Internal |
| RF-18 | Docs + DEC-044 | MUST | `docs/compatibility/cli-migrate.md` + DECISION-LOG |
| RF-19 | Matriz 039 | MUST | Status → Concluído no closeout |
| RF-20 | Smokes | MUST | happy write; `--check` no-write; `--to` inválido → 4; missing reverse → 4/2 |

### API de domínio (esboço — Blueprint congela)

```text
pub struct MigrateOptions {
    pub dir: Option<PathBuf>,
    pub to_stack: String,
    pub check: bool,
    pub ai: bool,
    pub provider: Option<String>,
}
pub struct BlockingGap { pub id: String, pub severity: String, pub evidence: String, pub detail: String }
pub struct MigrationPhase { pub id: String, pub title: String, pub modules: Vec<String>, pub evidence: Vec<String> }
pub struct MigrateReport {
    pub schema_version: u32, // 1
    pub mode: String,        // "write" | "check"
    pub from_stacks: Vec<String>,
    pub to_stack: String,
    pub comparison: String,  // same_family | cross_stack | …
    pub phases: Vec<MigrationPhase>,
    pub blocking_gaps: Vec<BlockingGap>,
    pub written: Vec<String>,
    pub warnings: Vec<String>,
}
pub fn run_migrate(opts: &MigrateOptions) -> CoreResult<MigrateReport>;
```

### Artefatos de disco (contratos)

| Path | Papel |
|------|--------|
| `DARE/MIGRATION/MIGRATION.md` | Plano humano + AGENT enrichable |
| `DARE/MIGRATION/migration-facts.json` | SoT máquina (schema 1) |
| `DARE/MIGRATION/parity/<module>.feature` | Esqueleto Gherkin (skill completa cenários) |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | Path jail `ProjectRoot` / `SafeRelativePath`; `atomic_write` | 100% writes sob jail |
| RNF-02 | Segurança | Redact secrets em facts/evidence/logs | Sem tokens em plaintext |
| RNF-03 | Performance | Caps: máx módulos/fases; leitura reverse/DNA limitada | Completa < 30 s em fixture típica |
| RNF-04 | Determinismo | Ordenação estável de fases, gaps, módulos, written[] | Golden/unit |
| RNF-05 | Portabilidade | Linux / macOS / Windows | Smokes path |
| RNF-06 | Compat | Diffs vs TS 3.18.1 classificados A/B/C em DEC-044 | Doc |
| RNF-07 | Observabilidade | Report human + `--json` envelope ADR-002 | Smoke |
| RNF-08 | Manutenibilidade | Ralph Loop verde no crate tocado | Exit 0 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--to`, paths e inputs antes de I/O | OWASP A03 |
| RS-02 | Não persistir secrets/PII de código legado em facts sem redact | OWASP A02 |
| RS-03 | Escrita só em `DARE/MIGRATION/**` (sem privilege de reescrever app) | OWASP A01 (least privilege) |
| RS-04 | `cargo audit` / deps sem CVE HIGH/CRITICAL no ciclo | OWASP A06 |
| RS-05 | Sem secrets hardcoded; provider/env via `DARE_*` existentes | Supply chain |
| RS-06 | Sem shell concat; AI via providers já existentes (SafeCommand) | RS-06 processo |
| RS-07 | `--check` garante zero mutação (útil em CI) | Integrity |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | workspace MSRV (1.85.0) |
| Domínio | `dare-project` | path crate |
| Path / process | `dare-core` | ProjectRoot, atomic_write, redact |
| Detecção stacks | `dare-project::stacks` / detect | existente |
| Enrich opcional | `dare-ai` | providers mock/codex (024) |
| CLI | `dare-cli` clap | Commands::Migrate aditivo |
| Testes | cargo test + assert_cmd smokes | — |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Provider AI (opcional) | Processo local | argv `DARE_*_COMMAND` | Outbound | Secções AGENT | 024 / DEC-025 |
| Filesystem projeto | Local | FS | R/W | `DARE/**` read; `DARE/MIGRATION/**` write | migrate |
| GraphRAG | — | — | — | **Fora** deste ciclo (soft opcional COULD se trivial) | 041+ |

---

## 9. RESTRIÇÕES

- Pré-requisitos de produto: **024, 036, 037, 038** concluídos (já na main).
- **Não** executar migração de código (sem codegen de stack, sem `git mv` em massa, sem apagar legado).
- Distinguir claramente de:
  - `dare-config` / `plan_migrate` (schema `dare.config.json`) — **008/022**
  - `KnowledgeGraph::migrate()` (schema graph.db) — **040**
- Help e mensagens de domínio em **en-US** (language-policy).
- Lista de stacks `--to` **fechada** (sem free-form arbitrário que quebre scaffolds).

---

## 10. FORA DO ESCOPO (v1 / ciclo 039)

- Executar strangler / cutover / dual-run em produção
- Gerar projeto novo completo (`dare init` / bootstrap → **046–047**)
- Preencher Gherkin semântico rico (isso é `/dare-migrate` skill IDE pós-CLI)
- Neo4j / embeddings / graph query neste comando
- Migrar banco de dados de aplicação do usuário
- OAuth/2FA ou auth de runtime (N/A CLI)
- Alterar `reverse.rs` / `dna.rs` / `patterns.rs` além de leituras

---

## 11. RISCOS E MITIGAÇÕES

| Risco | Prob. | Impacto | Mitigação |
|-------|-------|---------|-----------|
| Confusão com config/graph migrate | Média | Médio | Naming docs + DEC-044; API `run_migrate` em project |
| Stack allowlist desalinhada do TS | Média | Médio | Congelar lista no Blueprint; Classe B se subset |
| Plano sem reverse | Alta | Alto | Hard-fail cedo se IDEIA/REVERSE ausentes |
| `--ai` corrompe plano | Baixa | Alto | Soft-fail; write determinístico primeiro |
| Gherkin inventado no CLI | Média | Alto | Só esqueletos + ids de módulos; skill preenche |
| Merge conflict `main.rs` | Alta | Baixo | Variante `Migrate` aditiva |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo: plano + Gherkin esqueleto **sem** migração destrutiva
- [ ] `--to` allowlist e falha cedo em stack inválida
- [ ] `--check` zero-write
- [ ] Artefatos canónicos `DARE/MIGRATION/**` + schema facts = 1
- [ ] Soft-fail `--ai` opcional; skill IDE `/dare-migrate` documentada como pós-passo
- [ ] Segurança RS-01…RS-07 aceites
- [ ] DEC-044 e docs `cli-migrate.md` previstos
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-039-migrate.md`

---

## Próximas etapas

1. Humano: revisar/aprovar este Design (mudar Status → **APPROVED**).
2. Executar `/dare-blueprint` apontando para `DARE/DESIGN-039-migrate.md`.
3. `/dare-tasks` + execução (DAG) do ciclo 039.
