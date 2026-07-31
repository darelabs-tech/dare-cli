# DESIGN: Verificação avançada e bench (Microplano 049)

> **Versão:** v1.0 | **Data:** 2026-07-26 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/049-verificacao-avancada-e-bench.md`  
> **Referência:** Documento Mestre §38 Ciclo 20 · §5.5 Verificação · execute Ralph **029–030** · agent **030–031** · guard **034** · refine/subdag **033** · path/process **005/006** · baseline TS `@dewtech/dare-cli@3.18.1` · skill `dare-bench` · próximo **050**  
> **Posição:** 49 de 56  
> **Arquivo:** `DARE/DESIGN-049-verificacao-avancada-e-bench.md`  
> **Escopo deste ciclo:** estender **`dare-verify`** (pós-Ralph) · aspectos **fail-to-pass** / **anti-tamper** / **mutation** / **formal** · repair loop · **best-of-N** + Pareto · política **decay** (replan/escalate) · CLI **`dare bench`** (Fix·Rate, baseline, regressão) · flags **`dare execute --best-of|--full-mutation|--formal|--policy decay`** · docs + **DEC-050**.  
> **Não** comandos `dare ai` (**050**). **Não** dashboard/MCP (**051/052**). **Não** hooks (**048** já feito). DEC proposto: **DEC-050** (DEC-049 = hooks/steering).

---

## 1. DESCRIÇÃO

Portar e completar a **verificação avançada** do DARE CLI TypeScript no Rust: depois do Ralph Loop (build→test→lint), aplicar gates pós-Ralph (fail-to-pass, anti-tamper, mutation, formal), seleção best-of-N com Pareto, política de decay no loop de agente/execute, e o harness **`dare bench`** com fixtures, Fix·Rate e regressão de baseline.

O problema: sem esses aspectos, um task pode virar DONE com patches que quebram testes que já passavam, sem pressão de mutation/formal, e sem métrica de regressão reprodutível em CI. Quem usa: desenvolvedores, CI e o loop `--agent`/`--complete`. Entrega verificável: APIs em `dare-verify` + CLI `bench` + flags execute + fixtures + DEC-050.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Extensão `dare-verify` | Módulos novos; `cargo test -p dare-verify` | Exit 0 |
| O-02 | Fail-to-pass | Suite/fixture falha→passa após patch; report aspecto | Unit + integration |
| O-03 | Anti-tamper | Detecção de bypass/remoção de asserts; bloqueia DONE | Unit |
| O-04 | Mutation adapters | ≥1 adapter funcional por stack alvo; score limiar | Integration (tool presente) |
| O-05 | Formal opt-in | `--formal` / backend dafny\|verus\|lean; auditável | Unit + smoke |
| O-06 | Repair loop | ≤5 tentativas; para em sucesso ou esgota | Unit |
| O-07 | Best-of-N + Pareto | `--best-of N` produz N candidatos; vencedor Pareto | Integration |
| O-08 | Decay policy | `--policy decay`: CONTINUE/FRESH_START/REPLAN/ESCALATE/STOP | Unit |
| O-09 | CLI `dare bench` | `--suite`, `--json`, `--baseline`, `--fail-on-regression` | CLI smoke |
| O-10 | Fix·Rate / regressão | Pass-to-pass regression zera FixRate; limiar regressão | Golden |
| O-11 | Baseline files | `.dare/verification/<taskId>.json` schema estável | Unit |
| O-12 | Docs + DEC-050 | Docs compat + DECISION-LOG; matriz 049 | Review |
| O-13 | Ralph close | clippy/test verify+cli + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Paridade Ciclo 20 com TS 3.18.1 |
| Tech Lead | DARE CLI Rust | Gates pós-Ralph; DEC-050; opt-in formal/mutation |
| Engenheiro | Consumidor | `dare bench --suite … --fail-on-regression` |
| CI / Release | Pipelines | Exit codes estáveis; sem deps formais obrigatórias |
| Agente IDE | Claude/Cursor | `--best-of` / `--policy decay` no execute |
| Segurança | — | SafeCommand; path jail; sem secrets em reports |
| Compat | Baseline TS | Diffs A/B/C; suite inválida → exit **2** |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Estender `dare-verify` | MUST | Sem ciclo com `dare-cli`; Ralph existente preservado |
| RF-02 | Aspecto fail-to-pass | MUST | Lista de testes que falhavam passam após mudança; reportável |
| RF-03 | Aspecto anti-tamper | MUST | Heurísticas de remoção/bypass de asserts/gates; FAIL bloqueia DONE |
| RF-04 | Aspecto mutation | MUST | Adapters: stryker / mutmut / cargo-mutants / infection via SafeCommand |
| RF-05 | Threshold mutation | MUST | Score ≥ limiar (proposta Mestre **0.70**); Blueprint congela |
| RF-06 | Mutation incremental | SHOULD | Preferir diff git quando disponível; `--full-mutation` força completo |
| RF-07 | Aspecto formal | MUST | Opt-in; backends `dafny` (default) \| `verus` \| `lean` |
| RF-08 | Alvos formal | MUST | Seleção por tag `@dare-formal` (ou equivalente documentado) |
| RF-09 | Anti-bypass formal | MUST | Não aceitar “pass” sem ferramenta/prova auditável |
| RF-10 | Repair loop | MUST | Máx **5** reparos; log tentativas; para em OK ou esgota |
| RF-11 | Integração pós-Ralph | MUST | `--complete` / verify path: Ralph → aspectos avançados → DONE |
| RF-12 | Flags execute | MUST | `--best-of <n>`, `--full-mutation`, `--formal`/`--no-formal`, `--formal-backend`, `--policy decay\|fixed`, `--verify`/`--no-verify`, `--verdict-json` |
| RF-13 | Best-of-N | MUST | N worktrees/candidatos sob path jail; seleção Pareto determinística |
| RF-14 | Decay policy | MUST | `failureSignature` (sha256 truncado de aspecto+stderr normalizado); saturação → fresh-start/replan/escalate; máx tentativas documentado |
| RF-15 | REPLAN | SHOULD | Integra `spliceSubDag` / refine existente quando política pede REPLAN |
| RF-16 | `dare bench` | MUST | `--suite <dir>`, `--json`, `--baseline <file>`, `--fail-on-regression <n>`, `--filter` opcional |
| RF-17 | Fixtures bench | MUST | Layout: `suite.json`, `patch.diff`, `fail_to_pass.txt`, `pass_to_pass.txt`, `repo/` |
| RF-18 | Fix·Rate | MUST | Métrica definida no Blueprint; pass-to-pass regression → FixRate **0** (critério aceite microplano) |
| RF-19 | Regressão baseline | MUST | Compare vs baseline; excedeu limiar → exit **1**; suite inválida → exit **2** |
| RF-20 | Reports JSON | MUST | camelCase; `schemaVersion` estável; `--verdict-json` / `--json` |
| RF-21 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-22 | Exit codes | MUST | 0 ok; **1** regressão/gate fail; **2** suite/baseline/usage inválido; 3 NotFound; 4 InvalidInput; 5 Io; 6 Guard; 124 timeout |
| RF-23 | Capabilities | MUST | `dare-bench` `cli_commands:["bench"]`; execute flags documentados |
| RF-24 | Docs + DEC-050 | MUST | Docs + DECISION-LOG + matriz 049 Concluído |
| RF-25 | Tools ausentes | SHOULD | Mutation/formal: soft-fail ou skip auditável se binário ausente — Blueprint escolhe (CI sem Dafny não quebra default) |
| RF-26 | Spawn | MUST | Só `SafeCommand` argv-only; timeout; truncagem streams |

### 4.1 Superfície CLI (proposta)

```bash
dare bench [--suite <dir>] [--json] [--baseline <file>] [--fail-on-regression <n>] [--filter <glob>] [-d <dir>]

dare execute --complete <id> [--verify|--no-verify] [--full-mutation] [--formal|--no-formal]
  [--formal-backend dafny|verus|lean] [--best-of <n>] [--policy decay|fixed] [--verdict-json]
```

### 4.2 Aspectos de verificação (proposta)

| Aspecto | Quando | Bloqueia DONE se FAIL |
|---------|--------|------------------------|
| Ralph build/test/lint | Sempre (salvo `--no-verify` / config) | Sim |
| fail-to-pass | Verify on | Sim |
| anti-tamper | Verify on | Sim |
| mutation | Verify on; full se `--full-mutation` | Sim se abaixo limiar |
| formal | Só se `--formal` ou config formal.enabled | Sim se opted-in e FAIL |

### Fora de escopo (ver §10)

- `dare ai *` (**050**)
- Dashboard telemetry UI (**051**)
- MCP protocol server (**052**)
- Novos ErrorKinds além dos já existentes (salvo Blueprint justifique)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Ordenações e Fix·Rate estáveis; golden | Bit-stable reports |
| RNF-02 | Performance | `dare bench` suite pequena < 120 s CI soft | Soft |
| RNF-03 | Portabilidade | Win/macOS/Linux; paths posix-rel em fixtures | Cross-plat |
| RNF-04 | Observabilidade | LoopVerdict / BenchReport camelCase + tracing | Unit |
| RNF-05 | Isolamento | Domínio em `dare-verify`; CLI fino | `cargo tree` |
| RNF-06 | Compat | Paridade TS 3.18.1 aspectos/bench/exits | Diff DEC |
| RNF-07 | Opt-in pesado | Formal/mutation tools não são deps Cargo obrigatórias | Review |
| RNF-08 | Manutenibilidade | Adapters mutation/formal em módulos separados | Review |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar suite path, flags, N, limiares, backends | OWASP A03 |
| RS-02 | Redigir secrets em stderr/reports de gates | OWASP A02 |
| RS-03 | Path jail em suite/repo/worktrees/baselines | OWASP A01 / 005 |
| RS-04 | `cargo audit` sem HIGH/CRITICAL novas | OWASP A06 |
| RS-05 | Secrets só via env; nunca em fixtures commitadas | Supply chain |
| RS-06 | Spawn SafeCommand; sem shell concat | Process 006 |
| RS-07 | Worktrees best-of sob `.dare/` com jail | Isolation |
| RS-08 | Formal anti-bypass: não forjar prova | Integrity |
| RS-09 | Truncar stdout/stderr de tools (cap existente) | DoS / PII |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | workspace `1.85+` |
| CLI | `clap` | `=4.5.40` |
| Crate | `dare-verify` (estender) | workspace |
| Core | `dare-core` | SafeCommand, ProjectRoot |
| DAG / state | `dare-dag` | attempts, spliceSubDag |
| Hash | `sha2` | failureSignature / FixRate inputs |
| Tools externos | cargo-mutants, stryker, mutmut, infection, dafny, verus, lean | Opt-in no PATH |
| Baseline | `@dewtech/dare-cli` | 3.18.1 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem projeto | Local | FS | R/W | fixtures, `.dare/verification/`, worktrees | dare-verify |
| Mutation CLIs | Local | argv | Spawn | score / report | adapters |
| Formal CLIs | Local | argv | Spawn | prove result | adapters |
| Git (diff) | Local | argv | Spawn | incremental mutation | Soft |
| Baseline TS 3.18.1 | Referência | — | Comp. | FixRate / exits | Compat |
| `dare ai` | — | — | — | **Fora** | 050 |

---

## 9. RESTRIÇÕES

- Pré-requisitos **029–034** (e refine **033** para REPLAN) satisfeitos.
- Exit **2** para suite/baseline inválidos (Mestre §2.2) sem DEC breaking.
- Formal e mutation tools **não** pinados como deps Cargo obrigatórias.
- Um DEC (**050**).
- Sem dependência de `dare-ai` workflows / dashboard.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| `dare ai doctor/providers/run/prompt` | **050** |
| Dashboard / REST / MCP | **051/052** |
| Hooks / steering | Já **048** |
| Reescrever Ralph build/test/lint | Já **029** |
| Treinar modelos / LLM no bench | Bench é determinístico |
| Publicar crates formais no crates.io | N/A |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Tools formais/mutation ausentes em CI | Alta | Médio | Opt-in; skip auditável; fixtures sem tool para unit |
| R-02 | Fix·Rate diverge do TS | Alta | Alto | Congelar fórmula no Blueprint + golden |
| R-03 | Best-of-N caro / worktrees órfãos | Média | Médio | Cap N; cleanup; path jail |
| R-04 | Decay REPLAN quebra DAG | Média | Alto | Reusar spliceSubDag testado (033) |
| R-05 | Anti-tamper falsos positivos | Média | Médio | Heurísticas documentadas; fixtures |
| R-06 | Timeout tools longos | Alta | Médio | Timeouts SafeCommand; exit 124 |
| R-07 | Embed/assets race em Ralph full test | Média | Médio | Já visto em 048; clean dare-assets se necessário |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Aspectos pós-Ralph e limiar mutation aceites (ou fechados no Blueprint)
- [ ] Formal opt-in + anti-bypass alinhados
- [ ] Fix·Rate / baseline / exit 2 suite inválida alinhados
- [ ] Best-of-N + decay sem conflitar com agent `fixed` existente
- [ ] Fora de escopo (050/051/052) alinhado
- [ ] DEC id **050** confirmado
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-049-verificacao-avancada-e-bench.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Pós-Ralph + bench + flags execute; não ai/dashboard | 🟢 Mestre §38 / §5.5 |
| ambiguity | Fórmula exacta Fix·Rate e limiar `--fail-on-regression` | 🔴 Blueprint |
| ambiguity | Tool ausente: skip vs fail quando `--formal` explícito | 🟡 proposta: explícito → fail; default off → skip |
| ambiguity | Pareto dimensions exactas (score, tempo, aspects) | 🔴 Blueprint |
| gap | Schema JSON LoopVerdict / BenchReport | 🔴 Blueprint |
| gap | Inventário fixtures TS a portar | 🟡 Blueprint |

### PM

- Aceite v1: fail-to-pass + anti-tamper + mutation limiar + formal opt-in + bench Fix·Rate com regressão pass-to-pass → 0; Ralph verde; DEC-050.
- Preferir **opt-in** para tools pesados; default CI não exige Dafny/Stryker instalados.
- Decay e best-of devem reutilizar worktrees/jail existentes — sem nova superfície de shell.

---

## Próximas etapas

1. Revisar e aprovar este Design (especialmente RF-05 limiar, RF-18 Fix·Rate, RF-25 tools ausentes).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-049-verificacao-avancada-e-bench.md`.
