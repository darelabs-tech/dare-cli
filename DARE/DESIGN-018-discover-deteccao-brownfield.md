# DESIGN: Discover — detecção brownfield (Microplano 018)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/018-discover-deteccao-brownfield.md`  
> **Referência:** Microplanos **005** (path safety) · **007** (contratos) · **008** (config) · **009** (assets) · **011–014** (detect harness) · **004** (saída/`--json`) · baseline TS 3.18.1  
> **Posição:** 18 de 56  
> **Arquivo:** `DARE/DESIGN-018-discover-deteccao-brownfield.md` (não substitui Designs 001–017)  
> **Nota:** Este Design cobre **apenas** detecção determinística + `dare discover --check` (zero writes). Instalação idempotente (`dare discover` sem `--check`) fica no microplano **019**.

---

## 1. DESCRIÇÃO

Este Design cobre a **detecção brownfield** do DARE CLI nativo em Rust: localizar o project root e o Git root, identificar stacks por manifests/arquivos, reconhecer monorepo, detectar harnesses IDE já presentes e emitir um `DetectionReport` estável. O problema: sem um `--check` read-only e determinístico, developers e agentes não conseguem validar a stack antes de instalar artefatos DARE — e a instalação (019) não pode começar sem um relatório confiável.

A entrega é a crate `crates/dare-project` (domínio de detecção), o comando `crates/dare-cli/src/commands/discover.rs` com superfície `dare discover --check` (`--dir`/`-d`, `--json`), fixtures Node/Rust/Python/monorepo, e documentação de compatibilidade vs TypeScript 3.18.1. Quem consome são developers, agentes IDE (`/dare-discover`) e o microplano 019 (que transforma o relatório em `InstallPlan`).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Project root | Walk-up encontra markers (`dare.config.json`, `DARE/`, manifests de stack) | Unit + fixtures |
| O-02 | Git root | Detecta `.git` (dir/file) e/ou `git rev-parse --show-toplevel` via argv seguro | Unit |
| O-03 | Stacks | Detecta Node, Rust e Python pelos manifests canônicos | 3/3 fixtures |
| O-04 | Monorepo | Flag + evidências em fixture `monorepo` | Unit |
| O-05 | Harnesses | Reporta Claude/Cursor/Codex/Antigravity via `dare-harness` | Unit |
| O-06 | DetectionReport | Schema JSON `schemaVersion` = **1**, camelCase, campos congelados | Assert eq |
| O-07 | `--check` zero writes | Snapshot before/after do filesystem do projeto idêntico | Unit |
| O-08 | Conflitos de stack | Relatório inclui `conflicts` quando ≥2 stacks primárias competem | Unit |
| O-09 | Determinismo | Mesmo tree → mesmo JSON (ordem de stacks/harnesses estável) | Golden/unit |
| O-10 | Ralph Loop | `cargo fmt --check`, `clippy`, `test`, `audit`/`deny` | Exit 0 |
| O-11 | Docs DEC | `docs/compatibility/cli-discover-check.md` + diffs classificados | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Primeiro valor brownfield no Ciclo 1 (alpha) |
| Tech Lead | Time DARE CLI Rust | Crate `dare-project`; schema 1 estável; sem ciclos de deps |
| Engenheiro CLI | Time implementação | Detecção + wiring clap + fixtures |
| Usuário Final | Devs / agentes | `dare discover --check` / `--json` antes de instalar |
| CI | Pipelines | Smoke read-only sem side effects |
| Compatibilidade | Tech Lead | Diff vs TS 3.18.1 classificado (A/B/C/D) |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-project` | MUST | Membro do workspace; `dare-cli` depende dela; sem ciclo (cli ↛ domain ↛ cli) |
| RF-02 | `dare discover --check` | MUST | Exit 0 em projeto válido; imprime human ou JSON; **zero** create/write/delete no tree |
| RF-03 | `--dir` / `-d <path>` | MUST | Usa path como start do walk (equivalente TS); default = cwd |
| RF-04 | Localizar project root | MUST | Walk-up desde start até marker; se nenhum → report com `projectRoot: null` e stacks vazias (não panic) |
| RF-05 | Localizar Git root | MUST | `gitRoot` preenchido quando `.git` encontrado no walk ou via `git rev-parse`; senão `null` |
| RF-06 | Detectar stack Node | MUST | Marker `package.json` (e evidências opcionais: `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`) |
| RF-07 | Detectar stack Rust | MUST | Marker `Cargo.toml` |
| RF-08 | Detectar stack Python | MUST | Markers: `pyproject.toml` **ou** `requirements.txt` **ou** `setup.py` |
| RF-09 | Conflitos de stack | MUST | Se ≥2 famílias primárias (node/rust/python/…) → `conflicts: [{kinds, evidence}]` não vazio; stacks ainda listadas |
| RF-10 | Detectar monorepo | MUST | `monorepo: true` se evidência: `pnpm-workspace.yaml`, `lerna.json`, `nx.json`, `Cargo.toml` com `[workspace]`, ou ≥2 manifests filhos sob depth limitado |
| RF-11 | Detectar harnesses | MUST | Usa `detect_claude` / `detect_cursor` / `detect_codex` / `detect_antigravity`; lista estável por id |
| RF-12 | Produzir `DetectionReport` | MUST | Struct serializável; `schemaVersion: 1`; ver Apêndice C |
| RF-13 | Saída human en-US | MUST | Resumo: root, git, stacks, conflicts, monorepo, harnesses; linha explícita `mode: check (zero mutations)` |
| RF-14 | `--json` estável | MUST | Envelope 004 + `data` camelCase schema 1 |
| RF-15 | `dare discover` sem `--check` | MUST | Exit ≠ 0 com erro tipado “installation not implemented in this build / see microplano 019” — **não** escreve nada |
| RF-16 | Fixtures | MUST | `existing-node-project`, `existing-rust-project`, `existing-python-project`, `monorepo` sob `tests/fixtures/` (ou path canônico do repo) |
| RF-17 | Exit codes tipados | MUST | Documentados antes do happy path: 0 = check ok; 2 = usage/args; 3 = path inválido/escape; 4 = I/O; 5 = install não disponível (sem `--check`) |
| RF-18 | Ordenação determinística | MUST | `stacks[]` e `harnesses[]` ordenados por `id` lexicográfico; `evidence[]` ordenada |
| RF-19 | Stacks finas (Nest/Laravel/…) | SHOULD | Heurísticas adicionais (ex.: `nest-cli.json`, `artisan`, `manage.py`) como `id` secundário sem quebrar schema 1 |
| RF-20 | Go / PHP / outros | COULD | Markers `go.mod` / `composer.json` no mesmo schema; documentar se incluídos |
| RF-21 | Docs DEC | MUST | `docs/compatibility/cli-discover-check.md` + entradas na classification matrix se diff vs TS |
| RF-22 | Smoke CLI | MUST | `dare discover --check` e `--check --json` em tempdir com fixture |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Markers de project root (walk-up)

| Marker | Tipo | Nota |
|--------|------|------|
| `dare.config.json` | file | Projeto já DARE |
| `DARE/` | dir | Metodologia presente |
| `package.json` | file | Node |
| `Cargo.toml` | file | Rust |
| `pyproject.toml` / `requirements.txt` / `setup.py` | file | Python |
| `go.mod` / `composer.json` | file | COULD / SHOULD se RF-20 |

### Superfície CLI

```text
dare discover [--dir|-d <path>] --check   # detecção only (este microplano)
dare discover [--dir|-d <path>]           # MUST falhar tipado até 019
# + --json / --no-color globais (004)
```

### API de domínio (esboço)

```text
dare_project::detect(start: &Path) -> CoreResult<DetectionReport>
dare_project::format_human(r: &DetectionReport) -> String
dare_project::report_to_json(r: &DetectionReport) -> Value
DETECTION_SCHEMA_VERSION: u32 = 1
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo filesystem → mesmo JSON byte-a-byte (após normalizar paths absolutos se necessário) | Golden/unit |
| RNF-02 | Performance | Check tipicamente < 500 ms em repo local médio (depth limitado; sem scan recursivo ilimitado) | Smoke informal |
| RNF-03 | Disponibilidade | Funciona sem Git e sem DARE (relatório parcial) | Unit |
| RNF-04 | Observabilidade | Erros tipados via `thiserror`/`CoreError`; sem panic em paths ausentes | Unit |
| RNF-05 | Manutenibilidade | Lógica em `dare-project`; `discover.rs` thin (clap + render) | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths; fixtures cross-platform | CI 003 |
| RNF-07 | Limites | Depth máx. de scan monorepo documentado (ex.: 3 níveis); max entries | Const + teste |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dir` (existência, normalização); rejeitar escape / paths perigosos via `ProjectRoot` / path safety 005 | OWASP A03 |
| RS-02 | Não dumpar conteúdo de manifests além de IDs/evidências de path relativo; sem secrets de `.env` / tokens | OWASP A02 |
| RS-03 | `--check` e discover sem `--check` (stub) são **read-only** — zero writes | Integrity |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; Git via argv separado (`dare-core` process), nunca shell concatenado | Supply chain / 006 |
| RS-06 | Leituras sob jail do project root quando root resolvido; symlinks/junctions conforme política 005 | Path safety |
| RS-07 | Mensagens de erro sem vazar home paths desnecessários além do root reportado | Privacy |
| RS-08 | Limitar tamanho de leitura de manifests (cap de bytes) para evitar DoS local | Availability |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate nova | `dare-project` | `0.1.0-alpha.0` |
| CLI | `dare-cli` + clap | workspace |
| Path / FS | `dare-core` (`ProjectRoot`, `SafeRelativePath`) | 005 |
| Processos | `dare-core` process (Git argv) | 006 |
| Harnesses | `dare-harness` detect_* | 011–014 |
| JSON | serde / serde_json camelCase | workspace |
| Saída | renderer 004 (`--json`) | DEC-005 |
| Testes | tempfile + fixtures + smoke | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem do projeto | Local | read | In | manifests, dirs harness, `.git` | CLI |
| Git CLI (opcional) | Processo | argv | In | toplevel path | dare-core process |
| `dare-harness` | In-process | API | In | Detect structs | adapters 011–014 |
| stdout | Terminal | — | Out | human / JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | UX / campos / exit | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **005, 007, 008 e 009** concluídos (MUST do microplano); harness detect **011–014** disponíveis para RF-11.
- Mensagens **en-US**.
- **Zero writes** em `--check` — critério inegociável.
- Bump de `schemaVersion` exige ADR + nota de migration.
- Sem instalação de arquivos DARE / harnesses neste ciclo (019).
- Sem `dare init` / scaffold greenfield.
- Sem alteração de contratos de disco existentes sem ADR.
- Dependência: `dare-project` → `dare-core` (+ `dare-harness` para detect); **não** depende de `dare-cli`.

---

## 10. FORA DO ESCOPO (v1)

- Instalação idempotente: `dare.config.json`, `DARE/`, `.dare/`, templates, graph, adapters (→ **019**).
- Capability materialization `dare-discover` nos harnesses (→ **019**).
- `dare reverse` / `dna` / `migrate` / `patterns` (036+).
- Scaffold `dare init` / `bootstrap` (ciclos posteriores).
- GraphRAG / AST profundo / tree-sitter.
- Telemetria remota.
- Resolução interativa de conflitos de stack (só reportar).
- Correção automática de mojibake/`dare new` (welcome) — fora deste Design.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Heurística monorepo falso-positivo | Média | Médio | Evidências explícitas + depth cap; testes negativos |
| R-02 | Conflito Node+Rust em repo misto legítimo | Alta | Médio | Reportar conflict sem falhar exit 0 no `--check`; 019 decide política |
| R-03 | Diff de campos vs TS 3.18.1 | Alta | Médio | DEC + classification matrix antes do freeze schema 1 |
| R-04 | Walk-up escolhe root errado em nested packages | Média | Alto | Preferir markers DARE; documentar ordem de markers; fixture monorepo |
| R-05 | `git` ausente no PATH | Média | Baixo | Fallback só `.git`; `gitRoot: null` ok |
| R-06 | Scan profundo lento / infinito | Baixa | Alto | Depth + max entries (RNF-07); sem follow symlink escape |
| R-07 | Instalação acidental se `--check` esquecido | Média | Alto | RF-15 MUST falha tipada até 019 |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-22 priorizados (`--check` read-only, schema 1, fixtures, exit codes)
- [ ] Separação 018 (detect) vs 019 (install) aceite
- [ ] Markers de stack e monorepo aceites
- [ ] RS / path safety / argv Git validados
- [ ] Fora de escopo (install, reverse, init) alinhado
- [ ] Riscos R-01…R-07 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-018-discover-deteccao-brownfield.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-project/` | Nova crate — detecção |
| `crates/dare-project/src/detect.rs` | Root, stacks, monorepo, report |
| `crates/dare-project/src/lib.rs` | API pública |
| `crates/dare-cli/src/commands/discover.rs` | Wiring clap + human/JSON |
| `crates/dare-cli/src/main.rs` | `Commands::Discover` |
| `tests/fixtures/existing-node-project/` | Fixture Node |
| `tests/fixtures/existing-rust-project/` | Fixture Rust |
| `tests/fixtures/existing-python-project/` | Fixture Python |
| `tests/fixtures/monorepo/` | Fixture monorepo |
| `docs/compatibility/cli-discover-check.md` | Docs DEC (a criar) |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| Crate `dare-project` | 🔴 ausente |
| `Commands::Discover` / `discover.rs` | 🔴 ausente |
| Detect harnesses (011–014) | ✅ APIs `detect_*` existem |
| Path safety / process | ✅ 005/006 |
| Fixtures discover | 🔴 a materializar |
| Docs `cli-discover-check.md` | 🔴 ausente |

## Apêndice C — Campos JSON schema 1 (congelados)

```json
{
  "schemaVersion": 1,
  "mode": "check",
  "projectRoot": "/abs/or/null",
  "gitRoot": "/abs/or/null",
  "stacks": [
    {
      "id": "node",
      "family": "node",
      "confidence": "high",
      "evidence": ["package.json"]
    }
  ],
  "conflicts": [],
  "monorepo": false,
  "monorepoEvidence": [],
  "harnesses": [
    {
      "id": "claude",
      "present": false,
      "evidence": []
    }
  ],
  "dareAlreadyPresent": false
}
```

Notas:
- `dareAlreadyPresent` = `dare.config.json` ou `DARE/` no root resolvido.
- Paths absolutos no JSON são aceitáveis no schema 1 (documentar em DEC; golden tests podem normalizar).
- `confidence`: `high` | `medium` | `low` (string enum estável).
- Campos extras exigem bump de `schemaVersion` + ADR.

## Apêndice D — Exit codes (v1)

| Code | Significado |
|------|-------------|
| 0 | `--check` concluiu; relatório emitido |
| 2 | Args/usage inválidos |
| 3 | Path/`--dir` inválido ou path safety reject |
| 4 | Falha de I/O / processo Git inesperado (quando tratado como erro) |
| 5 | `discover` sem `--check` (install ainda não implementado — 019) |

## Apêndice E — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-018-discover-deteccao-brownfield.md`.  
3. `/dare-tasks` → `mp018-*` + `dare-dag-018.yaml`.  
4. Após closeout → [`019-discover-instalacao-do-dare.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/019-discover-instalacao-do-dare.md).
