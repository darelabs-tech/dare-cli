# DESIGN: Comando welcome (Microplano 016)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/016-comando-welcome.md`  
> **Referência:** Microplanos **004** (saída CLI) e **015** (release alpha) · DEC-017 · CI-005 (fix `dare new`) · baseline TS 3.18.1  
> **Posição:** 16 de 56  
> **Arquivo:** `DARE/DESIGN-016-comando-welcome.md` (não substitui Designs 001–015)  
> **Nota:** Existe implementação parcial em `dare-cli::commands::welcome` + smoke CLI + DEC-017 no decision log. Este Design congela o contrato MUST (banner TTY, `--no-banner` / `DARE_NO_BANNER`, quick-start sem `dare new`, snapshots human/no-color) e lista gaps (docs dedicados DEC-017, TASKS/DAG formal, Ralph de closeout).

---

## 1. DESCRIÇÃO

Este Design cobre o comando **`dare welcome`** — a primeira superfície de UX do binário nativo: banner ASCII (apenas em TTY), quick-start do fluxo Design → Architecture → Review → Execute, e flags/env para silenciar o banner. O problema: o welcome legado mencionava `dare new` (comando inexistente — CI-005 Classe B) e imprimia banner em pipes/CI, poluindo saída automatizada.

A entrega é `render_welcome` em `crates/dare-cli/src/commands/welcome.rs`, wiring em `main.rs`, testes unitários com override de TTY, smoke CLI, e documentação DEC-017. Quem consome são developers novos no DARE e scripts que chamam `dare welcome --no-banner`.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Banner só em TTY | `stdout_is_tty=false` → sem ASCII `____` | Unit + smoke |
| O-02 | `--no-banner` | Flag desliga banner mesmo em TTY | Unit + smoke |
| O-03 | `DARE_NO_BANNER` | Env truthy (`1`/`true`/`yes`) desliga banner | Smoke |
| O-04 | Sem `dare new` | Saída **não** contém `dare new` | Assert + debug_assert |
| O-05 | Quick-start atualizado | Menciona design → blueprint → tasks → execute | Snapshot |
| O-06 | Snapshot human no-color | TTY + `no_color` → `BANNER_PLAIN` + tagline + QUICK_START | Assert eq |
| O-07 | Snapshot non-TTY | Só `QUICK_START` bit-igual | Assert eq |
| O-08 | Respeito `--no-color` / `NO_COLOR` | Banner plain quando no_color | Unit |
| O-09 | Ralph Loop | fmt / clippy / test / audit / deny | Exit 0 |
| O-10 | Docs DEC-017 | Doc em `docs/compatibility/` | Presente |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Onboarding UX nativo |
| Tech Lead | Time DARE CLI Rust | DEC-017; CI-005 fix |
| Engenheiro CLI | Time implementação | `welcome.rs` estável |
| Usuário Final | Devs novos | Quick-start claro |
| CI / Automação | Pipelines | Sem banner em non-TTY |
| Compatibilidade | Tech Lead | Paridade observável vs TS |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `dare welcome` | MUST | Exit 0; imprime quick-start |
| RF-02 | Banner condicionado a TTY | MUST | Sem TTY → sem banner ASCII |
| RF-03 | `--no-banner` | MUST | Desliga banner; quick-start permanece |
| RF-04 | `DARE_NO_BANNER` | MUST | Truthy: `1`, `true`, `TRUE`, `yes`, `YES` |
| RF-05 | `--no-color` / `NO_COLOR` | MUST | Com banner: usa `BANNER_PLAIN` em vez de ASCII art |
| RF-06 | Quick-start | MUST | Passos design → blueprint → tasks → execute; **sem** `dare new` |
| RF-07 | Hints úteis | MUST | Menciona pelo menos `dare info` |
| RF-08 | `render_welcome(opts)` | MUST | Função testável; override `stdout_is_tty` |
| RF-09 | Snapshots | MUST | `snapshot_human_tty_no_color` + `snapshot_no_tty` |
| RF-10 | Smoke CLI | MUST | `welcome --no-banner`; env `DARE_NO_BANNER=1` |
| RF-11 | Docs DEC-017 | MUST | Ex. `docs/compatibility/cli-welcome.md` |
| RF-12 | Default CLI sem subcomando | SHOULD | Hint aponta `dare welcome` (não regressar) |
| RF-13 | JSON mode welcome | SHOULD | Envelope ok via renderer 004; body = texto welcome |
| RF-14 | Banner ANSI colorido | COULD | v1 = ASCII sem ANSI color; plain sob no_color |
| RF-15 | i18n pt-BR na saída | COULD | Fora — CLI en-US |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### API / superfície

```text
dare welcome [--no-banner]   # + flags globais --json / --no-color

WelcomeOptions { no_banner, stdout_is_tty: Option<bool>, no_color }
render_welcome(opts: &WelcomeOptions) -> String
```

### Política de banner

| Condição | Banner |
|----------|--------|
| `no_banner` ou `DARE_NO_BANNER` truthy | Off |
| Non-TTY | Off |
| TTY + color allowed | ASCII `BANNER` + tagline |
| TTY + no_color / `NO_COLOR` | `BANNER_PLAIN` + tagline |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Snapshots bit-iguais (LF) | Assert eq |
| RNF-02 | Performance | Render tipicamente < 1 ms | Irrelevante |
| RNF-03 | Compatibilidade | Win / macOS / Linux (TTY detect) | CI 003 |
| RNF-04 | Observabilidade | Sem side effects além de stdout via CLI | — |
| RNF-05 | Manutenibilidade | Lógica em `welcome.rs`; main thin | Clippy limpo |
| RNF-06 | UX | Quick-start ≤ ~30 linhas | Legível |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Sem path/user input no render (só flags/env booleanos) | OWASP A03 |
| RS-02 | Sem secrets na saída welcome | OWASP A02 |
| RS-03 | N/A ownership (comando local read-only) | — |
| RS-04 | `cargo audit` + `cargo deny` | OWASP A06 |
| RS-05 | Sem secrets em código/banner | Supply chain |
| RS-06 | Env `DARE_NO_BANNER` só controla banner (não execução) | Injection |
| RS-07 | Não imprimir caminhos absolutos sensíveis | Privacy |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-cli` | `0.1.0-alpha.0` |
| TTY | `std::io::IsTerminal` | std |
| CLI | clap | workspace |
| Saída | renderer 004 (`ok_msg`) | DEC-005 |
| Testes | unit + `assert_cmd` smoke | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| stdout TTY | Terminal | — | Out | banner + quick-start | CLI |
| Env `DARE_NO_BANNER` / `NO_COLOR` | Config | env | In | booleans | Utilizador / CI |
| Baseline TS 3.18.1 | Referência | — | In | copy/UX | Compat |
| CI 003 / release 015 | Test / artefacto | cargo / GHA | In | unit + smoke + binário | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **004** e **015** concluídos (saída CLI + canal alpha).
- Mensagens CLI **en-US**.
- CI-005: nunca reintroduzir `dare new`.
- Sem git commit automático.
- Implementação parcial: **alinhar gaps** (docs DEC-017), não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- Comando `dare info` completo (017) — só menção no quick-start.
- Wizard interativo / prompts.
- Banner com cores ANSI elaboradas.
- Localização pt-BR na saída.
- Alterar comportamento default sem subcomando além do hint já existente.
- Novos comandos de onboarding além de welcome.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | TTY detect diverge Win/Unix | Média | Médio | Override `stdout_is_tty` nos testes; CI multi-OS |
| R-02 | Regressão `dare new` | Baixa | Alto | debug_assert + smoke `not()` |
| R-03 | Snapshot frágil a copy edits | Média | Baixo | Snapshot só estrutura crítica; `QUICK_START` constante |
| R-04 | Docs DEC-017 só no decision log | Alta | Baixo | RF-11: criar `cli-welcome.md` |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-15 priorizados (TTY / flags / sem `dare new` / snapshots)
- [ ] Política `DARE_NO_BANNER` truthy aceite
- [ ] Pré-requisitos 004 e 015 confirmados
- [ ] DEC-017 / docs alinhados
- [ ] RS validados
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-016-comando-welcome.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-cli/src/commands/welcome.rs` | Render + testes |
| `crates/dare-cli/src/main.rs` | Subcomando `Welcome` |
| `crates/dare-cli/tests/cli_smoke.rs` | Smoke welcome |
| `docs/compatibility/cli-welcome.md` | DEC-017 (a criar/completar) |
| `docs/DECISION-LOG.md` | DEC-017 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| Banner TTY / `--no-banner` / `DARE_NO_BANNER` | ✅ parcial |
| Sem `dare new` | ✅ |
| Quick-start atualizado | ✅ |
| Snapshots unit | ✅ |
| Smoke CLI | ✅ |
| Docs DEC-017 dedicados | ⚠️ gap (só decision log) |
| TASKS/DAG/Ralph formal 016 | ⚠️ pendente |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-016-comando-welcome.md`.  
3. `/dare-tasks` → `mp016-*`.  
4. Após closeout → [`017-comando-info.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/017-comando-info.md).
