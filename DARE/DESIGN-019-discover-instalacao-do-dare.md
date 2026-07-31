# DESIGN: Discover — instalação do DARE (Microplano 019)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/019-discover-instalacao-do-dare.md`  
> **Referência:** Microplanos **005** (path/fs/backup) · **007** (contratos) · **008** (config) · **009** (assets) · **010** (capability matrix) · **011–014** (install/validate harness) · **018** (`detect` / `DetectionReport`) · **004** (saída/`--json`) · baseline TS 3.18.1  
> **Posição:** 19 de 56  
> **Arquivo:** `DARE/DESIGN-019-discover-instalacao-do-dare.md` (não substitui Designs 001–018)  
> **Nota:** Este Design cobre a **instalação idempotente** de `dare discover` (sem `--check`). A detecção read-only permanece em **018** (`--check`). `dare init` / bootstrap greenfield ficam em ciclos posteriores (**046–047**).

---

## 1. DESCRIÇÃO

Este Design cobre a transformação do `DetectionReport` (018) em uma **instalação brownfield idempotente** do DARE CLI nativo: planear e aplicar `dare.config.json`, árvore `DARE/` + `.dare/`, templates canónicos, `dare-graph.yml`, merge de `.gitignore`, e adapters dos quatro harnesses (Claude, Cursor, Codex, Antigravity), com **rollback** se a aplicação falhar a meio.

O problema: após `--check`, developers e agentes precisam materializar a metodologia no projeto existente sem duplicar ficheiros, sem corromper customizações unmanaged, e com saída clara (human/`--json`). Quem consome são developers, agentes IDE (`/dare-discover`) e o Ciclo 1 alpha (`welcome` + `info` + `discover`).

A entrega principal vive em `crates/dare-project/src/install.rs` (`InstallPlan` / `apply`), wiring em `dare discover` (sem `--check`), capability `dare-discover` nos harnesses, testes de idempotência + rollback, e docs DEC-020.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | `InstallPlan` | Plano determinístico a partir de `DetectionReport` + opções | Unit |
| O-02 | `dare.config.json` | Ficheiro válido (schema 008); `ide` coerente com stacks/harnesses | Unit + fixture |
| O-03 | Árvore DARE | `DARE/` (mín. README + `EXECUTION/`) e `.dare/` criados | Unit |
| O-04 | Templates + graph | `templates/**` canónicos + `dare-graph.yml` no root | Unit |
| O-05 | `.gitignore` | Merge idempotente (linhas DARE sem duplicar) | Unit |
| O-06 | Harnesses | Install + `validate_*` dos 4 adapters passam | Unit/integ |
| O-07 | Idempotência | Segunda execução: sem duplicação / sem corrupção | Aceite MUST |
| O-08 | Rollback | Falha a meio restaura snapshot pré-apply (ficheiros tocados) | Unit |
| O-09 | Capability | `dare-discover` materializado nos 4 outputs da matrix | Assert paths |
| O-10 | CLI | `dare discover` (sem `--check`) exit 0 no happy path; `--check` inalterado (018) | Smoke |
| O-11 | Ralph Loop | `cargo fmt --check`, `clippy`, `test`, `audit`/`deny` | Exit 0 |
| O-12 | Docs DEC | `docs/compatibility/cli-discover-install.md` + DEC-020 | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 1: discover útil em brownfield |
| Tech Lead | Time DARE CLI Rust | InstallPlan + rollback; sem ciclos de deps |
| Engenheiro CLI | Time implementação | Wiring clap; reuso 008/009/011–014 |
| Usuário Final | Devs / agentes | `dare discover` após `--check` |
| CI | Pipelines | Smoke install em tempdir + validate harness |
| Compatibilidade | Tech Lead | Diff vs TS 3.18.1 classificado (A/B/C/D) |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `install` em `dare-project` | MUST | `plan_install` + `apply_install` (+ tipos `InstallPlan` / `InstallReport`); `dare-cli` thin |
| RF-02 | `dare discover` sem `--check` | MUST | Corre `detect` → plan → apply; **remove** stub Internal de 018; exit 0 se Ok |
| RF-03 | Preservar `--check` | MUST | Comportamento 018 inalterado (zero writes) |
| RF-04 | Flags CLI | MUST | `--dir`/`-d`; globais `--json` / `--no-color`; opcional `--force` (SHOULD) para sobrescrever managed |
| RF-05 | Pré-condição project root | MUST | Se `projectRoot` null → `InvalidInput`/`NotFound` tipado; **não** cria root artificial |
| RF-06 | Política de conflicts | MUST | Se `conflicts` não vazio: **não abortar** — instalar na mesma (como TS); emitir **warning** no human + campo `warnings`/`conflicts` no `InstallReport`; exit **0** no happy path. SHOULD: `--strict-conflicts` aborta com InvalidInput=4. Documentar em DEC-020 |
| RF-07 | Gerar `InstallPlan` | MUST | Lista ordenada de steps: config → dirs → templates/graph → gitignore → harnesses → capability paths; IDs estáveis |
| RF-08 | `dare.config.json` | MUST | Criar se ausente; se presente e unmanaged/custom → preserve (não clobber) salvo `--force`; campos mínimos: `ide` (ou `backend`), extras preservados (008) |
| RF-09 | Seleção de `ide` | MUST | Heurística: se 1 harness `present` → mapear; senão default `claude-code` 🟡 **ou** `hybrid` se cursor+antigravity; documentar tabela no Blueprint |
| RF-10 | Criar `DARE/` | MUST | Pelo menos `DARE/README.md` + `DARE/EXECUTION/` (dir); não apagar specs existentes |
| RF-11 | Criar `.dare/` | MUST | Dir + estado mínimo se aplicável (ex. placeholder sem state machine completa — 026+); path safety |
| RF-12 | Materializar templates | MUST | Copiar templates canónicos embed (009) para `templates/` no projeto (DESIGN/BLUEPRINT/TASKS/…); idempotente |
| RF-13 | `dare-graph.yml` | MUST | Criar no project root se ausente (contrato 007 mínimo: backend default documentado); preserve se customized |
| RF-14 | Merge `.gitignore` | MUST | Inserir bloco/linhas DARE (ex. `.dare/`, backups) sem duplicar; criar ficheiro se ausente |
| RF-15 | Aplicar harnesses | MUST | Chamar install dos 4 adapters (`force` conforme flag); preserve unmanaged (011–014) |
| RF-16 | Validar harnesses pós-install | MUST | `validate_*` dos 4 passa no happy path (ou subset instalado se política parcial — MUST = todos no v1 alpha) |
| RF-17 | Capability `dare-discover` | MUST | Garantir outputs da matrix nos 4 IDEs (já em `capability-matrix.yml`); conteúdo alinhado a `.claude/commands/dare-discover.md` / render 010 |
| RF-18 | Backup pré-write | MUST | Antes de sobrescrever ficheiro existente managed/alvo, `backup` 005 sob `.dare/backups/…` |
| RF-19 | Rollback em falha | MUST | Se step N falha: restaurar ficheiros backupados nesta sessão; remover ficheiros **criados** nesta sessão quando seguro; reportar erro original |
| RF-20 | Idempotência | MUST | `discover` ×2 no mesmo tree: segunda run exit 0; sem duplicar lines gitignore; validate ainda ok |
| RF-21 | `InstallReport` | MUST | Contagens: created/updated/skipped/backed_up; lista de paths relativos POSIX; `mode: "install"` |
| RF-22 | Saída human en-US | MUST | Resumo do plano aplicado + linha `mode: install`; erros tipados |
| RF-23 | `--json` | MUST | Envelope 004; `data` = report schema (Apêndice C); `schemaVersion: 1` |
| RF-24 | Exit codes | MUST | Alinhados a **004** (ver Apêndice D); documentar vs stub 018 |
| RF-25 | Docs DEC-020 | MUST | `docs/compatibility/cli-discover-install.md` + DEC-020 + classification vs TS |
| RF-26 | Smoke CLI | MUST | Install em tempdir fixture Node; segunda run; falha forçada → rollback |
| RF-27 | Dry-run | SHOULD | `--dry-run`: emite plan/report sem writes |
| RF-28 | Harness subset | COULD | `--ide <id>` limita adapters; default = todos |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI

```text
dare discover [--dir|-d <path>] --check              # 018 — zero writes
dare discover [--dir|-d <path>]                      # 019 — install (warn on conflicts)
dare discover [--dir|-d <path>] --force              # SHOULD — overwrite managed
dare discover [--dir|-d <path>] --dry-run            # SHOULD
dare discover [--dir|-d <path>] --strict-conflicts   # SHOULD — abort if stack conflicts
# + --json / --no-color globais (004)
```

### API de domínio (esboço)

```text
dare_project::plan_install(report: &DetectionReport, opts: &InstallOptions) -> CoreResult<InstallPlan>
dare_project::apply_install(root: &ProjectRoot, plan: &InstallPlan, opts: &InstallOptions) -> CoreResult<InstallReport>
dare_project::install(start: &Path, opts: &InstallOptions) -> CoreResult<InstallReport>
  // detect → warn if conflicts (unless --strict-conflicts) → plan → apply
```

### Steps canónicos do `InstallPlan` (ordem fixa)

| # | Step id | Efeito |
|---|---------|--------|
| 1 | `ensure_dirs` | `DARE/`, `DARE/EXECUTION/`, `.dare/` |
| 2 | `write_config` | `dare.config.json` |
| 3 | `materialize_templates` | `templates/**` |
| 4 | `write_graph` | `dare-graph.yml` |
| 5 | `merge_gitignore` | `.gitignore` |
| 6 | `install_harness_claude` | adapter 011 |
| 7 | `install_harness_cursor` | adapter 012 |
| 8 | `install_harness_codex` | adapter 013 |
| 9 | `install_harness_antigravity` | adapter 014 |
| 10 | `ensure_capability_discover` | ficheiros `dare-discover` (se não cobertos pelo install matrix completo) |
| 11 | `validate_harnesses` | validate_* |

> Nota: se o install dos adapters 011–014 já materializa **todas** as capabilities da matrix, o step 10 pode ser no-op verificado — Blueprint decide; critério: paths `dare-discover` existem nos 4 IDEs.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo report + opts → mesmo plan (ordem de steps/paths) | Unit |
| RNF-02 | Performance | Install tipicamente < 5 s em fixture pequena (sem rede) | Smoke |
| RNF-03 | Idempotência | Segunda apply ≤ N writes (só skips) | Contagem no report |
| RNF-04 | Observabilidade | Tracing span por step; erros `CoreError` tipados | Unit |
| RNF-05 | Manutenibilidade | Lógica em `dare-project`; CLI thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths; atomic_write 005 | CI 003 |
| RNF-07 | Rollback | Janela de sessão: só artefatos desta apply | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dir` / project root; jail `ProjectRoot` / `SafeRelativePath` em toda write | OWASP A03 / 005 |
| RS-02 | Não logar conteúdo completo de configs com secrets; redact 004 | OWASP A02 |
| RS-03 | Writes só sob project root; deny symlink escape | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell concatenado | Supply chain / 006 |
| RS-06 | Preserve unmanaged (force=false default) — não clobber regras do utilizador | Integrity |
| RS-07 | Backup antes de overwrite; rollback testado em falha parcial | Integrity |
| RS-08 | Cap de tamanho ao ler ficheiros existentes para merge (gitignore/config) | Availability |
| RS-09 | `--check` permanece zero-write (não regressar 018) | Integrity |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Domínio | `dare-project` (+ `install.rs`) | `0.1.0-alpha.0` |
| CLI | `dare-cli` + clap | workspace |
| Path / FS / backup | `dare-core` | 005 |
| Config | `dare-config` | 008 |
| Assets / templates | `dare-assets` embed + materialize | 009 |
| Capabilities | `capability-matrix.yml` | 010 |
| Harnesses | `dare-harness` install/validate | 011–014 |
| Contratos | `dare-contracts` (`dare-graph.yml`) | 007 |
| JSON | serde camelCase | workspace |
| Saída | renderer 004 | DEC-005 |
| Testes | tempfile + fixtures 018 + smoke | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem do projeto | Local | read/write | In/Out | config, DARE, templates, harness files | CLI |
| `dare-harness` | In-process | API | Out | install/validate | adapters |
| `dare-assets` / `dare-config` | In-process | API | Out | templates, config default | 008/009 |
| stdout | Terminal | — | Out | human / JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | UX install / gitignore / paths | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **011–014** e **018** concluídos (MUST do microplano).
- Mensagens **en-US**.
- Sem alterar schema de `DetectionReport` (018) sem ADR.
- Sem `dare init` / scaffold de aplicação greenfield.
- Sem `dare update` completo (021–022) — só install inicial.
- Sem GraphRAG ingest / Neo4j.
- `dare-project` **não** depende de `dare-cli`; deps permitidas: `dare-core`, `dare-harness`, `dare-assets`, `dare-config`, `dare-contracts` (evitar ciclo).
- Diffs intencionais vs TS (ex.: exit codes 004, correção mojibake) → DEC-020 / classification matrix.
- Bump de schema de `InstallReport` exige ADR + migration note.

---

## 10. FORA DO ESCOPO (v1)

- `dare validate` (→ **020**).
- `dare update` / UPDATE-MANIFEST / migrations de release (→ **021–022**).
- `dare init` / `bootstrap` / stacks scaffolder (→ **046–047**).
- Resolução interativa de conflicts de stack (UI prompt) — só warning + opcional `--strict-conflicts`.
- Install remoto de skills-pacote / registry (→ **044–045**).
- Drivers de execução de agentes / worktrees (→ **030+**).
- Self-update do binário (→ **053**).
- Telemetria remota.
- Materializar **todas** as 49 capabilities se o alpha puder reutilizar install adapters existentes — MUST mínimo: harness validate ok **e** `dare-discover` presente; full matrix parity já coberta pelos adapters 011–014 quando `install_*` completo.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Rollback incompleto (dirs criados) | Média | Alto | Journal de created paths; rmdir vazio; testes de falha injetada |
| R-02 | Clobber de `dare.config.json` custom | Média | Alto | Preserve + extras 008; backup; `--force` explícito |
| R-03 | Repo polyglot (Node+Rust) comum em brownfield | Alta | Baixo | Default = warn+install (RF-06); `--check` para revisão; `--strict-conflicts` só se o utilizador quiser gate |
| R-04 | Diff paths/conteúdo vs TS 3.18.1 | Alta | Médio | DEC-020 + classification; golden parcial |
| R-05 | Install harness parcial (1 IDE falha) | Média | Alto | Abort + rollback; não deixar half-installed sem report |
| R-06 | Idempotência quebrada no merge `.gitignore` | Média | Médio | Bloco marcado / set de linhas; teste double-run |
| R-07 | Regressão `--check` com writes | Baixa | Alto | Smoke 018 permanece; gate CI |
| R-08 | Ciclo de deps `dare-project`↔`dare-assets` | Baixa | Alto | Dep graph test; materialize só via APIs públicas |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-28 priorizados (InstallPlan, rollback, idempotência, harnesses, capability)
- [x] Política de **conflicts** (warn + install; `--strict-conflicts` opcional) aceite
- [x] Separação 018 (`--check`) vs 019 (install) aceite
- [x] RS / backup / path safety validados
- [x] Fora de escopo (validate, update, init) alinhado
- [x] Riscos R-01…R-08 com mitigação
- [x] Pronto para `/dare-blueprint` → `BLUEPRINT-019-discover-instalacao-do-dare.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-project/src/install.rs` | InstallPlan / apply / rollback |
| `crates/dare-project/src/lib.rs` | Re-exports |
| `crates/dare-cli/src/commands/discover.rs` | Wiring check vs install |
| `crates/dare-cli/src/main.rs` | Flags Discover |
| `assets/capability-matrix.yml` | Entrada `dare-discover` (já existe) |
| `assets/templates/**` | Fonte templates canónicos |
| `tests/fixtures/existing-*-project/` | Fixtures 018 reutilizadas |
| `docs/compatibility/cli-discover-install.md` | Docs DEC (a criar) |
| `docs/DECISION-LOG.md` | DEC-020 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `detect` / `--check` (018) | ✅ DONE |
| `dare discover` sem `--check` | 🔴 stub Internal exit 1 |
| `install.rs` / InstallPlan | 🔴 ausente |
| Harness `install_*` / `validate_*` | ✅ 011–014 |
| `materialize_to` / templates embed | ✅ 009 (dest típico `.dare/assets` — pode precisar copy para `templates/` root) |
| `dare-config` default + load | ✅ 008 |
| Backup/restore | ✅ 005 |
| Capability `dare-discover` na matrix | ✅ id presente |
| Docs install | 🔴 ausente |

## Apêndice C — `InstallReport` schema 1 (proposto)

```json
{
  "schemaVersion": 1,
  "mode": "install",
  "projectRoot": "/abs",
  "steps": [
    {
      "id": "write_config",
      "status": "created",
      "paths": ["dare.config.json"]
    }
  ],
  "created": ["dare.config.json", "DARE/README.md"],
  "updated": [],
  "skipped": [".gitignore"],
  "backedUp": [".dare/backups/.../dare.config.json"],
  "harnessesValidated": ["antigravity", "claude", "codex", "cursor"],
  "conflicts": [],
  "warnings": [],
  "dryRun": false
}
```

Notas:
- Arrays sorted lexicograficamente onde aplicável.
- `status` por step: `created` \| `updated` \| `skipped` \| `failed` \| `rolled_back`.
- `conflicts` espelha o `DetectionReport` (pode ser não vazio com exit 0).
- `warnings` inclui mensagens en-US (ex.: stack conflict); sorted.
- Campos extras → bump `schemaVersion` + ADR.

## Apêndice D — Exit codes (alinhados a 004)

| Code | `ErrorKind` | Uso neste microplano |
|------|-------------|----------------------|
| 0 | — | install Ok **ou** `--check` Ok |
| 1 | Internal | falha interna inesperada / rollback incompleto grave |
| 2 | Usage | args inválidos / clap |
| 3 | NotFound | `--dir` / project root ausente |
| 4 | InvalidInput | path safety; input inválido; **ou** `--strict-conflicts` com conflicts ≠ [] |
| 5 | Io | I/O ao escrever/ler tree |

> **Diff vs stub 018:** sem `--check` deixa de ser exit 1 “not implemented”; passa a install (0 ou erro tipado). Documentar em DEC-020 (classe B).

## Apêndice E — Heurística `ide` (🟡)

| Condição | `ide` escrito |
|----------|---------------|
| Só claude present / default | `claude-code` |
| Só cursor | `cursor` |
| Só codex | `codex` |
| Só antigravity | `antigravity` |
| cursor + antigravity | `hybrid` |
| claude + cursor | `claude-hybrid` |
| Nenhum / ambíguo | `claude-code` (default alpha) |

Blueprint pode ajustar com evidência TS `installIdeFiles`.

## Apêndice F — Próximas etapas

1. Revisar e aprovar este Design (heurística `ide` ainda 🟡 no Apêndice E — ok para Blueprint).  
2. `/dare-blueprint` → `BLUEPRINT-019-discover-instalacao-do-dare.md`.  
3. `/dare-tasks` → `mp019-*` + `dare-dag-019.yaml`.  
4. Após closeout → [`020-validate.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/020-validate.md).

### Política de conflicts (congelada neste Design)

| Modo | Comportamento |
|------|----------------|
| Default | Conflicts ≠ [] → **warning** + install continua → exit **0** se apply Ok |
| `--strict-conflicts` (SHOULD) | Conflicts ≠ [] → **não** aplica → InvalidInput exit **4** |
| `--check` (018) | Só reporta conflicts; zero writes; exit 0 |

Justificativa: repos brownfield polyglot são legítimos; abortar no default repetiria um falso positivo e diverge do TS (`installIdeFiles` não bloqueia por stack mista). O utilizador revisa com `--check` antes se quiser.
