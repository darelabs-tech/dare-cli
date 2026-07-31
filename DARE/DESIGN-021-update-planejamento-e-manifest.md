# DESIGN: Update — planejamento e manifest (Microplano 021)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/021-update-planejamento-e-manifest.md`  
> **Referência:** Microplanos **005** (path/fs/SHA) · **007** (`UpdateManifestV1`) · **008** (config) · **009** (assets + SHA-256) · **010** (capability matrix) · **011–014** (harnesses; `UPDATE_HARNESS_IDES` + Codex DEC-014) · **019** (install / managed paths) · baseline TS 3.18.1  
> **Posição:** 21 de 56  
> **Arquivo:** `DARE/DESIGN-021-update-planejamento-e-manifest.md` (não substitui Designs 001–020)  
> **Nota:** Este Design cobre **apenas** leitura de manifests, classificação SHA-256, construção de `UpdatePlan` e superfície CLI **`--dry-run` / `--target`** (zero writes no happy path). Aplicação, backup versionado, políticas keep/replace/ask, migrations de config, `--force`/`-y` e rollback ficam em **022**.

---

## 1. DESCRIÇÃO

Este Design define o mecanismo de **planeamento de atualização** do DARE CLI nativo: comparar assets/artefactos já instalados no projeto com a versão canónica embutida (ou manifesto de release) e produzir um `UpdatePlan` determinístico, sem escrever em disco.

O problema: após upgrade do binário, developers e agentes precisam saber **exactamente** o que mudaria — ficheiros em falta, ficheiros idênticos, ficheiros a aplicar, e ficheiros **customizados** pelo utilizador — antes de qualquer apply. Hoje o TS 3.18.1 usa `templates/UPDATE-MANIFEST.json` (schema 1) com lacunas conhecidas (releases 3.9+ ausentes; Codex omitido de `appliesTo`). O port Rust deve **ler** o schema 1, introduzir um **manifest novo versionado** sem reproduzir esses bugs, e garantir que **Codex entra no plano**.

A entrega vive em `crates/dare-update/src/plan.rs` (+ crate nova no workspace), wiring fino em `dare update --dry-run` / `--target <harness>`, reuso de `dare-contracts::UpdateManifestV1`, hashes de `dare-assets`, e docs DEC-021 / compatibilidade.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Ler UPDATE-MANIFEST schema 1 | `load_update_manifest` / parser rejeita schema ≠ 1 | Unit |
| O-02 | Manifest novo versionado | Artefacto canónico embed/path com schema ≥ 2 (ou `version` dedicado) documentado | Unit + fixture |
| O-03 | Classificação SHA-256 | Cada path → `identical` \| `missing` \| `apply` \| `customized` | Unit golden |
| O-04 | `UpdatePlan` | Plano ordenado, IDs estáveis, filtrável por harness | Unit |
| O-05 | `--dry-run` | Emite plano/report; **zero** writes sob project root | Integ (listing before/after) |
| O-06 | `--target <harness>` | Plano restrito ao harness pedido (ex. `codex`) | Unit + smoke |
| O-07 | Codex no plano | Entries/`appliesTo`/`UPDATE_HARNESS_IDES` incluem `codex`; teste `update_policies_include_codex` | Assert |
| O-08 | Dry-run descreve mudanças exactas | Contagens + lista de paths relativos POSIX bate com classificação | Aceite MUST |
| O-09 | Customizações detectadas | Fixture `customized-assets` → status `customized` | Golden |
| O-10 | Ralph Loop | `cargo fmt --check`, `clippy`, `test`, `audit`/`deny` | Exit 0 |
| O-11 | Docs DEC-021 | `docs/compatibility/cli-update-plan.md` + DEC-021 + classification vs TS | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 3: update previsível e seguro |
| Tech Lead | Time DARE CLI Rust | Manifest dual (v1 + novo); Codex; sem ciclos de deps |
| Engenheiro CLI | Time implementação | Crate `dare-update`; wiring clap dry-run |
| Usuário Final | Devs / agentes IDE | `dare update --dry-run` antes de apply |
| CI | Pipelines | Golden + fixture `customized-assets` |
| Compatibilidade | Tech Lead | Diff vs TS 3.18.1 classificado (A/B/C/D); bugs legados **não** reproduzidos |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-update` | MUST | Membro do workspace; `plan.rs` com API pública; `dare-cli` thin |
| RF-02 | Ler UPDATE-MANIFEST schema 1 | MUST | Reusar `dare-contracts::UpdateManifestV1`; rejeitar `schemaVersion ≠ 1` |
| RF-03 | Manifest novo versionado | MUST | Definir e embutir/carregar manifesto “desired state” (schema documentado, ex. `schemaVersion: 2` ou `updateManifestVersion: 2`) com releases **completas** até a versão do CLI; **não** omitir 3.9+ como no bug TS |
| RF-04 | Inventário de paths geridos | MUST | Plano deriva de: entries do manifest novo + paths managed dos adapters 011–014 / assets 009 (lista fechada no Blueprint) |
| RF-05 | Calcular SHA-256 | MUST | Reusar `dare_assets::sha256_hex` (ou equivalente `sha2`); hash do conteúdo em disco vs hash canónico esperado |
| RF-06 | Classificar ficheiro | MUST | Ver Apêndice B — enum `AssetUpdateStatus` |
| RF-07 | `identical` | MUST | Ficheiro existe e SHA == esperado canónico |
| RF-08 | `missing` | MUST | Ficheiro não existe no project root (path relativo) |
| RF-09 | `apply` | MUST | Existe **ou** está ausente mas conteúdo esperado ≠ managed actual: ficheiro managed com SHA ≠ esperado **e** ainda marcado managed 🟡 **OU** ausente → tipicamente `missing`; se managed e hash diverge do **último known-good** mas bate baseline antigo → `apply`. Regra canónica no Blueprint: **missing** se ausente; **apply** se existe, é managed (ou sem marcador de custom), e SHA ≠ canónico actual; **customized** se existe e SHA ≠ canónico **e** conteúdo não é managed / divergiu do managed esperado |
| RF-10 | `customized` | MUST | Ficheiro existe, SHA ≠ canónico, e política de detecção indica edição do utilizador (ex. sem `<!-- dare:managed` / frontmatter unmanaged — alinhado 011–014); fixture `customized-assets` |
| RF-11 | Criar `UpdatePlan` | MUST | Struct com `schemaVersion: 1`, lista ordenada de `UpdateItem { path, status, expectedSha256?, actualSha256?, appliesTo[] }`; ordenação determinística por path POSIX |
| RF-12 | Filtrar por `appliesTo` | MUST | Changes com `appliesTo: ["*"]` aplicam a todos; harness-specific só entram se o target (ou set activo) intersecta |
| RF-13 | Codex explícito | MUST | (a) `UPDATE_HARNESS_IDES` inclui `codex`; (b) manifesto novo tem entries/`appliesTo` que cobrem paths Codex (`AGENTS.md`, skills); (c) teste dedicado falha se Codex omitido — Classe C vs TS (TS omitia Codex de `appliesTo`) |
| RF-14 | CLI `dare update` | MUST | Subcomando registado; sem flags de apply obrigatórias neste microplano |
| RF-15 | `--dry-run` | MUST | Default **recomendado** para alpha 🟡 **ou** flag obrigatória até 022; emite plano; **zero** writes (incl. sem backup, sem touch `.dare/`) |
| RF-16 | `--target <harness>` | MUST | Aceita id de harness: `claude-code` \| `cursor` \| `codex` \| `antigravity` \| `hybrid` \| `claude-hybrid` (subset de `UPDATE_HARNESS_IDES`); plano filtrado; id inválido → `InvalidInput` exit 4 |
| RF-17 | Sem `--target` | MUST | Plano cobre **todos** os harnesses relevantes + changes `*` |
| RF-18 | Saída human en-US | MUST | Resumo: contagens por status + lista de paths (customized destacados); mensagens sem mojibake |
| RF-19 | `--json` | MUST | Envelope 004; `data` = `UpdatePlan` / dry-run report (Apêndice C); `schemaVersion: 1` |
| RF-20 | Exit codes | MUST | Happy dry-run → 0; alinhados a **004** (Apêndice D) |
| RF-21 | Apply ainda não | MUST | Sem `--dry-run` (e sem flags 022): ou (A) exige `--dry-run` nesta release, ou (B) stub `Internal`/`NotImplemented` tipado apontando 022 — **decidir no Blueprint**; não aplicar writes silenciosas |
| RF-22 | Path safety | MUST | Toda leitura via `ProjectRoot` / `SafeRelativePath`; deny `..`, absolutos, symlink escape |
| RF-23 | Caps de leitura | MUST | `read_limited` (007) ao hashear ficheiros existentes |
| RF-24 | Docs DEC-021 | MUST | `docs/compatibility/cli-update-plan.md` + DEC-021 + matriz Classe A/B/C/D vs TS |
| RF-25 | Smoke CLI | MUST | `--dry-run --json` em fixture com managed + customized; assert statuses |
| RF-26 | `--dir` / root | SHOULD | Resolver project root como `info`/`discover` (`--dir`/`-d` ou cwd walk) |
| RF-27 | Target por versão CLI | COULD | Ambiguity skill IDE (`--target 3.2.0`): **fora** deste Design — se necessário, flag separada `--to-version` em ciclo futuro; microplano 021 = harness |
| RF-28 | Diff textual no dry-run | COULD | Mostrar unified diff; v1 = status + hashes bastam |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI (021)

```text
dare update --dry-run [--target <harness>] [--dir|-d <path>]
dare update --target <harness> --dry-run   # equivalente
# + --json / --no-color globais (004)

# NÃO neste microplano (→ 022):
# dare update [-y|--force]   # apply
# políticas keep|replace|ask, backup, migrations
```

### API de domínio (esboço)

```text
dare_update::load_legacy_manifest(...) -> CoreResult<UpdateManifestV1>
dare_update::load_desired_manifest(...) -> CoreResult<UpdateManifestV2>  // nome final no Blueprint
dare_update::classify_path(root, path, expected_sha, meta) -> CoreResult<AssetUpdateStatus>
dare_update::plan_update(root, opts: &UpdatePlanOptions) -> CoreResult<UpdatePlan>

UpdatePlanOptions { target: Option<HarnessId>, /* no apply flags here */ }
```

### Ambiguidades resolvidas neste Design

| Tópico | Decisão | Marcador |
|--------|---------|----------|
| `--target` | Harness id (microplano + doc mestre), **não** versão semver | 🟢 |
| Skill `/dare-update` com `--target 3.2.0` | Documentar desalinhamento; corrigir skill na implementação 021/022 | 🟡 |
| Apply sem dry-run | Stub ou exigir `--dry-run` até 022 | 🟡 Blueprint |
| Schema do manifest novo | Novo schema versionado; leitor schema 1 mantido | 🟢 (doc mestre) |
| Releases 3.9+ | Incluir no manifest novo; **não** reproduzir buraco TS | 🟢 |
| Codex | Incluir explicitamente (Classe C vs TS) | 🟢 |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo tree + opts → mesmo `UpdatePlan` (ordem, hashes) | Unit |
| RNF-02 | Performance | Dry-run tipicamente < 3 s em fixture média (sem rede) | Smoke |
| RNF-03 | Observabilidade | Tracing span por classificação; erros `CoreError` tipados | Unit |
| RNF-04 | Manutenibilidade | Lógica em `dare-update`; CLI thin; sem ciclo `dare-update`↔`dare-cli` | Clippy + dep graph |
| RNF-05 | Compatibilidade | Win/macOS/Linux paths POSIX relativos no report | CI 003 |
| RNF-06 | Idempotência de leitura | Dry-run ×2 → mesmo JSON (exceto campos de tempo se existirem — preferir **sem** timestamps no schema 1) | Unit |
| RNF-07 | Independência de 022 | Plano útil sozinho; apply pode consumir o mesmo tipo | Contrato estável |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dir` / harness id / paths do manifest antes de I/O | OWASP A03 |
| RS-02 | Não logar conteúdo completo de ficheiros (só path + hash curto); redact 004 | OWASP A02 |
| RS-03 | Leituras só sob project root; deny symlink escape | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell; sem rede no dry-run | Supply chain / 006 |
| RS-06 | Dry-run **nunca** escreve (incluindo `.dare/backup-*`) | Integrity |
| RS-07 | Cap de tamanho ao ler ficheiros para hash | Availability |
| RS-08 | Manifest malformado → erro tipado, sem panic | Robustez |
| RS-09 | Não executar conteúdo de assets como código | Injection |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Domínio | crate nova `dare-update` | `0.1.0-alpha.0` |
| CLI | `dare-cli` + clap | workspace |
| Contratos | `dare-contracts` (`UpdateManifestV1`) | 007 |
| Assets / SHA | `dare-assets` (`sha256_hex`, embed) | 009 |
| Path / FS | `dare-core` | 005 |
| Harness IDs | `dare-harness` (`UPDATE_HARNESS_IDES`) | 013/DEC-014 |
| Config (só leitura se necessário) | `dare-config` | 008 |
| JSON | serde camelCase | workspace |
| Hash | `sha2` | workspace pin |
| Saída | renderer 004 | DEC-005 |
| Testes | tempfile + fixture `customized-assets` | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem do projeto | Local | read-only (021) | In | assets instalados, hashes | CLI |
| `templates/UPDATE-MANIFEST.json` / embed | Contrato | JSON | In | schema 1 legacy | 007 |
| Manifest novo (embed/assets) | Contrato | JSON/YAML | In | desired state + appliesTo | 021 |
| `dare-assets` / `dare-harness` | In-process | API | In | hashes canónicos, IDs harness | 009–014 |
| stdout | Terminal | — | Out | human / JSON plan | CLI |
| Baseline TS 3.18.1 | Referência | — | In | UX classificação / paths | Compat |
| Rede / registry | — | — | — | **Nenhuma** neste microplano | — |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **008–014** concluídos (MUST do microplano).
- Mensagens **en-US**; docs de compatibilidade pt-BR/en conforme padrão do repo.
- `dare-update` **não** depende de `dare-cli`; deps permitidas: `dare-core`, `dare-contracts`, `dare-assets`, `dare-harness`, `dare-config` (evitar ciclo).
- Sem writes no dry-run; sem backup; sem apply; sem políticas keep/replace/ask (→ **022**).
- Sem self-update do binário (→ **053**).
- Diffs intencionais vs TS (Codex em `appliesTo`; releases 3.9+) → DEC-021 / classification matrix (**Classe C** — bugfix consciente).
- Bump de schema de `UpdatePlan` ou manifest novo exige ADR + migration note.

---

## 10. FORA DO ESCOPO (v1)

- Aplicação de updates, backup `.dare/backup-*`, rollback (→ **022**).
- Políticas `keep` \| `replace` \| `ask` (→ **022**).
- Migrations de `dare.config.json` no apply (→ **022**; domínio já em **008**).
- Flags `--force`, `-y` / non-interactive apply (→ **022**).
- `--target` como versão semver (skill desalinhada) — adiar ou renomear.
- `dare validate` DAG (→ **020**).
- `dare skill update` / registry remoto (→ **044–045**).
- Publish/CDN de assets.
- UI interativa de resolução de customized.
- Telemetria remota.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Confusão `apply` vs `customized` | Alta | Alto | Tabela Apêndice B + fixtures golden + DEC-021 |
| R-02 | Reproduzir buraco 3.9+ no manifest | Média | Médio | Checklist O-02; teste “releases contínuas” |
| R-03 | Omitir Codex (paridade com bug TS) | Média | Alto | RF-13 + teste `update_policies_include_codex` |
| R-04 | `--target` ambíguo (harness vs versão) | Alta | Médio | Design fixa harness; actualizar skill IDE |
| R-05 | Hash de ficheiros enormes | Baixa | Médio | `read_limited` + erro tipado |
| R-06 | Dry-run com side-effect acidental | Baixa | Alto | Teste before/after listing; sem chamar apply |
| R-07 | Ciclo de deps workspace | Baixa | Alto | `dare-update` só deps lib; dep-graph test |
| R-08 | Drift paths managed vs 019 install | Média | Médio | Lista fechada partilhada / gerada da matrix 010 |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-28 priorizados (`UpdatePlan`, classificação, dry-run, Codex, `--target` harness)
- [ ] Decisão RF-21 (exigir `--dry-run` vs stub apply) aceite
- [ ] Schema do manifest novo (nome + `schemaVersion`) aceite
- [ ] RS / path safety / zero-write dry-run validados
- [ ] Fora de escopo (022 apply/backup/migrations) alinhado
- [ ] Riscos R-01…R-08 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-021-update-planejamento-e-manifest.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-update/src/plan.rs` | Classificação + `UpdatePlan` |
| `crates/dare-update/src/lib.rs` | Re-exports |
| `crates/dare-update/Cargo.toml` | Nova crate |
| `Cargo.toml` (workspace) | Adicionar member + dep |
| `crates/dare-contracts/src/update_manifest.rs` | Leitor schema 1 (já existe) |
| `crates/dare-cli/src/commands/update.rs` | Wiring clap dry-run |
| `crates/dare-cli/src/main.rs` | Subcomando `update` |
| `templates/UPDATE-MANIFEST.json` / embed | Legacy schema 1 |
| `assets/` ou `templates/` (manifest novo) | Desired state versionado |
| `crates/dare-harness/src/codex.rs` | `UPDATE_HARNESS_IDES` |
| Fixtures `customized-assets` | Golden customized |
| `docs/compatibility/cli-update-plan.md` | Docs DEC (a criar) |
| `docs/DECISION-LOG.md` | DEC-021 |

## Apêndice B — Classificação `AssetUpdateStatus`

| Status | Condição (resumo) | Acção típica em 022 |
|--------|-------------------|---------------------|
| `identical` | Existe ∧ SHA == canónico | no-op |
| `missing` | Não existe | criar (apply) |
| `apply` | Existe ∧ managed ∧ SHA ≠ canónico | substituir (política replace/force) |
| `customized` | Existe ∧ SHA ≠ canónico ∧ unmanaged / editado | keep por default; replace só com `--force` |

> Detalhe exacto de “managed vs unmanaged” alinha-se aos adapters 011–014 (`<!-- dare:managed` / frontmatter). Blueprint congela a função `is_managed`.

## Apêndice C — `UpdatePlan` schema 1 (proposto)

```json
{
  "schemaVersion": 1,
  "mode": "dry-run",
  "projectRoot": "/abs",
  "target": null,
  "cliVersion": "0.1.0-alpha.0",
  "counts": {
    "identical": 0,
    "missing": 0,
    "apply": 0,
    "customized": 0
  },
  "items": [
    {
      "path": "AGENTS.md",
      "status": "missing",
      "expectedSha256": "…",
      "actualSha256": null,
      "appliesTo": ["codex", "*"]
    }
  ]
}
```

- Keys **camelCase**; sem timestamps no schema 1 (determinismo).
- `target`: `null` = all; senão string harness id.
- Paths relativos POSIX; ordenados lexicograficamente.

## Apêndice D — Exit codes (alinhamento 004)

| Situação | Exit | Notas |
|----------|------|-------|
| Dry-run OK | 0 | Mesmo com `customized` > 0 |
| `--target` inválido / args | 4 | `InvalidInput` |
| Project root não encontrado | 4 ou NotFound tipado | Documentar vs 017/018 |
| Manifest ilegível / schema unsupported | 1 ou Config tipado | Sem panic |
| Erro interno | 1 | `Internal` |

## Apêndice E — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `UpdateManifestV1` load/save | ✅ 007 |
| Fixture `UPDATE-MANIFEST.json` | ✅ mínima |
| `sha256_hex` / assets manifest | ✅ 009 |
| `UPDATE_HARNESS_IDES` + Codex | ✅ 013 / DEC-014 |
| Crate `dare-update` | 🔴 ausente |
| `plan.rs` / `UpdatePlan` | 🔴 ausente |
| CLI `dare update` | 🔴 ausente |
| Manifest novo versionado | 🔴 ausente |
| Docs update plan | 🔴 ausente |

## Apêndice F — Classification vs TypeScript 3.18.1 (rascunho)

| Tema | Classe | Notas |
|------|--------|-------|
| Schema 1 reader | A | Paridade |
| Status enum identical/missing/apply/customized | A | Paridade |
| SHA-256 detecção | A | Paridade |
| Releases 3.9+ no manifest novo | C | Bugfix consciente |
| Codex em `appliesTo` / plano | C | Bugfix consciente (TS omitia) |
| Exit codes / envelope JSON 004 | B | Alinhamento nativo |
| Apply/backup/migrations | — | Fora → 022 |

---

## Próximas etapas

1. Revisar e aprovar este Design (checklist §12).
2. Executar `/dare-blueprint` → `DARE/BLUEPRINT-021-update-planejamento-e-manifest.md`.
3. Continuar Método DARE (`/dare-tasks` → execute); apply em **022**.
