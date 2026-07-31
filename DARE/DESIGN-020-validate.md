# DESIGN: Validate — validação do DAG (Microplano 020)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/020-validate.md`  
> **Referência:** Microplanos **004** (saída/`--json`) · **005** (path safety) · **007** (contratos / `parse_dag_yaml`) · **008** (config paths) · baseline TS 3.18.1 · Documento Mestre §20  
> **Posição:** 20 de 56  
> **Arquivo:** `DARE/DESIGN-020-validate.md` (não substitui Designs 001–019)  
> **Nota:** Validação **read-only** de `DARE/dare-dag.yaml` (v2.1 + legado). Não executa tasks; não escreve disco. `dare execute` / ranks ficam em **026+**.

---

## 1. DESCRIÇÃO

Este Design cobre o comando **`dare validate`** do CLI nativo: carregar e validar o grafo de tasks (`dare-dag.yaml`) com regras determinísticas — parser v2.1 e legado, IDs únicos kebab-case, dependências/referências, ciclos, prompts/specs obrigatórios, limites e warnings — emitindo relatório human e JSON estável, com `--strict` para CI (warnings viram falha).

O problema: sem um gate local/CI, DAGs quebrados (ciclo, `depends_on` fantasma, id inválido) só falham tarde no execute. Quem consome são developers, agentes (`/dare-validate`), pre-commit e pipelines.

A entrega principal: crate **`dare-dag`** (domínio `validate`), wiring `crates/dare-cli/src/commands/validate.rs`, fixtures válidas/inválidas, docs DEC-021.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Parser v2.1 + legado | Fixtures `dare-dag.v21.yaml` / `dare-dag.legacy.yaml` (007) + inválidos | Parse Ok / Err tipado |
| O-02 | IDs únicos + kebab-case | Regex `^[a-z0-9]+(-[a-z0-9]+)*$`; duplicatas detectadas | Unit |
| O-03 | Dependências | `depends_on` → id existente; sem self-dep | Unit |
| O-04 | Ciclos | Detecta ciclo; reporta path estável | Unit |
| O-05 | Prompts / specs | `subtask_prompt` e/ou `spec_file` conforme regras MUST | Unit |
| O-06 | `--strict` | Warnings elevam falha (exit ≠ 0) | Unit + smoke |
| O-07 | Zero writes | Snapshot FS before/after idêntico | Unit |
| O-08 | Ordenação estável | Mesmos issues → mesma ordem (severity, code, path) | Golden/unit |
| O-09 | Human + JSON | Schema `ValidationReport` v1; envelope 004 | Smoke |
| O-10 | Exit codes | Mapa documentado alinhado a 004 + regra de falha de validação | Assert |
| O-11 | Ralph Loop | fmt / clippy / test / audit / deny | Exit 0 |
| O-12 | Docs DEC | `docs/compatibility/cli-validate.md` + DEC-021 | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Ciclo 2: gate de integridade do plano |
| Tech Lead | Time DARE CLI Rust | Crate `dare-dag`; reuso parse 007; sem ciclos de deps |
| Engenheiro CLI | Time implementação | clap + renderer 004 |
| Usuário Final | Devs / agentes | `dare validate` / `--strict` em CI |
| CI | Pipelines | Exit ≠ 0 bloqueia merge com DAG inválido |
| Compatibilidade | Tech Lead | Diff vs TS 3.18.1 classificado |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-dag` | MUST | Workspace member; depende de `dare-core` + `dare-contracts` (parse); **não** de `dare-cli` |
| RF-02 | `dare validate` | MUST | Default path `DARE/dare-dag.yaml` relativo ao project root (walk como info/discover 🟡 ou cwd+rel) |
| RF-03 | `--dag <path>` | MUST | Path relativo ou absoluto; lido sob path safety quando jail aplicável |
| RF-04 | `--strict` | MUST | Se houver **warnings**, exit ≠ 0 (mesmo sem errors) |
| RF-05 | Parser v2.1 | MUST | Reusa / envolve `dare_contracts::parse_dag_yaml` → `DagDocument::V21` |
| RF-06 | Parser legado | MUST | `DagDocument::Legacy` validável com mesmas regras de grafo (ids = keys) |
| RF-07 | IDs kebab-case | MUST | Cada task id casa `^[a-z0-9]+(-[a-z0-9]+)*$` senão error `invalid_id` |
| RF-08 | IDs únicos | MUST | Duplicata → error `duplicate_id` |
| RF-09 | `depends_on` refs | MUST | Ref inexistente → error `missing_dependency` |
| RF-10 | Self-dependency | MUST | `depends_on` contém próprio id → error `self_dependency` |
| RF-11 | Ciclos | MUST | Ciclo → error `cycle` com `path` de ids ordenado deterministicamente |
| RF-12 | Complexity | MUST | Valor ∈ {`LOW`,`MED`,`HIGH`} (case-sensitive MUST 🟡 alinhado fixtures); senão error ou warning — **Blueprint congela** |
| RF-13 | Title não vazio | MUST | `title.trim().is_empty()` → error `empty_title` |
| RF-14 | Prompt / spec | MUST | Em v2.1: se `subtask_prompt` **e** `spec_file` ambos vazios → error `missing_prompt_or_spec`; se `spec_file` set e ficheiro ausente → **warning** `missing_spec_file` (strict falha) |
| RF-15 | Limits | SHOULD | `parent_context_chars` / `task_output_chars` / `timeout_seconds` > 0; senão warning |
| RF-16 | `ValidationReport` | MUST | `schemaVersion: 1` camelCase; ver Apêndice C |
| RF-17 | Severidade | MUST | Issues `error` \| `warning`; códigos string estáveis (`cycle`, `duplicate_id`, …) |
| RF-18 | Ordenação | MUST | Sort por (`severity` errors first, `code` asc, `taskId` asc, `message` asc) |
| RF-19 | Saída human en-US | MUST | Resumo ok/fail; lista issues; linha `mode: validate (zero mutations)` |
| RF-20 | `--json` | MUST | Envelope 004; `data` = ValidationReport |
| RF-21 | Zero writes | MUST | Nenhuma create/update/delete no tree |
| RF-22 | Exit codes | MUST | Ver Apêndice D (domínio devolve report; CLI mapeia) |
| RF-23 | Fixtures | MUST | Válido v2.1; ciclo; missing dep; bad id; legacy válido; sob `tests/fixtures/dag/` ou contracts fixtures |
| RF-24 | Smoke CLI | MUST | validate ok; validate --strict com warning → fail; --dag missing → NotFound |
| RF-25 | Docs DEC-021 | MUST | `cli-validate.md` + DEC-021 + classification vs TS |
| RF-26 | Capability `dare-validate` | SHOULD | Já na matrix 010; não bloquear closeout se só CLI |
| RF-27 | Models block | COULD | Validar chaves conhecidas em `models` — fora MUST |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Superfície CLI

```text
dare validate [--dag <path>] [--strict]
# + --json / --no-color globais (004)
```

### API de domínio (esboço)

```text
dare_dag::validate_dag(doc: &DagDocument, opts: &ValidateOptions) -> ValidationReport
dare_dag::validate_path(root: &ProjectRoot, rel: &SafeRelativePath, opts: &ValidateOptions) -> CoreResult<ValidationReport>
  // load via dare_contracts::load_dag → validate_dag; I/O errors tipados
dare_dag::format_human(r: &ValidationReport) -> String
dare_dag::report_to_json(r: &ValidationReport) -> Value
```

`ValidateOptions { strict: bool }` — `strict` **não** muda a classificação issue→error no domínio; só a CLI (ou um helper `report_failed(r, strict)`) decide exit. Alternativa 🟡: domínio seta `ok: false` se strict && warnings — **Blueprint congela uma**.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmo YAML → mesmo JSON de issues (ordem estável) | Golden/unit |
| RNF-02 | Performance | DAG ≤ 500 tasks valida < 1 s típico | Smoke informal |
| RNF-03 | Disponibilidade | Funciona sem Git / sem harness | Unit |
| RNF-04 | Observabilidade | Erros tipados; sem panic em YAML malformado | Unit |
| RNF-05 | Manutenibilidade | Lógica em `dare-dag`; CLI thin | Clippy |
| RNF-06 | Compatibilidade | Win/macOS/Linux paths | CI 003 |
| RNF-07 | Cap leitura | Respeitar cap 007 `read_limited` | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--dag` (existência, jail `ProjectRoot` / `SafeRelativePath`) | OWASP A03 / 005 |
| RS-02 | Não ecoar conteúdo completo de prompts gigantes em erros (truncar message) | OWASP A02 / privacy |
| RS-03 | Zero writes (read-only) | Integrity |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell | Supply chain |
| RS-06 | Cap de bytes na leitura do YAML (007) | Availability |
| RS-07 | Mensagens sem vazar paths fora do project root desnecessariamente | Privacy |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate nova | `dare-dag` | `0.1.0-alpha.0` |
| Parse DAG | `dare-contracts` | 007 |
| Path / erros | `dare-core` | 004/005 |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| JSON | serde camelCase | workspace |
| Saída | OutputRenderer 004 | DEC-005 |
| Testes | tempfile + fixtures | workspace |
| Container | `Dockerfile.rust` + `docker-compose.ci.yml` | 003 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem | Local | read | In | `dare-dag.yaml` + opcional `spec_file` existence | CLI |
| stdout | Terminal | — | Out | human / JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | regras / exit / mensagens | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **004, 007, 008** (MUST do microplano).
- Mensagens **en-US**.
- Sem mutação de DAG / state / execute.
- Sem validação de `dare.config.json` profunda (008 já cobre load; fora deste comando salvo path default).
- Sem GraphRAG / refine / review.
- `dare-dag` **não** depende de `dare-cli` / `dare-project` / `dare-harness`.
- Bump de `ValidationReport.schemaVersion` exige ADR + migration note.
- Diffs vs TS → DEC-021 / classification matrix.

---

## 10. FORA DO ESCOPO (v1)

- `dare execute` / ranks / Kahn (→ **026–029**).
- `dare update` (→ **021–022**).
- `dare refine` / sub-DAG (→ **033**).
- `dare review` semântico (→ **032**).
- `dare guard` (→ **034**).
- Validação completa de conteúdo markdown das specs (só existência / não-vazio de prompt).
- UI interativa de correção.
- Telemetria remota.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Diff de exit codes vs TS | Alta | Médio | DEC-021; preferir report JSON + mapa 004 documentado |
| R-02 | Legacy map vs v2.1 regras divergentes | Média | Médio | Tabela de regras por variante no Blueprint; testes ambos |
| R-03 | Ciclo report instável (ordem DFS) | Média | Alto | Algoritmo + sort canónico do ciclo (menor id lexico como start) |
| R-04 | `spec_file` relativo ambíguo | Média | Médio | Resolver relativo a `DARE/` ou root — Blueprint congela |
| R-05 | Crate `dare-dag` vs lógica só em contracts | Baixa | Médio | Seguir microplano: crate dedicada; parse reutilizado |
| R-06 | False positive complexity case | Baixa | Baixo | Congelar enum + fixture |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-27 priorizados (parser, ciclo, strict, zero writes, schema 1)
- [ ] Localização default do DAG + resolução `spec_file` aceites (ou defer Blueprint)
- [ ] Política `--strict` / exit codes aceite
- [ ] RS / path safety / read-only validados
- [ ] Fora de escopo (execute, update, refine) alinhado
- [ ] Riscos R-01…R-06 com mitigação
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-020-validate.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-dag/` | Nova crate — validação |
| `crates/dare-dag/src/validate.rs` | Regras + report |
| `crates/dare-dag/src/lib.rs` | API pública |
| `crates/dare-cli/src/commands/validate.rs` | Wiring clap |
| `crates/dare-cli/src/main.rs` | `Commands::Validate` |
| `crates/dare-contracts/src/dag.rs` | Parse v2.1/legado (reuso) |
| `tests/fixtures/dag/` | Fixtures validate (a criar) |
| `docs/compatibility/cli-validate.md` | Docs DEC (a criar) |
| `docs/DECISION-LOG.md` | DEC-021 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `parse_dag_yaml` / load_dag | ✅ 007 `dare-contracts` |
| Fixtures v21 / legacy | ✅ contracts tests |
| Crate `dare-dag` | 🔴 ausente |
| `Commands::Validate` | 🔴 ausente |
| validate rules / cycles | 🔴 ausente |
| Docs `cli-validate.md` | 🔴 ausente |

## Apêndice C — `ValidationReport` schema 1 (proposto)

```json
{
  "schemaVersion": 1,
  "mode": "validate",
  "ok": true,
  "dagPath": "DARE/dare-dag.yaml",
  "format": "v2.1",
  "taskCount": 3,
  "errorCount": 0,
  "warningCount": 0,
  "strict": false,
  "issues": [
    {
      "severity": "error",
      "code": "cycle",
      "taskId": "task-001",
      "message": "dependency cycle detected: task-001 -> task-002 -> task-001",
      "path": ["task-001", "task-002", "task-001"]
    }
  ]
}
```

Notas:
- `format`: `"v2.1"` \| `"legacy"`.
- `ok`: `errorCount == 0` **e** (`!strict` \|\| `warningCount == 0`) — se strict aplicado no domínio; senão CLI calcula.
- `issues[]` sorted (RF-18).
- `path` opcional (só ciclos / chains).
- Campos extras → bump + ADR.

## Apêndice D — Exit codes

| Code | Quando |
|------|--------|
| 0 | `ok == true` (sem errors; warnings permitidos se !strict) |
| 1 | Validação falhou (`ok == false`) **ou** Internal grave |
| 2 | Usage / clap |
| 3 | Ficheiro `--dag` / default não encontrado |
| 4 | Path safety / input inválido |
| 5 | Io ao ler YAML |

> Emitir JSON/human **também** quando exit 1 (report com issues), via renderer de sucesso com exit override **ou** envelope ok:false — Blueprint alinha a 004 (erro tipado vs report Ok). Preferência 🟡: **Ok(report) + exit code derivado** para preservar `data` no `--json` em falhas de validação (classe B vs “só stderr”).

## Apêndice E — Códigos de issue (congelar no Blueprint)

| code | severity default | Semântica |
|------|------------------|-----------|
| `invalid_id` | error | kebab-case fail |
| `duplicate_id` | error | id repetido |
| `missing_dependency` | error | depends_on desconhecido |
| `self_dependency` | error | depende de si |
| `cycle` | error | ciclo no grafo |
| `empty_title` | error | title vazio |
| `invalid_complexity` | error | fora do enum |
| `missing_prompt_or_spec` | error | ambos vazios (v2.1) |
| `missing_spec_file` | warning | path declarado ausente |
| `invalid_limits` | warning | limits ≤ 0 |
| `parse_error` | error | YAML inválido (pode ser CoreError antes do report) |

## Apêndice F — Próximas etapas

1. Revisar e aprovar este Design (resolver 🟡: project root walk, complexity case, ok/strict no domínio vs CLI, JSON em falha).  
2. `/dare-blueprint` → `BLUEPRINT-020-validate.md`.  
3. `/dare-tasks` → `mp020-*` + `dare-dag-020.yaml`.  
4. Após closeout → [`021-update-planejamento-e-manifest.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/021-update-planejamento-e-manifest.md).
