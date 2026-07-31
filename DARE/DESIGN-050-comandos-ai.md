# DESIGN: Comandos `dare ai` (Microplano 050)

> **Versão:** v1.0 | **Data:** 2026-07-29 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/050-comandos-ai.md`  
> **Referência:** Documento Mestre §39 Ciclo 21 · §15 drivers/enrichment · §22 `src/ai/` · fundação **`dare-ai` 024** (DEC-025) · drivers agent **031** (DEC-037) · process/path **005/006** · baseline TS `@dewtech/dare-cli@3.18.1` · skill `/dare-ai` · próximo **051**  
> **Posição:** 50 de 56  
> **Arquivo:** `DARE/DESIGN-050-comandos-ai.md`  
> **Escopo deste ciclo:** superfície CLI **`dare ai`** (`doctor` · `providers` · `run` · `prompt`) · diagnóstico de providers · capabilities · execução de enrichment com timeouts/redaction · mock CI · docs + **DEC-051**.  
> **Não** dashboard/REST/MCP (**051/052**). **Não** reescrever pipeline `design --ai` (já **024**). **Não** misturar com `AgentDriver` / `dare execute --agent` (**031**). DEC proposto: **DEC-051** (DEC-050 = verify/bench **049**).

---

## 1. DESCRIÇÃO

Expor no CLI Rust a superfície **`dare ai`** para **diagnosticar** e **executar** providers de enrichment terminal-first já fundados em `crates/dare-ai` (024): `doctor` (ausente / inválido / pronto), `providers` (lista + capabilities), `run` (enrichment de um workflow com facts/markdown), e `prompt` (pré-visualização do prompt **sem vazar env**).

O problema: hoje o enrichment só aparece embutido em `dare design --ai` (e flags `--ai` em outros comandos), sem ferramenta dedicada para CI/ops inspecionar providers, montar prompts ou correr enrichment de forma isolada e auditável. Quem usa: developers, CI (mock) e agentes IDE que precisam de saída JSON estável. Entrega verificável: `crates/dare-cli/src/commands/ai.rs` + extensões mínimas em `dare-ai` + smokes + DEC-051.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | CLI `dare ai` | Subcomando no help; `Commands::Ai` | Smoke help |
| O-02 | `ai doctor` | Estados `missing` / `invalid` / `ready` por provider | Unit + CLI |
| O-03 | `ai providers` | Lista canónica + capabilities; ordenação estável | CLI `--json` |
| O-04 | `ai run` | Enrich + schema validate; `--command` + facts/markdown | Integration + mock |
| O-05 | `ai prompt` | Imprime prompt redigido; **zero** dump de env | Unit |
| O-06 | JSON envelope | `schemaVersion` estável; camelCase | Golden |
| O-07 | Timeouts | Reusa `ENRICH_TIMEOUT` (20 min); timeout → exit **124** | Unit |
| O-08 | Redaction | Secrets/tokens/prompts longos redigidos em logs/erros | Unit |
| O-09 | Provider ausente | Doctor `missing`; `run` com exe missing → exit **1** tipado | CLI |
| O-10 | Malformed output | Schema fail → exit **4** (InvalidInput); sem write parcial | Unit |
| O-11 | Mock CI | `--provider mock` determinístico sem rede/spawn | Smoke |
| O-12 | Docs + DEC-051 | `docs/compatibility/cli-ai.md` + DECISION-LOG; matriz 050 | Review |
| O-13 | Ralph close | test/clippy `dare-ai`+`dare-cli` + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Paridade Ciclo 21 com TS 3.18.1 |
| Tech Lead | DARE CLI Rust | Separação `dare-ai` ≠ `dare-agent`; DEC-051 |
| Engenheiro | Consumidor | `dare ai doctor` / `run --provider mock` |
| CI / Release | Pipelines | Mock + exits estáveis; sem CLIs externos |
| Agente IDE | Claude/Cursor | `--json` para orquestração |
| Segurança | — | Redaction; sem API key no CLI; SafeCommand |
| Compat | Baseline TS | Diffs A/B/C; ids `mock`/`codex`/… |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Módulo `commands/ai.rs` | MUST | Wired em `main.rs` / `commands/mod.rs`; help lista `ai` |
| RF-02 | `dare ai doctor` | MUST | Reporta status por provider (ou `--provider` único): `missing` \| `invalid` \| `ready` (+ motivo curto en-US) |
| RF-03 | Doctor exit | MUST | Exit **0** sempre que o diagnóstico completar (informacional), salvo usage inválido → **2** |
| RF-04 | `dare ai providers` | MUST | Lista ids canónicos ordenados: `mock`, `codex`, `claude-code`, `cursor-cli`, `antigravity-cli` |
| RF-05 | Capabilities | MUST | Por provider: flags estáveis (ex. `enrich`, `implemented`, `envOverride`, `defaultTimeoutSecs`) — schema no Blueprint |
| RF-06 | `dare ai run` | MUST | Flags: `--command <id>`, `--provider`, `--facts <path>` e/ou `--markdown <path>`, `--json`, `-d` |
| RF-07 | Commands suportados em `run` | MUST | Pelo menos: `design` (4 ENRICHABLE atuais) + **≥1** workflow adicional documentado (🟡 proposta: `blueprint` **ou** `reverse` — Blueprint congela a lista v1) |
| RF-08 | Schema por command | MUST | Registry de section ids por command; malformed JSON/sections → InvalidInput **4**; não escreve artefato parcial |
| RF-09 | Escrita em `run` | MUST | Default: **não** escreve DESIGN/BLUEPRINT (stdout/report only) **ou** `--write` opt-in explícito — Blueprint escolhe uma; aceito se documentado e testado |
| RF-10 | `dare ai prompt` | MUST | Monta o prompt que seria enviado; imprime human ou `--json`; **não** executa provider |
| RF-11 | Prompt sem vazar env | MUST | Não imprime valores de `DARE_*_COMMAND`, `PATH`, tokens, nem argv override completo com secrets; redaction via `redact_prompt_for_log` / equivalente |
| RF-12 | Provider default | MUST | Sem `--provider`: produto `codex` (alinha DEC-025); CI smokes usam `mock` |
| RF-13 | Providers não implementados | MUST | `claude-code` / `cursor-cli` / `antigravity-cli`: doctor = `invalid`/`not_implemented`; `run` → exit **4** tipado (não silent no-op) — **ou** implementar neste ciclo se effort permitir (SHOULD) |
| RF-14 | Implementar CLIs restantes | SHOULD | Completar adapters Claude/Cursor/Antigravity em `dare-ai` (SafeCommand + env override), reusando padrões de `CodexCliProvider` |
| RF-15 | Timeouts | MUST | `ENRICH_TIMEOUT` = 20 min; cancel/kill árvore; mapear timeout → **124** |
| RF-16 | Caps | MUST | Reusar `STDOUT_CAP` / `STDERR_CAP` / `BODY_MAX` / `PROMPT_LOG_MAX` |
| RF-17 | JSON | MUST | `--json` em doctor/providers/run/prompt; `schemaVersion: 1` (congelar no Blueprint) |
| RF-18 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-19 | Exit codes | MUST | 0 ok; **1** provider/runtime fail; **2** usage; **3** NotFound (facts path); **4** InvalidInput/malformed; **5** Io; **6** Guard N/A; **124** timeout |
| RF-20 | Path safety | MUST | Facts/markdown sob ProjectRoot / SafeRelativePath; rejeitar traversal |
| RF-21 | SafeCommand | MUST | Spawn argv-only; sem shell concatenado |
| RF-22 | Capability matrix | MUST | Capability `dare-ai-cli` (ou nome Blueprint) → `cli_commands:["ai"]`; atualizar manifest hash |
| RF-23 | Docs + DEC-051 | MUST | `docs/compatibility/cli-ai.md`; append DEC-051; matriz 050 Concluído |
| RF-24 | Separação agent | MUST | **Não** depende de `dare-agent`; ids ProviderId ≠ AgentDriver ids (`claude-code` ≠ `claude`) |
| RF-25 | Smoke suite | MUST | `crates/dare-cli/tests/ai_cli.rs`: doctor mock ready; providers json; prompt no-env-leak; run mock ok; unknown provider; missing facts → 3; malformed → 4 |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare ai doctor [--provider <id>] [--json] [-d]
dare ai providers [--json] [-d]
dare ai run --command <id> [--provider <id>] [--facts <path>] [--markdown <path>] [--write] [--json] [-d]
dare ai prompt --command <id> [--facts <path>] [--markdown <path>] [--provider <id>] [--json] [-d]
```

### Relação com 024

- Pipeline `dare design --ai` **permanece**; `dare ai run --command design` reutiliza as mesmas APIs (`AiProvider`, schema, inject) sem duplicar lógica.
- Este microplano **adiciona** a superfície de diagnóstico/execução isolada — não remove flags `--ai` existentes.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | `mock` bit-a-bit estável para mesmo request | Golden |
| RNF-02 | Performance | Doctor/providers sem spawn pesado (probe PATH leve) | p95 doctor < 2 s típico |
| RNF-03 | Segurança | Sem API keys no processo CLI; só CLIs locais | Audit docs |
| RNF-04 | Segurança | Redaction em stderr/logs/prompt preview | Unit |
| RNF-05 | Observabilidade | `--json` + human; sem PII/secrets | Review |
| RNF-06 | Manutenibilidade | Domínio em `dare-ai`; CLI thin | Sem ciclo crates |
| RNF-07 | Compat | Linux / macOS / Windows (PATH probe + argv) | Smokes onde aplicável |
| RNF-08 | UX | Help lists subcomandos; erros acionáveis en-US | Manual |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar `--command`, `--provider`, paths de facts/markdown antes de I/O | OWASP A03 |
| RS-02 | Não persistir/imprimir secrets; redact prompts/stderr; caps de tamanho | OWASP A02 |
| RS-03 | Path jail ProjectRoot; sem leitura fora do projeto | OWASP A01 / path safety |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL no Ralph close | OWASP A06 |
| RS-05 | Overrides só via env `DARE_*_COMMAND` — nunca hardcoded secrets | Supply chain |
| RS-06 | Spawn via SafeCommand (argv); proibir shell string | Process safety 006 |
| RS-07 | Prompt preview não ecoa env completo nem valores de override sensíveis | Aceite microplano |
| RS-08 | Output malformed do LLM nunca executado como código — só parse schema | Prompt injection defense |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | workspace `rust-toolchain.toml` |
| Domínio | `dare-ai` | Estender (doctor helpers, capabilities, schemas multi-command) |
| CLI | `dare-cli` | `commands/ai.rs` + clap |
| Processo | `dare-core` SafeCommand / ProcessRunner | 006 |
| Path | `ProjectRoot` / `SafeRelativePath` | 005 |
| Providers | mock + codex (+ SHOULD claude/cursor/antigravity) | terminal-first |
| Testes | `cargo test` unit + `ai_cli` integration | tempfile |
| Docs | `docs/compatibility/cli-ai.md` | + DEC-051 |
| Containerização | N/A neste ciclo | CLI |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Codex CLI | Provider local | argv | Spawn | enrich JSON stdout | `CodexCliProvider` |
| Claude Code CLI | Provider local | argv | Spawn | idem | SHOULD neste ciclo |
| Cursor CLI | Provider local | argv | Spawn | idem | SHOULD |
| Antigravity CLI | Provider local | argv | Spawn | idem | SHOULD |
| Filesystem projeto | Local | FS | R | facts/markdown | path jail |
| Env `DARE_*_COMMAND` | Config | env | In | argv override | dare-ai |
| Baseline TS 3.18.1 | Referência | — | Comp. | exits / ids | Compat |
| Dashboard/MCP | — | — | — | **Fora** | 051/052 |

---

## 9. RESTRIÇÕES

- Pré-requisito **024** concluído (`dare-ai` + DEC-025); drivers agent **031** existem mas **não** são dependência de crate.
- Um DEC (**051**); não reabrir DEC-025/037 sem necessidade.
- Sem SDK cloud (Anthropic/OpenAI HTTP) — só CLIs locais.
- Sem dependência de dashboard/MCP.
- Exit codes alinhados ao envelope CLI existente (Mestre §2.2).
- Docker fase omitida (padrão 046–049).

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| Dashboard / REST compat / MCP | **051/052** |
| Reescrever `dare design --ai` pipeline | Já **024** |
| `AgentDriver` / execute `--agent` | **031** — superfície distinta |
| Self-update / packaging 1.0 | **053+** |
| Schemas Zod literais TS (8 workflows) 100% se inventário incompleto | Blueprint lista v1 mínima; restantes COULD incremental |
| Treinar/fine-tune modelos | N/A |
| API keys / cloud LLM no processo `dare` | Política terminal-first |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Confundir ProviderId com AgentDriver ids | Alta | Alto | Docs + testes; RF-24; nomes distintos |
| R-02 | Prompt vaza env/override | Média | Alto | RS-07; unit `prompt_no_env_leak`; redact |
| R-03 | Schemas multi-command incompletos vs TS | Alta | Médio | Lista v1 congelada no Blueprint; diffs B documentados |
| R-04 | Doctor falso `ready` (PATH stub) | Média | Médio | Probe: exe resolve + `--help`/version leve opcional — Blueprint |
| R-05 | Timeout 20 min em CI | Baixa | Médio | Smokes só `mock`; timeout unit com fake runner |
| R-06 | `assets_verify_ok` / CARGO_TARGET_DIR flake | Média | Médio | Já visto 048/049; clean `dare-assets`; target local |
| R-07 | `--write` corrompe artefato | Média | Alto | Default no-write; write opt-in + atomic; testes fail-keep |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Subcomandos `doctor` / `providers` / `run` / `prompt` alinhados ao Mestre §39
- [ ] Critérios: doctor ausente/inválido/pronto; prompt sem vazar env; mock CI
- [ ] Lista de `--command` v1 e política `--write` aceites (ou fechadas no Blueprint)
- [ ] Separação `dare-ai` ≠ `dare-agent` confirmada
- [ ] DEC id **051** confirmado (050 = verify/bench)
- [ ] Fora de escopo (051/052 dashboard/MCP) alinhado
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-050-comandos-ai.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Superfície `dare ai` + diagnosis/run/prompt; reusa `dare-ai` | 🟢 Mestre §39 · microplano 050 |
| ambiguity | Lista exacta de `--command` v1 além de `design` | 🔴 Blueprint |
| ambiguity | `run` escreve disco por default ou só com `--write` | 🟡 proposta: **no-write default** + `--write` opt-in |
| ambiguity | Doctor: o que distingue `invalid` vs `not_implemented` | 🟡 Blueprint enum status |
| gap | Schema JSON reports doctor/providers/run/prompt | 🔴 Blueprint |
| gap | Completar 3 providers CLI vs deixar stubs tipados | 🟡 SHOULD completar; MUST stubs tipados |

### PM

- Aceite v1: doctor diferencia ausente/inválido/pronto; providers+capabilities; run mock+schema; prompt sem env leak; malformed → 4; Ralph verde; DEC-051.
- Preferir **mock** em CI; providers reais opt-in via PATH/env.
- Não bloquear v1 em 100% dos 8 schemas TS — congelar subset e classificar diffs.

---

## Próximas etapas

1. Revisar e aprovar este Design (especialmente RF-07 commands, RF-09 write policy, RF-13/14 providers).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-050-comandos-ai.md`.
