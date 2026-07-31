# DESIGN: Refine e sub-DAG (Microplano 033)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (ciclo autorizado sem pausa humana)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/033-refine-e-sub-dag.md`  
> **Referência:** Documento Mestre §29 Ciclo 11 · skill `/dare-refine` · microplanos **020** (validate) · **026** (dare-dag) · **032** (review) · baseline TS 3.18.1  
> **Posição:** 33 de 56  
> **Arquivo:** `DARE/DESIGN-033-refine.md`  
> **Escopo deste ciclo:** `dare refine <task-id>` — score LOW/MED/HIGH/CRITICAL, proposta de split, `--apply` + `spliceSubDag`, depth≤2, anti-ciclo, preservar `parentId`/`dependsOn`, `--strict` → exit **2**, capability `dare-refine`. **Não** patterns/graph CLI/skills lifecycle. **Não** guard (**034**).

---

## 1. DESCRIÇÃO

Portar a avaliação determinística de complexidade de uma task e o splice de sub-DAG (REPLAN). A heurística vive em `dare-dag::subdag`; a superfície CLI em `crates/dare-cli/src/commands/refine.rs`. Quem consome: agentes no loop Execute/REPLAN; humanos via `/dare-refine`; CI com `--strict`. Entrega: `subdag.rs` + comando `refine` + docs + **DEC-040**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Score tipado | LOW\|MED\|HIGH\|CRITICAL | Unit |
| O-02 | Proposta de split | `proposal.subtasks[]` determinístico | Unit |
| O-03 | `spliceSubDag` | DAG válido pós-apply; rewire deps | Unit |
| O-04 | MaxDepth | depth > 2 → `MaxDepthError` | Unit |
| O-05 | Cycle | splice que cicla → `CycleError` | Unit |
| O-06 | Preserve parentId/dependsOn | state + YAML coerentes | Unit |
| O-07 | `--apply` | escreve `dare-dag.yaml` + `.dare/state.json` | Unit + smoke |
| O-08 | `--strict` HIGH/CRITICAL | exit **2** | Smoke |
| O-09 | No-op LOW/MED | exit 0; sem writes sem `--apply` | Smoke |
| O-10 | Capability + docs + DEC + matriz | `dare-refine` + `cli-refine.md` + DEC-040 + Concluído | Exit 0 |
| O-11 | Ralph | fmt + clippy -p dare-dag -p dare-cli + tests + smokes | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Tasks cabem numa conversa de agente |
| Tech Lead | splice seguro; DEC-040 |
| Engenheiro CLI | wire `Commands::Refine` additivo |
| Agentes / REPLAN | `--apply` + JSON |
| Compat | diffs classificados vs TS 3.18.1 |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `dare_dag::subdag` | MUST | Módulo público; sem ciclo crate |
| RF-02 | `assess_complexity(signals) -> Level` | MUST | LOW/MED/HIGH/CRITICAL + score + signals |
| RF-03 | Sinais | MUST | files, prompt_chars, deps, heavy keywords, DAG complexity baseline |
| RF-04 | `propose_split(task) -> Proposal` | MUST | ≥2 subtasks para HIGH/CRITICAL; ids `{id}-a`… kebab-safe |
| RF-05 | `splice_sub_dag(doc, parent, subs)` | MUST | remove parent; insere subs; rewire dependents → last child; first child herda deps do parent |
| RF-06 | Max depth **2** | MUST | `task_depth` via `parentId` no state; `MaxDepthError` se depth(child)>2 |
| RF-07 | Cycle block | MUST | pós-splice `find_cycle_path` → `CycleError` |
| RF-08 | State merge | MUST | children: `parentId=parent`, `dependsOn` do YAML; parent status `SPLIT` se existir |
| RF-09 | CLI `dare refine <task-id>` | MUST | default: report; `--split` força proposal; `--apply` persiste |
| RF-10 | `--strict` | MUST | HIGH\|CRITICAL → exit **2** (mesmo sem apply) |
| RF-11 | `--format human\|json` | MUST | default human |
| RF-12 | Task not found | MUST | NotFound exit **3** |
| RF-13 | Path-unsafe id | MUST | InvalidInput **4** |
| RF-14 | Capability | MUST | `cli_commands: ["refine"]` + README `assets/capabilities/dare-refine` |
| RF-15 | Docs | MUST | `docs/compatibility/cli-refine.md` + DEC-040 |
| RF-16 | Mensagens en-US | MUST | domínio inglês |
| RF-17 | No-op | MUST | LOW/MED sem `--apply`: exit 0, `applied=false` |
| RF-18 | Apply sem recomenda split | MUST | InvalidInput 4 salvo `--force-split` MVP: exigir `--split`+HIGH/CRITICAL ou proposal não vazia |
| RF-19 | Validate pós-apply | SHOULD | `validate_dag` sem errors (warnings OK) |
| RF-20 | Smoke | MUST | happy apply / strict exit 2 / no-op |

### Superfície CLI

```text
dare refine <task-id>
  [--split] [--apply] [--strict]
  [--format human|json]
  # globais --json / --no-color
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Tipo | Requisito |
|----|------|-----------|
| RNF-01 | Det | Mesmos sinais → mesmo score/proposal (sem timestamps no report body) |
| RNF-02 | Sec | Path jail; atomic writes; sem shell; sem secrets em signals |
| RNF-03 | UX | Exit 2 só strict HIGH/CRITICAL; usage clap também 2 — distinguir por mensagem |

---

## 6. RESTRIÇÕES E FORA DE ESCOPO

- **Não** patterns / graph CLI / skills lifecycle
- **Não** `--from-agent` / `--ai` enrich real (adiar 050; MVP sem)
- **Não** alterar COMPLEXITY_ALLOWED do validate para CRITICAL no YAML (CRITICAL só no *refine score*; YAML continua LOW\|MED\|HIGH)
- **Não** merge em main / push

---

## 7. STACK TÉCNICA

| Camada | Escolha |
|--------|---------|
| Linguagem | Rust 1.85 / edition 2021 |
| Domínio | `dare-dag` (`subdag.rs`) |
| CLI | clap 4.5 em `dare-cli` |
| Contratos | `dare-contracts` DagDocument + RuntimeStateV1 |
| Testes | unit `dare-dag` + smoke `dare-cli` |

---

## 8. SEGURANÇA (OWASP)

- A01: path jail em reads/writes DAG + state
- A03: validar task-id path-safe
- A04: FileLock em state (reusar `ensure_state` / save path)
- A09: não logar prompts inteiros nos signals (cap chars)

---

## 9. CRITÉRIOS DE ACEITE (microplano)

- [ ] Split aplicado produz DAG válido
- [ ] Cycle e MaxDepth geram erros específicos
- [ ] Strict HIGH/CRITICAL retorna 2
- [ ] `cargo fmt --check`, clippy `-p dare-dag -p dare-cli`, tests + smokes
- [ ] Diffs classificados (DEC-040)
- [ ] Matriz 033 → Concluído

---

## 10. RISCOS

| Risco | Mitigação |
|-------|-----------|
| CRITICAL vs validate LOW\|MED\|HIGH | CRITICAL só no report; YAML children LOW/MED |
| Exit 2 conflita com Usage | Mensagem `refine strict: level HIGH|CRITICAL` |
| Merge conflict main.rs | Variante Refine isolada |

---

## 11. APROVAÇÃO

Ciclo Design→Blueprint→Tasks→Execute **autorizado** pelo utilizador sem pausa humana (chat microplano 033 / worktree).
