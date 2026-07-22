# CLI: `dare design` (Ciclo 023)

Render **determinístico** de `DARE/DESIGN.md` a partir de uma descrição posicional ou prompts interativos. Complementa [DEC-024](../DECISION-LOG.md) e o Blueprint 023. **Sem rede, sem LLM neste ciclo.**

Source: `crates/dare-cli/src/commands/design.rs`

## Command

```bash
dare design <description...>
dare design --interactive
dare design "My API for user onboarding" --json
```

| Flag / arg | Efeito |
|------------|--------|
| `<description...>` | Argumento posicional trailing; clap junta tokens com espaço. Obrigatório quando **não** `--interactive`. |
| `--interactive` | Prompts TTY (title + description); omitir descrição posicional. |
| `--json` | Envelope JSON (004); `data` = `DesignReport` schema 1 |
| `--no-color` | Sem ANSI (global 004) |

**Sem `--ai` neste ciclo** — enrichment por IA → microplano **024** (`dare-ai`).

### Regras de uso

| Condição | Resultado |
|----------|-----------|
| `!interactive && description vazio` | Usage exit **2** — `"description required (or --interactive)"` |
| `interactive && !description.is_empty()` | Usage exit **2** — `"cannot combine --interactive with description"` |
| `interactive && !stdin.is_terminal()` | Usage exit **2** — `"design --interactive requires a TTY"` |
| `description.trim().is_empty()` (após join) | InvalidInput exit **4** |
| `description.len() > 32768` | InvalidInput exit **4** (`DESC_MAX`) |
| project root não encontrado | InvalidInput exit **4** |
| markers AGENT malformados (BEGIN sem END) | InvalidInput exit **4** — `"malformed AGENT markers in DARE/DESIGN.md"` |
| falha de I/O em `atomic_write` | Io exit **5** |

## Path de disco (único)

| Path | Semântica |
|------|-----------|
| `DARE/DESIGN.md` | **Único** output deste comando (`DESIGN_REL`). Escrita via `atomic_write` sob `ProjectRoot`. |

Path alternativo de design (ex.: `DARE/DESIGN-*.md`) → **fora de escopo**; microplano **025**.

Leitura de ficheiro existente: cap `DESIGN_READ_CAP` = 262 144 bytes (RS-06).

## Markers ENRICHABLE (AGENT)

Delimitação parseável para merge preserve e injeção futura (024):

```text
<!-- AGENT:BEGIN section="<id>" -->
<body>
<!-- AGENT:END section="<id>" -->
```

| `section` id | Secção do template | Conteúdo inicial (023) |
|--------------|-------------------|------------------------|
| `description` | `## 1. DESCRIÇÃO` | Texto do user |
| `objectives` | `## 2. OBJETIVOS E MÉTRICAS DE SUCESSO` | Tabela stub `[A definir]` |
| `functional-requirements` | `## 4. REQUISITOS FUNCIONAIS` | Tabela stub `[A definir]` |
| `stack` | `## 7. STACK TÉCNICA` | Tabela stub `[A definir]` |

Constantes: `MARKER_BEGIN = <!-- AGENT:BEGIN section="`; `MARKER_END_PREFIX = <!-- AGENT:END section="`.

Secções **sem** marker (stakeholders, NFR, segurança, integrações, restrições, fora de escopo, riscos, checklist) são reescritas pelo template canónico em create; em update com markers existentes, texto fora de BEGIN/END permanece intacto.

## Preserve / appendix

Algoritmo `merge_preserve`:

1. **Create** (`DESIGN.md` ausente ou vazio): escreve markdown canónico completo; `action = created`.
2. **Update com markers**: substitui apenas blocos BEGIN/END de ids `ENRICHABLE`; parágrafos/tabelas unmanaged entre markers sobrevivem; `action = updated`.
3. **Update sem markers** (ficheiro user legado): escreve `fresh` e **anexa** appendix:

```markdown
## APPENDIX — Preserved previous content

<!-- dare:preserved -->
{existing}
```

   `preservedRegions = 1`; conteúdo anterior intacto sob `dare:preserved`.

CLI alpha: **sempre** merge-preserve no path feliz (`force_full_rewrite` só em testes internos — não exposto).

## Interactive (TTY)

Prompts en-US, ordem congelada:

1. `Title (empty = derive from description): `
2. `Description: ` (single line alpha)

Requer stdin TTY (`std::io::IsTerminal`). Pipe ou CI sem TTY + `--interactive` → Usage exit **2**.

## Exit codes

| Code | Quando |
|------|--------|
| 0 | Sucesso; `DesignReport.ok == true`; ficheiro escrito |
| 2 | Usage (clap; regras acima; `--interactive` sem TTY) |
| 4 | InvalidInput (root null, descrição vazia/oversize, path jail, markers malformados) |
| 5 | Io (falha read/write) |

Exit **1** (Internal) reservado a falhas não previstas; **3** (NotFound) reservado a `--dir` futuro — não exposto neste ciclo.

## DesignReport schema 1 (congelado)

Campos camelCase em `--json`:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `mode` | string | Always `"design"` |
| `ok` | bool | `true` on success |
| `path` | string | `"DARE/DESIGN.md"` (POSIX relativo) |
| `action` | string | `"created"` \| `"updated"` |
| `title` | string | Título usado no header |
| `markerCount` | number | Pares BEGIN/END escritos (esperado: 4) |
| `preservedRegions` | number | Blocos unmanaged preservados (0 ou ≥1) |
| `interactive` | bool | Eco da flag CLI |
| `warnings` | string[] | ex.: title truncated to 60 chars |

Bump requer ADR + migration note.

### Human output (exemplo)

```text
design: ok
path: DARE/DESIGN.md
action: created
title: My API
markerCount: 4
preservedRegions: 0
mode: design
```

## Fixtures / snapshots

Diretório: `tests/fixtures/design/`

| Ficheiro | Uso |
|----------|-----|
| `input-basic.txt` | Descrição fixa para golden tests |
| `golden-basic.md` | Estrutura esperada; data normalizada (`1970-01-01` em unit) |
| `existing-with-notes.md` | Fixture preserve — notes unmanaged sobrevivem após regenerate |

Testes:

```bash
cargo test -p dare-cli -- design
cargo test -p dare-cli --test cli_smoke -- design
```

Smokes MUST: `design_creates_file`, `design_json_schema`, `design_empty_desc_usage_or_4`, `design_preserve_notes`, `design_interactive_no_tty_exits_2`.

## Fora de escopo (023)

| Item | Microplano |
|------|------------|
| `--ai` / `--provider` / enrichment LLM | **024** |
| Injeção de conteúdo IA nos markers | **024** (023 só coloca markers + preserve) |
| `dare blueprint` / path alternativo de design | **025** |
| `--force` full rewrite na superfície CLI | Não exposto (preserve sempre) |

## Segurança / contratos

- Path jail (`ProjectRoot` / `SafeRelativePath`) — RS-01
- `atomic_write` sob project root — RS-03
- `DESC_MAX` 32 768 + `DESIGN_READ_CAP` 262 144 — RS-06
- Markers comment-only (HTML comments) — RS-07
- Sem shell, sem rede — RS-05
- Mensagens de erro ≤200 chars; sem dump de descrição longa — RS-02

## Diff vs TypeScript `@dewtech/dare-cli@3.18.1`

Paridade **parcial determinística**: o TS baseline pode invocar LLM e emitir markdown ad-hoc; o rewrite nativo 023 gera template canónico fixo + markers AGENT, sem IA.

| Item | TS 3.18.1 | Native 023 | Classificação |
|------|-----------|------------|---------------|
| Geração de conteúdo | LLM / heurística variável | Template embed determinístico | **C** — SoT nativo congelado (DEC-024) |
| Markers `AGENT:BEGIN/END` | Ausente ou ad-hoc | 4 secções ENRICHABLE fixas | **C** — preparação 024 |
| Merge preserve / `dare:preserved` | Comportamento histórico opaco | Algoritmo §5.3 Blueprint | **B** — melhoria observável |
| Path output | Variável / cwd-dependent | Sempre `DARE/DESIGN.md` | **A** — alinhado RF-09 |
| `--interactive` sem TTY | Indefinido | Usage exit **2** | **B** — CI-safe (T-06) |
| `--ai` | Presente no TS | Ausente (024) | **C** — escopo deferido |
| `DesignReport` JSON | Ad-hoc / ausente | `schemaVersion: 1` camelCase | **C** — ADR-002 envelope |
| Exit codes | Mapa histórico TS | Congelado 004 (0/2/4/5 neste ciclo) | **B** |

Snapshots nativos (`golden-basic.md`, smokes) são **SoT alpha** para regressão — não reproduzir variabilidade LLM do TS.

## Local verify

```bash
docker compose -f docker-compose.ci.yml config
cargo test -p dare-cli -- design
cargo test -p dare-cli --test cli_smoke -- design
```

`docker compose -f docker-compose.ci.yml config` exit **0** verificado em **mp023-001** (Fase 1). Compose CI reutilizado (sem imagem nova) — herança microplanos 003/015.

**Waiver:** se Docker não estiver instalado localmente, a verificação compose pode ser omitida; CI continua a ser gate.

## Related

- **DEC-024** — [`docs/DECISION-LOG.md`](../DECISION-LOG.md)
- Output envelope: [`cli-output-and-errors.md`](cli-output-and-errors.md)
- Path safety: [`path-safety.md`](path-safety.md)
- Capability: `dare-design` em [`capabilities-canonical.md`](capabilities-canonical.md)
- Template SoT: `assets/templates/DESIGN-template.md`
