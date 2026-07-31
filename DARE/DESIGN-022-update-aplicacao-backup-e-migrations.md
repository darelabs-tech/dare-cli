# DESIGN: Update — aplicação, backup e migrations (Microplano 022)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/022-update-aplicacao-backup-e-migrations.md`  
> **Referência:** Microplano **021** (`UpdatePlan`, classificação SHA-256, `--dry-run`/`--target`) · **005** (atomic write / backup / path safety) · **006** (processos) · **008** (`dare-config` migrate) · **009** (assets embed) · **011–014** (harnesses) · **004** (saída/`--json`) · Documento Mestre §21 · baseline TS 3.18.1  
> **Posição:** 22 de 56  
> **Arquivo:** `DARE/DESIGN-022-update-aplicacao-backup-e-migrations.md` (não substitui Designs 001–021)  
> **Nota:** Este Design cobre **aplicar** o `UpdatePlan` (021) com políticas keep/replace/ask, backup versionado, migrations de config, escrita atómica, `--force`/`-y`, report human/JSON e **rollback** se a aplicação falhar. Planeamento/dry-run/classificação permanecem em **021**. Self-update do binário (`dare self update`) fica em ciclo posterior (**041+** / ADR-009).

---

## 1. DESCRIÇÃO

Este Design cobre a **fase de aplicação** de `dare update` no CLI nativo Rust: a partir de um `UpdatePlan` (021), decidir por ficheiro (keep / replace / ask), criar backup versionado sob `.dare/backup-*`, aplicar migrations de `dare.config.json` via `dare-config`, escrever assets/harness paths de forma atómica, e emitir `UpdateApplyReport` (human + `--json`). Se qualquer step falhar, **rollback** restaura o snapshot da sessão — aplicação parcial **não** persiste.

O problema: após `--dry-run`, developers precisam sincronizar artefatos DARE com a versão do CLI **sem perder customizações** sem consentimento, e com caminho de restauração verificável. Quem consome são developers, agentes IDE (`/dare-update`) e CI (com `-y` / `--force` explícito).

A entrega principal: `crates/dare-update/src/apply.rs`, wiring em `crates/dare-cli/src/commands/update.rs` (`dare update`, `--force`, `-y`), testes de rollback + preserve customized, e docs DEC-023.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Políticas keep/replace/ask | Matriz status×flags → ação determinística | Unit 100% casos tabela |
| O-02 | Customizações preservadas | `customized` + default/`-y` **não** sobrescreve sem `--force` | Aceite MUST |
| O-03 | Backup versionado | Dir `.dare/backup-<version>/…` com cópias pré-write; restore round-trip | Unit |
| O-04 | Migrations config | `apply_migrate` / plan 008 integrado no apply quando manifesto/plan exigir | Unit |
| O-05 | Escrita atómica | `atomic_write` 005 em todos os replaces | Unit |
| O-06 | `--force` | `customized` → replace (com backup) | Unit + smoke |
| O-07 | `-y` / `--yes` | Aplica plan sem prompts; customized → **keep** (salvo `--force`) | Unit + smoke |
| O-08 | Report human/JSON | `UpdateApplyReport` schemaVersion **1** camelCase | Assert eq |
| O-09 | Rollback | Falha a meio → tree equivalente ao pré-apply (ficheiros da sessão) | Unit |
| O-10 | Aplicação parcial | Após rollback, nenhum ficheiro “meio aplicado” residual da sessão | Aceite MUST |
| O-11 | Ralph Loop | fmt / clippy / test / audit / deny | Exit 0 |
| O-12 | Docs DEC | `docs/compatibility/cli-update-apply.md` + DEC-023 + diffs classificados | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 3: update seguro em brownfield |
| Tech Lead | Time DARE CLI Rust | Políticas + rollback; crate `dare-update`; sem ciclos |
| Engenheiro CLI | Time implementação | `apply.rs` + flags clap; reuso 005/008/021 |
| Usuário Final | Devs / agentes | `dare update -y` sem perder edits manuais |
| CI | Pipelines | `-y` não interativo; `--force` só se explícito |
| Compatibilidade | Tech Lead | Diff vs TS 3.18.1 (backup path, ask/TTY) classificado |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `apply` em `dare-update` | MUST | `apply_update(plan, opts) -> CoreResult<UpdateApplyReport>`; cli thin |
| RF-02 | Pré-condição plan 021 | MUST | Consome `UpdatePlan` + classificações `identical\|missing\|apply\|customized`; sem reclassificar SHA no apply salvo verificação opcional SHOULD |
| RF-03 | `dare update` (apply) | MUST | Sem `--dry-run`: corre plan→apply; exit 0 no happy path |
| RF-04 | Preservar `--dry-run` 021 | MUST | Com `--dry-run`: **zero writes** (comportamento 021 inalterado) |
| RF-05 | Política **keep** | MUST | Não escreve o path; conta em `kept`; usado para `identical` e para `customized` sem consentimento de replace |
| RF-06 | Política **replace** | MUST | Backup + `atomic_write` do conteúdo canónico (embed/manifest); conta em `replaced`/`created` |
| RF-07 | Política **ask** | MUST | Em TTY sem `-y`/`--force`: pedir Y/n por ficheiro `customized` (ou batch); default **N** = keep. Sem TTY: tratar ask como **keep** (seguro) |
| RF-08 | Matriz de decisão | MUST | Ver tabela §4.1 — congelada no Blueprint |
| RF-09 | `--force` | MUST | Força **replace** em `customized` e `apply`; não pede ask |
| RF-10 | `-y` / `--yes` | MUST | Aplica sem prompts; **não** implica `--force` (customized → keep) |
| RF-11 | Backup versionado | MUST | Antes do primeiro write da sessão: criar `.dare/backup-<cliVersion>/` (ou `.dare/backup-<cliVersion>-<utc>/` se colisão); copiar cada ficheiro a sobrescrever; path safety |
| RF-12 | Restore a partir do backup | MUST | API/`restore_session` consegue repor ficheiros backupados; teste round-trip |
| RF-13 | Migrations de config | MUST | Se plan incluir step(s) de config **ou** `dare.config.json` no conjunto apply: chamar `dare_config::apply_migrate` (ou plan+apply) com backup; pointers JSON documentados |
| RF-14 | Escrita atómica | MUST | Todo replace/create de ficheiro via `dare_core::fs::atomic_write` sob `ProjectRoot` |
| RF-15 | Rollback em falha | MUST | Journal de sessão: backups + created; on error → restore backups em ordem inversa + remover created desta sessão; propagar erro original |
| RF-16 | Aplicação parcial não persiste | MUST | Após falha+rollback, listing relevante == pré-apply (exceto dir backup vazio/parcial limpo — Blueprint define cleanup) |
| RF-17 | `UpdateApplyReport` | MUST | schemaVersion 1; contagens kept/replaced/created/skipped/backedUp/migrated; lists paths POSIX; `mode: "update"`; `backupRoot` |
| RF-18 | Human en-US | MUST | Resumo ações + backup root + linha `mode: update`; warnings de keep customized |
| RF-19 | `--json` | MUST | Envelope 004; `data` = report schema Apêndice C |
| RF-20 | `--dir` / project root | MUST | Resolve root (walk 018/info); InvalidInput/NotFound se sem projeto DARE aplicável 🟡 Blueprint |
| RF-21 | `--target <harness>` | MUST | Herdado 021: apply só entries do harness (incl. **codex**); default = todos |
| RF-22 | Exit codes | MUST | Alinhados a **CoreError 004** (Apêndice D) |
| RF-23 | Fixtures | MUST | `customized-assets` + caso missing/apply; falha forçada → rollback |
| RF-24 | Docs DEC-023 | MUST | `docs/compatibility/cli-update-apply.md` + DEC-023 + classification vs TS |
| RF-25 | Smoke CLI | MUST | `dare update -y` tempdir; `--force` customized; falha→rollback; `--dry-run` zero write |
| RF-26 | Interativo ask | SHOULD | Prompt batch “replace all customized?” quando TTY |
| RF-27 | Cleanup backup órfão | SHOULD | Flag ou GC docs; v1 pode deixar backups versionados |
| RF-28 | `--backup-dir` override | COULD | Path relativo sob root; default automático |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### 4.1 Matriz de decisão (MUST)

| Classificação (021) | Flags | Ação |
|---------------------|-------|------|
| `identical` | qualquer | **keep** |
| `missing` | qualquer apply path | **replace** (create) + sem backup de destino (não existe) |
| `apply` | default / `-y` / `--force` | **replace** (+ backup se destino existe) |
| `customized` | default (TTY) | **ask** → Y replace / N keep (default N) |
| `customized` | default (non-TTY) | **keep** |
| `customized` | `-y` only | **keep** |
| `customized` | `--force` (± `-y`) | **replace** (+ backup) |

> **Aceite:** “Nenhuma customização é perdida sem consentimento” = sem `--force` e sem resposta Y explícita no ask, `customized` nunca é replace.

### Superfície CLI

```text
dare update --dry-run [--target <harness>] [-d <dir>]     # 021 — zero writes
dare update [-y|--yes] [--force] [--target <harness>] [-d <dir>]   # 022 — apply
# + --json / --no-color globais (004)
```

### API de domínio (esboço)

```text
dare_update::apply_update(root, plan, opts) -> CoreResult<UpdateApplyReport>
dare_update::format_apply_human(r) -> String
dare_update::apply_report_to_json(r) -> Value
UPDATE_APPLY_SCHEMA_VERSION: u32 = 1

ApplyOptions {
  yes: bool,       // -y
  force: bool,     // --force
  interactive: bool, // TTY detected && !yes && !force
  target: Option<HarnessId>, // from plan filter 021
}
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Ordem de apply = ordem estável do `UpdatePlan` (021); report lists sorted | Unit |
| RNF-02 | Performance | Update típico < 5 s em projeto médio com dezenas de assets | Smoke informal |
| RNF-03 | Disponibilidade | Funciona offline (só assets embed + disco local) | Unit |
| RNF-04 | Observabilidade | Erros tipados; journal de rollback logável sem secrets | Unit |
| RNF-05 | Manutenibilidade | Lógica em `dare-update`; `update.rs` thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths; backup dirs | CI 003 |
| RNF-07 | Integridade | Falha mid-write não deixa ficheiro truncado (atomic_write) | Unit 005 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dir` / paths do plan; rejeitar escape via `ProjectRoot` / `SafeRelativePath` | OWASP A03 |
| RS-02 | Não imprimir conteúdo completo de configs com secrets; report só paths + status; redact 004 | OWASP A02 |
| RS-03 | `--force` não bypassa jail de path; ownership = project root do usuário | OWASP A01 / Integrity |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell concatenado | Supply chain / 006 |
| RS-06 | Backups e writes só sob project jail; symlinks conforme 005 | Path safety |
| RS-07 | Rollback testado — falha parcial não deixa estado inconsistente duradouro | Integrity |
| RS-08 | Limitar tamanho de leitura de assets/manifests (cap bytes) | Availability |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-update` | `0.1.0-alpha.0` (criada/estendida em 021) |
| CLI | `dare-cli` + clap | workspace |
| Path / FS | `dare-core` atomic_write, backup/restore, ProjectRoot | 005 |
| Config migrate | `dare-config` apply_migrate / plan_migrate | 008 |
| Plan / SHA | `dare-update` plan 021 + sha2 | 021 |
| Assets | `dare-assets` embed / materialize | 009 |
| JSON | serde / serde_json camelCase | workspace |
| Saída | renderer 004 | DEC-005 |
| Testes | tempfile + fixtures + smoke | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem projeto | Local | read/write | In/Out | assets, config, `.dare/backup-*` | CLI |
| Assets embutidos | Embed | in-process | In | bytes canónicos + hashes | dare-assets |
| `dare-config` migrate | In-process | API | In/Out | `dare.config.json` | dare-config |
| stdin (ask) | TTY | — | In | Y/n | CLI (só interactive) |
| stdout | Terminal | — | Out | human / JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | UX políticas / backup | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisito:** microplano **021** concluído (`UpdatePlan`, classificação, dry-run, Codex no plan, manifest).
- Mensagens **en-US**.
- Exit codes alinhados a **004** (não inventar mapa paralelo sem DEC).
- Bump de `schemaVersion` do report exige ADR + migration note.
- Sem self-update do binário neste ciclo.
- Sem alterar contratos de disco breaking sem ADR.
- Dependência: `dare-update` → `dare-core` + `dare-config` + `dare-assets` (+ contracts se necessário); **não** depende de `dare-cli`.
- Bug TS “UPDATE-MANIFEST sem releases 3.9+” **não** será reproduzido (decisão Mestre §21) — classificar Classe B/C na DEC.
- Path de backup TS `.dare/backup-<versão>/` vs core `.dare/backups/<utc>-sha/` — **resolver no Blueprint** (preferência: session dir versionado compat TS + cópias via primitives 005) 🟡.

---

## 10. FORA DO ESCOPO (v1)

- Planeamento / SHA / `--dry-run` / leitura de manifest (→ **021**, já pré-requisito).
- `dare self update` / canais stable-beta (→ ciclo self-update / ADR-009).
- `dare skill update` / registry remoto.
- Graph drift / ingest (040+).
- Merge field-level de JSON de settings IDE (harness continua skip/replace managed como 011–014).
- UI/prompt rico multi-select (ask Y/n ou batch SHOULD basta).
- GC automático agressivo de todos os backups históricos (SHOULD docs only).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | `--force` acidental em CI | Média | Alto | Docs; `-y` ≠ force; smoke prova customized kept com só `-y` |
| R-02 | Ask em non-TTY bloqueia / hang | Média | Médio | Non-TTY → keep; nunca ler stdin sem TTY |
| R-03 | Diff path backup vs TS / vs 005 | Alta | Médio | Blueprint congela layout; DEC classifica |
| R-04 | Rollback incompleto (dirs criados) | Média | Alto | Journal created+backed_up; testes falha forçada |
| R-05 | Migration config corrompe extras | Baixa | Alto | Reusar 008 preserve extras; backup pré-migrate |
| R-06 | Ordem apply não-determinística | Média | Médio | Sort estável do plan 021 |
| R-07 | Partial write sem atomic | Baixa | Alto | Só `atomic_write` |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-28 priorizados (matriz keep/replace/ask + rollback)
- [ ] Separação 021 (plan/dry-run) vs 022 (apply) aceite
- [ ] Matriz §4.1 aceite (“customized sem consentimento = keep”)
- [ ] Layout `.dare/backup-*` a congelar no Blueprint aceite como aberto 🟡
- [ ] RS / path safety / atomic / redact validados
- [ ] Fora de escopo (self-update, skill update) alinhado
- [ ] Riscos R-01…R-07 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-022-update-aplicacao-backup-e-migrations.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-update/src/apply.rs` | Domínio apply + rollback |
| `crates/dare-update/src/plan.rs` | Pré-requisito 021 |
| `crates/dare-cli/src/commands/update.rs` | Wiring clap |
| `crates/dare-cli/src/main.rs` | `Commands::Update` |
| `.dare/backup-<version>/` | Backup de sessão (contrato disco) |
| `dare.config.json` | Migrations 008 |
| `tests/fixtures/customized-assets/` | Fixture preserve |
| `docs/compatibility/cli-update-apply.md` | Docs DEC (a criar) |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| Crate `dare-update` | 🔴 / ⬜ via 021 |
| `apply.rs` / políticas / rollback | 🔴 ausente |
| `Commands::Update` apply path | 🔴 ausente |
| `atomic_write` / `backup` 005 | ✅ |
| `dare-config` migrate | ✅ parcial (008) |
| Fixture `customized-assets` | 📋 inventário; materializar se preciso |
| Docs `cli-update-apply.md` | 🔴 ausente |

## Apêndice C — Campos JSON schema 1 (congelados)

```json
{
  "schemaVersion": 1,
  "mode": "update",
  "cliVersion": "0.1.0-alpha.0",
  "projectRoot": "/abs",
  "backupRoot": ".dare/backup-0.1.0-alpha.0",
  "target": null,
  "force": false,
  "yes": true,
  "kept": ["path/a"],
  "created": ["path/b"],
  "replaced": ["path/c"],
  "skipped": [],
  "backedUp": ["path/c"],
  "migrated": ["dare.config.json"],
  "warnings": [],
  "rolledBack": false
}
```

Notas:
- Em falha com rollback bem-sucedido: `rolledBack: true`; lists refletem tentativa ou ficam vazias pós-restore — **Blueprint congela** semântica.
- Paths em lists: relativos POSIX ao project root; sorted.
- Campos extras → bump schemaVersion + ADR.

## Apêndice D — Exit codes (v1, alinhados a 004)

| Code | `ErrorKind` | Uso |
|------|-------------|-----|
| 0 | — | Apply (ou dry-run 021) OK |
| 1 | Internal | Bug / estado inconsistente pós-rollback falho |
| 2 | Usage | Args inválidos |
| 3 | NotFound | Project root / plan entry path base ausente |
| 4 | InvalidInput / Config | Path safety; config migrate inválida |
| 5 | Io | I/O / falha de write (após tentativa de rollback) |

## Apêndice E — Próximas etapas

1. Revisar e aprovar este Design (esp. matriz §4.1 e backup path 🟡).  
2. `/dare-blueprint` → `BLUEPRINT-022-update-aplicacao-backup-e-migrations.md`.  
3. `/dare-tasks` → `mp022-*`.  
4. Após closeout → [`023-design-deterministico.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/023-design-deterministico.md).
