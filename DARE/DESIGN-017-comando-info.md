# DESIGN: Comando info (Microplano 017)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/017-comando-info.md`  
> **Referência:** Microplanos **007–015** (contratos, config, assets, release) · **004** (saída/`--json`) · **005** (path safety) · **009** (assets verify) · baseline TS 3.18.1  
> **Posição:** 17 de 56  
> **Arquivo:** `DARE/DESIGN-017-comando-info.md` (não substitui Designs 001–016)  
> **Nota:** Existe implementação parcial em `dare-cli::commands::info` (`collect_info` / `format_human` / `report_to_json`, `schema_version=1`). Este Design congela o contrato MUST (read-only, versão/plataforma, project root, assets, backend/grafo, progresso de tasks, JSON estável) e lista gaps (smoke CLI, docs DEC, heurística TASKS, formal closeout).

---

## 1. DESCRIÇÃO

Este Design cobre o comando **`dare info`** — diagnóstico **read-only** da instalação do CLI e do projeto atual: versão, plataforma, project root, integridade dos assets embutidos, presença de `dare.config.json` / `DARE/` / `.dare/state.json`, path do grafo, backend/IDE e progresso aproximado de tasks. O problema: sem um comando único e seguro (zero mutações), developers e CI não conseguem inspecionar rapidamente se o DARE está instalado e coerente no working tree.

A entrega é a API em `crates/dare-cli/src/commands/info.rs`, wiring CLI (`dare info [--root]`, `--json` global), testes unitários de root walk + schema + read-only, smoke CLI, e documentação de compatibilidade. Quem consome são developers, agentes IDE (`/dare-info`) e pipelines.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Versão + plataforma | JSON/human incluem `version` e `platform.{os,arch,family}` | Unit |
| O-02 | Project root | Walk-up detecta `dare.config.json` / `DARE/` / `Cargo.toml` | Unit |
| O-03 | Integridade assets | `assetsOk` via `verify_embedded_assets` | Unit + smoke |
| O-04 | Backend + grafo | Lê `ide`/`backend` do config; path `dare-graph.yml` ou `DARE/dare-graph.yml` | Unit |
| O-05 | Progresso tasks | Contagem done/pending a partir de `TASKS.md` (ou `TASKS-*.md`) | Unit |
| O-06 | JSON estável | `schemaVersion` = **1** (camelCase); campos congelados | Assert eq |
| O-07 | Zero mutações | `collect_info` não cria/altera ficheiros no cwd | Unit before/after |
| O-08 | Projeto sem DARE | Diagnóstico útil (`projectRoot` null ou parcial; assets ainda verificáveis) | Unit |
| O-09 | Ralph Loop | fmt / clippy / test / audit / deny | Exit 0 |
| O-10 | Docs | `docs/compatibility/cli-info.md` + DEC | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Diagnóstico confiável alpha |
| Tech Lead | Time DARE CLI Rust | Schema JSON estável; read-only |
| Engenheiro CLI | Time implementação | `info.rs` + path safety |
| Usuário Final | Devs / agentes | `dare info` / `--json` |
| CI | Pipelines | Smoke sem side effects |
| Compatibilidade | Tech Lead | Diff vs TS classificada |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `dare info` | MUST | Exit 0; human com versão, plataforma, project, assets, mode read-only |
| RF-02 | Versão CLI | MUST | `version` = `CARGO_PKG_VERSION` |
| RF-03 | Plataforma | MUST | `platform.os/arch/family` via `std::env::consts` |
| RF-04 | Detectar project root | MUST | Walk-up desde cwd/`--root` até marker; senão `projectRoot: null` |
| RF-05 | Integridade assets | MUST | `assetsOk` true/false; `assetsError` se falha |
| RF-06 | Config / DARE / state | MUST | Flags booleanas `configPresent`, `dareDirPresent`, `statePresent` |
| RF-07 | Grafo | MUST | `graphPresent` + `graphPath` se `dare-graph.yml` ou `DARE/dare-graph.yml` |
| RF-08 | Backend/IDE | MUST | Lê `ide` **ou** `backend` string de `dare.config.json` via `ProjectRoot` |
| RF-09 | Progresso tasks | MUST | `tasks.{source,done,pending,totalMarked}` a partir de TASKS |
| RF-10 | `--json` estável | MUST | Envelope 004 + data com `schemaVersion: 1` camelCase |
| RF-11 | Zero mutações | MUST | Nenhum create/write/delete no filesystem do projeto |
| RF-12 | `--root <path>` | MUST | Usa path como start do walk (clap Option) |
| RF-13 | Human legível en-US | MUST | `format_human` inclui linha `mode: read-only (zero mutations)` |
| RF-14 | Smoke CLI | MUST | `dare info` e `dare info --json` em tempdir |
| RF-15 | Docs DEC | MUST | `docs/compatibility/cli-info.md` |
| RF-16 | Contagem TASKS robusta | SHOULD | Preferir `DARE/TASKS.md`; senão primeiro `TASKS-*.md`; documentar heurística emoji/DONE |
| RF-17 | Ordenação determinística de TASKS-* | SHOULD | Se vários `TASKS-*.md`, escolher lexicograficamente estável |
| RF-18 | Campos extras futuros | COULD | Só com bump `schemaVersion` + ADR |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contratos de disco (read-only)

| Path | Uso |
|------|-----|
| `dare.config.json` | Marker root + `ide`/`backend` |
| `DARE/` | Marker root + `TASKS.md` / `TASKS-*.md` |
| `.dare/state.json` | Presença apenas |
| `dare-graph.yml` ou `DARE/dare-graph.yml` | Path do grafo |
| `Cargo.toml` | Marker root (brownfield Rust) |

### Superfície CLI

```text
dare info [--root <path>]   # + --json / --no-color globais

collect_info(cwd: &Path) -> CoreResult<InfoReport>
format_human(r: &InfoReport) -> String
report_to_json(r: &InfoReport) -> Value
INFO_SCHEMA_VERSION: u32 = 1
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | JSON camelCase estável; ordem de campos via serde | Schema 1 |
| RNF-02 | Performance | Tipicamente < 200 ms em repo local | Smoke informal |
| RNF-03 | Disponibilidade | Funciona sem `DARE/` (diagnóstico parcial) | Unit |
| RNF-04 | Observabilidade | Erros assets em `assetsError` (sem panic) | Unit |
| RNF-05 | Manutenibilidade | Lógica em `info.rs`; main thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths | CI 003 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--root` via path normal; walk não segue escapes perigosos além de markers | OWASP A03 |
| RS-02 | Não imprimir secrets de config (só `ide`/`backend` string curta); sem dump completo do JSON | OWASP A02 |
| RS-03 | Read-only — sem writes (ownership N/A) | Integrity |
| RS-04 | `cargo audit` + `cargo deny` | OWASP A06 |
| RS-05 | Sem secrets em código | Supply chain |
| RS-06 | Leituras de config/grafo via `ProjectRoot`/`SafeRelativePath` quando aplicável | Path safety 005 |
| RS-07 | Mensagens de erro sem paths sensíveis desnecessários além do root reportado | Privacy |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-cli` | `0.1.0-alpha.0` |
| Path | `dare-core` ProjectRoot / SafeRelativePath | workspace |
| Assets | `dare-assets::verify_embedded_assets` | 009 |
| JSON | serde / serde_json camelCase | workspace |
| Saída | renderer 004 (`--json`) | DEC-005 |
| Testes | tempfile + unit + smoke | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem projeto | Local | read | In | config, TASKS, graph, state | CLI |
| Assets embutidos | Embed | in-process | In | manifest verify | dare-assets |
| stdout | Terminal | — | Out | human/JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | UX/campos | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **007–015** (contratos, config, assets, release) + **004/005/009** implicitamente.
- Mensagens **en-US**.
- **Zero writes** — critério de aceite inegociável.
- Bump de `schemaVersion` exige ADR + nota de migration.
- Sem git commit automático.
- Implementação parcial: **alinhar gaps**, não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- `dare discover` brownfield (018+).
- Mutação / repair / `dare update` (021+).
- GraphRAG query profunda (040+).
- Dashboard/UI (051).
- Contagem formal via `dare-dag.yaml` state store (026+) — v1 = heurística TASKS.md.
- Telemetria remota.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Heurística ✅/DONE conta duplo | Alta | Médio | Documentar; SHOULD refinar contagem; testes com fixture |
| R-02 | Vários `TASKS-*.md` ordem não-determinística | Média | Baixo | Sort lexicográfico (RF-17) |
| R-03 | Path absoluto no JSON em CI | Média | Baixo | Aceitável no schema 1; documentar |
| R-04 | Diff vs TS sem classificação | Média | Médio | DEC + classification matrix entry |
| R-05 | Smoke CLI ausente | Alta | Médio | RF-14 MUST neste ciclo |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-18 priorizados (read-only, schema 1, root, assets, tasks)
- [ ] Contratos de disco read-only aceite
- [ ] Heurística TASKS documentável
- [ ] RS validados
- [ ] Fora de escopo 018+/026 aceite
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-017-comando-info.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-cli/src/commands/info.rs` | Domínio info |
| `crates/dare-cli/src/main.rs` | `Commands::Info` |
| `crates/dare-cli/tests/cli_smoke.rs` | Smoke (a adicionar) |
| `docs/compatibility/cli-info.md` | Docs DEC (a criar) |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `collect_info` / human / JSON schema 1 | ✅ parcial |
| Root walk + read-only unit | ✅ |
| CLI `dare info` + `--root` | ✅ parcial |
| Smoke CLI info | 🔴 ausente |
| Docs `cli-info.md` / DEC | 🔴 ausente |
| Sort estável TASKS-* | ⚠️ gap SHOULD |
| TASKS formal / Ralph closeout | ⚠️ pendente |

## Apêndice C — Campos JSON schema 1 (congelados)

```json
{
  "schemaVersion": 1,
  "version": "0.1.0-alpha.0",
  "platform": { "os": "windows", "arch": "x86_64", "family": "windows" },
  "projectRoot": null,
  "assetsOk": true,
  "assetsError": null,
  "configPresent": false,
  "graphPath": null,
  "graphPresent": false,
  "backend": null,
  "tasks": { "source": null, "done": 0, "pending": 0, "totalMarked": 0 },
  "dareDirPresent": false,
  "statePresent": false
}
```

## Apêndice D — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-017-comando-info.md`.  
3. `/dare-tasks` → `mp017-*`.  
4. Após closeout → [`018-discover-deteccao-brownfield.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/018-discover-deteccao-brownfield.md).
