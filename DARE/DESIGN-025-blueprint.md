# DESIGN: Comando `dare blueprint` (Microplano 025)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/025-blueprint.md`  
> **Referência:** Microplanos **020** (`dare-dag` validate) · **023** (design / markers) · **024** (`dare-ai`) · Documento Mestre §23 · baseline TS 3.18.1  
> **Posição:** 25 de 56  
> **Arquivo:** `DARE/DESIGN-025-blueprint.md`  
> **Escopo deste ciclo apenas:** comando **`dare blueprint`** que materializa artefatos de execução a partir de um Design. Tudo o que pertence a microplanos posteriores fica em **Fora do Escopo**.

---

## 1. DESCRIÇÃO

Este Design cobre o comando **`dare blueprint`** do CLI nativo: ler um Design (`DARE/DESIGN.md` ou path informado), gerar de forma **determinística** (e opcionalmente enriquecida via **024**) o **`DARE/BLUEPRINT.md`**, o **`DARE/TASKS.md`**, o **`DARE/dare-dag.yaml`** válido, e as specs em **`DARE/EXECUTION/`**, depois **validar o DAG** com a engine do 020. Suporta **`--force`** e preserva customizações sem force.

Resolve a lacuna entre Design aprovado e plano executável — hoje coberto sobretudo por skills IDE (`/dare-blueprint` + `/dare-tasks`). Quem usa: developers, agentes via capability `dare-blueprint`, e CI que exige DAG válido pós-geração.

Entrega: `crates/dare-cli/src/commands/blueprint.rs`, assets `dare-blueprint`, fixtures/goldens, docs + DEC (nº no Blueprint).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Ler Design | Default `DARE/DESIGN.md` ou path arg; jail 005 | Unit/smoke |
| O-02 | Gerar BLUEPRINT.md | Ficheiro sob `DARE/` com secções canónicas do template | Snapshot |
| O-03 | Gerar TASKS.md | Tabela de tasks alinhada ao blueprint gerado | Snapshot |
| O-04 | Gerar dare-dag.yaml | YAML v2.1 parseável; ≥2 tasks rank 0 quando aplicável | `dare_dag` validate ok |
| O-05 | Gerar EXECUTION/ | `task-*.md` (ou ids kebab) por task do DAG | Assert dirs/files |
| O-06 | Validar DAG pós-geração | `validate_path` / API 020; falha → exit ≠ 0 + sem leave parcial se política assim definir | Unit/smoke |
| O-07 | `--force` | Sobrescreve artefatos geridos | Smoke |
| O-08 | Sem `--force` | Não apaga customizações (política documentada) | Unit |
| O-09 | Determinismo | Mesmo Design → mesmas estruturas estáveis (voláteis normalizados) | Golden ×2 |
| O-10 | Capability | `dare-blueprint` nos 4 harnesses; `cli_commands` inclui `blueprint` | Matrix |
| O-11 | Report human/`--json` | Schema report documentado | Smoke |
| O-12 | Ralph + docs | fmt/clippy/test/audit/deny + `cli-blueprint.md` + DEC | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Design → execução sem só depender da IDE |
| Tech Lead | Time DARE CLI Rust | Escopo 025; não puxar execute/viz (026+) |
| Engenheiro CLI | Time implementação | `commands/blueprint.rs` |
| Usuário Final | Devs | `dare blueprint` / `--force` |
| Agentes IDE | 4 harnesses | Capability `dare-blueprint` |
| CI | Pipelines | DAG válido após generate |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `dare blueprint` | MUST | Sem path → lê `DARE/DESIGN.md` sob project root |
| RF-02 | `dare blueprint <design>` | MUST | Path relativo/absoluto sob jail; resolve ficheiro Design |
| RF-03 | Project root | MUST | `find_project_root`; ausente → InvalidInput 4 |
| RF-04 | Design ausente | MUST | NotFound 3 ou InvalidInput 4 — **congelar no Blueprint** (preferência NotFound 3) |
| RF-05 | Gerar `DARE/BLUEPRINT.md` | MUST | Conteúdo a partir do Design + template `assets/templates/BLUEPRINT-template.md` (secções MUST presentes) |
| RF-06 | Gerar `DARE/TASKS.md` | MUST | Lista/tabela de tasks derivadas do plano (heurística determinística documentada no Blueprint) |
| RF-07 | Gerar `DARE/dare-dag.yaml` | MUST | Schema v2.1 (`title`, `version`, `limits`, `models`, `tasks[]` com `id`, `title`, `depends_on`, `complexity`, `spec_file`, `subtask_prompt`) |
| RF-08 | Gerar `DARE/EXECUTION/**` | MUST | Um spec file por task; `spec_file` no DAG aponta para path relativo sob `DARE/` (padrão 020) |
| RF-09 | IDs de tasks | MUST | kebab-case únicos; prompts non-empty |
| RF-10 | Validar DAG após geração | MUST | Chamar validação **020**; `ok=false` → falha do comando (exit 1 + report validate) |
| RF-11 | `--force` | MUST | Permite sobrescrever artefatos geridos existentes |
| RF-12 | Sem `--force` | MUST | Se artefato existente for **customizado**/unmanaged segundo política — **não** sobrescrever; warn ou skip; detalhe no Blueprint (alinhar espírito update keep) |
| RF-13 | Escrita atómica | MUST | Writes sob `ProjectRoot` via 005; falha a meio não deixa DAG inválido sem report (política: all-or-nothing **ou** rollback — congelar no Blueprint) |
| RF-14 | Capability `dare-blueprint` | MUST | Outputs 4 harnesses; `cli_commands: ["blueprint"]` na matrix |
| RF-15 | Assets capability | MUST | `assets/capabilities/dare-blueprint/` coerente com 010/011–014 |
| RF-16 | Report | MUST | Human + `--json`; schema `BlueprintReport` (nº versão no Blueprint) |
| RF-17 | Exit codes | MUST | Mapa 004 (0 ok; 1 validate fail / internal; 2 usage; 3 not found; 4 invalid; 5 io) |
| RF-18 | Docs + DEC | MUST | `docs/compatibility/cli-blueprint.md` + DEC no DECISION-LOG |
| RF-19 | Caps de leitura | MUST | Cap bytes ao ler Design (ex. ≥ DESIGN_READ_CAP); reject oversize |
| RF-20 | `--ai` / `--provider` | SHOULD | Reusar **024** para enriquecer secções/markers do BLUEPRINT (não obrigatório para aceite mínimo se checklist microplano não lista AI — microplano **não** lista `--ai`; **congelar:** suporte SHOULD alinhado às flags AI partilhadas do Mestre, sem bloquear path determinístico) |
| RF-21 | Snapshots/fixtures | MUST | ≥1 fixture Design → artefatos golden (estrutura) |
| RF-22 | Templates | MUST | Usar templates embed 009 (`BLUEPRINT-template`, e templates de task/spec se existirem) |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare blueprint
dare blueprint <design-path>
dare blueprint --force
dare blueprint --ai [--provider mock|codex|…]   # SHOULD (024)
# + --json / --no-color (004)
```

### Contratos de disco (canónicos deste ciclo)

| Path | Papel |
|------|-------|
| `DARE/DESIGN.md` (ou path arg) | Input |
| `DARE/BLUEPRINT.md` | Output |
| `DARE/TASKS.md` | Output |
| `DARE/dare-dag.yaml` | Output |
| `DARE/EXECUTION/**` | Output specs |

> **Nota de naming do repo de rewrite:** artefatos `DESIGN-NNN-*` / `BLUEPRINT-NNN-*` usados neste monorepo de microplanos **não** são o contrato de disco do comando `dare blueprint` alpha — o comando escreve os nomes canónicos acima (paridade Doc Mestre / TS). Path arg permite apontar a um Design alternativo **como input**; outputs continuam nos paths canónicos salvo decisão explícita no Blueprint (🟡 default: outputs sempre canónicos sob `DARE/`).

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo Design → mesma estrutura de tasks/DAG (ordenação estável) | Golden |
| RNF-02 | Performance | Geração local tipicamente < 2 s sem `--ai` | Informal |
| RNF-03 | Offline | Path determinístico sem rede | Unit |
| RNF-04 | Observabilidade | Erros tipados; span `blueprint` | Unit |
| RNF-05 | Manutenibilidade | Lógica em `commands/blueprint.rs` (+ helpers); validação via `dare-dag` | Clippy |
| RNF-06 | Cross-platform | Paths via `SafeRelativePath` | CI 003 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar path do Design e paths de escrita sob project root | OWASP A03 / 005 |
| RS-02 | Redact em logs; não dumpar Design inteiro em tracing default | OWASP A02 / 004 |
| RS-03 | Writes atómicos; política de falha parcial documentada | 005 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; `--ai` sem API key no CLI (024) | Supply chain |
| RS-06 | Caps de bytes (Design read + artefatos gerados) | Availability |
| RS-07 | Conteúdo gerado tratado como texto; sem shell | Injection |
| RS-08 | `subtask_prompt` / specs sem interpolar env secrets | A02 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| Root / FS | `dare-project` + `dare-core` | 005/019 |
| Validate | `dare-dag` | 020 |
| AI opcional | `dare-ai` | 024 |
| Templates | `dare-assets` embed | 009 |
| Capability | matrix 010 | workspace |
| Saída | OutputRenderer | 004 |
| Testes | tempfile + fixtures | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem | Local | r/w | In/Out | Design + 4 artefatos | CLI |
| `dare-dag` validate | Lib | — | In | YAML gerado | CLI |
| Provider AI (opcional) | Processo | 024 | In/Out | Enrich blueprint | Provider |
| Baseline TS 3.18.1 | Referência | — | In | Paridade classificada | Compat |

---

## 9. RESTRIÇÕES

- Pré-requisitos: **020**, **023**, **024**.
- Paths de implementação: `commands/blueprint.rs` + `assets/capabilities/dare-blueprint`.
- Outputs canónicos sob `DARE/` (tabela §4).
- Mensagens CLI en-US.
- Sem mudar contratos sem ADR/DEC.
- Heurística de decomposição Design→tasks deve ser **determinística e documentada** (não LLM obrigatório).

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo / dono |
|------|----------------|
| `dare dag viz` / ranks runtime canvas | **026–027** |
| `dare execute --next/--complete` state store | **028–029** |
| `execute --agent` / worktrees | **030–031** |
| `dare refine` / sub-DAG | **033** |
| `dare review` | **032** |
| `dare tasks` como comando separado (se existir no TS) | Só se não for alias — **neste ciclo** a geração de TASKS/DAG/EXECUTION é responsabilidade de **`dare blueprint`** |
| Multi-output `BLUEPRINT-NNN-*` no disco do utilizador | Não no contrato canónico alpha |
| Init/bootstrap de stacks | **046–047** |
| GraphRAG / MCP / dashboard | **040+ / 051+** |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Heurística Design→tasks fraca vs skill IDE | Alta | Médio | Geração mínima válida + DEC; `--ai` SHOULD para enriquecer |
| R-02 | Sobrescrever customizações sem force | Média | Alto | Política keep; testes |
| R-03 | DAG inválido deixado no disco | Média | Alto | Validate pós-write; rollback ou fail claro |
| R-04 | Diff vs TS | Alta | Médio | Classificação + goldens nativos |
| R-05 | Escopo vazar para execute | Média | Médio | Checklist Fora do Escopo |
| R-06 | Path arg escapa jail | Baixa | Alto | SafeRelativePath / resolve 005 |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo = checklist do microplano 025 (ler Design, gerar 4 artefatos, `--force`, validate DAG, capability)
- [ ] Aceite: artefatos passam validate; sem force não sobrescreve customizações; outputs deterministas
- [ ] Fora do Escopo deixa 026+ explícitos
- [ ] RS-01…RS-08 ok
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-025-blueprint.md`

---

## Apêndice A — Paths (025)

| Path | Papel |
|------|-------|
| `crates/dare-cli/src/commands/blueprint.rs` | Comando |
| `crates/dare-cli/src/main.rs` | `Commands::Blueprint` |
| `assets/capabilities/dare-blueprint/` | Capability |
| `assets/capability-matrix.yml` | `dare-blueprint` + `cli_commands` |
| `assets/templates/BLUEPRINT-template.md` | Template |
| `tests/fixtures/blueprint/` | Goldens |
| `docs/compatibility/cli-blueprint.md` | Docs |
| `docs/DECISION-LOG.md` | DEC |

## Apêndice B — Gap atual

| Item | Estado |
|------|--------|
| Templates BLUEPRINT | ✅ assets |
| `dare-dag` validate | ✅ 020 |
| `dare-ai` | ✅ 024 |
| Capability na matrix | ✅ (verificar `cli_commands`) |
| `Commands::Blueprint` | 🔴 |
| Geração TASKS/DAG/EXECUTION | 🔴 |
| `--force` / preserve | 🔴 |
| Docs DEC | 🔴 |

## Apêndice C — Critérios de aceite (microplano)

- [ ] Artefatos passam `dare validate`  
- [ ] Sem `--force` não sobrescreve customizações  
- [ ] Outputs deterministas  
- [ ] fmt / clippy / test aprovados  
- [ ] Diferenças vs TS classificadas  
- [ ] Ralph/CI verde (pipeline 015 existente)

## Apêndice D — Próximas etapas

1. Aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-025-blueprint.md`.  
3. `/dare-tasks` → `mp025-*`.  
4. Closeout → microplano **026** (DAG parser / ranks / state).
