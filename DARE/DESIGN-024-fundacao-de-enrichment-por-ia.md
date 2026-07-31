# DESIGN: Fundação de enrichment por IA (Microplano 024)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/024-fundacao-de-enrichment-por-ia.md`  
> **Referência:** Microplanos **006** (processos seguros) · **023** (markers + preserve) · Documento Mestre §15 / §22 · baseline TS 3.18.1 (`src/ai/`)  
> **Posição:** 24 de 56  
> **Arquivo:** `DARE/DESIGN-024-fundacao-de-enrichment-por-ia.md`  
> **Escopo deste ciclo apenas:** crate `dare-ai` + ligação opcional `dare design --ai`. Tudo o que pertence a microplanos posteriores fica em **Fora do Escopo**.

---

## 1. DESCRIÇÃO

Este Design cobre a **fundação de enrichment por IA** do CLI nativo: um pipeline **opcional** que, após a geração determinística (023), chama um **provider terminal-first** (sem API key no CLI), **valida a resposta por schema**, e **injeta texto apenas dentro dos markers** `<!-- AGENT:BEGIN/END section="…" -->` já emitidos em `DARE/DESIGN.md`.

Resolve a lacuna entre scaffold determinístico e documento enriquecido por agente de IDE/CLI externo — alinhado ao TS 3.18.1 (`src/ai/`), mas em Rust na crate **`dare-ai`**. Quem usa: developers com `dare design --ai`, testes via **mock**, e CI determinística sem rede.

Entrega: crate `crates/dare-ai`, wiring mínimo em `dare design --ai/--provider`, fixtures mock, docs de compatibilidade e DEC (número no Blueprint).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-ai` | Member do workspace; sem ciclo com `dare-cli` ↔ `dare-ai` indevido | `cargo test -p dare-ai` |
| O-02 | Trait `AiProvider` | Contrato tipado (enrich request/response) | Unit |
| O-03 | Provider `mock` | Resposta fixa/golden; zero rede/processo | Unit determinístico |
| O-04 | Um provider CLI real | Spawn argv separado (006); override env; timeout 20 min | Integração / unit com fake cmd |
| O-05 | Overrides `DARE_*_COMMAND` | Env documentados; argv resolvido sem shell | Unit |
| O-06 | Timeout 20 min | Cancel/kill filho; erro tipado (mapear exit/timeout 006) | Unit com sleep fake |
| O-07 | Validação por schema | Resposta inválida → rejeitada; sem write parcial | Unit |
| O-08 | Injeção só em markers | Só secções ENRICHABLE; unmanaged intacto | Unit (usa merge 023) |
| O-09 | Falha não corrompe | Em erro de provider/schema, ficheiro = estado pré-enrich (ou só deterministic) | Unit |
| O-10 | Redação de logs | Secrets/tokens/prompts longos não em cleartext default | Unit |
| O-11 | Superfície `design --ai` | Flag + `--provider`; default documentado | Smoke |
| O-12 | Ralph + docs | fmt/clippy/test/audit/deny + docs compat | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Enrichment opcional sem API key no CLI |
| Tech Lead | Time DARE CLI Rust | Separação `dare-ai` vs `dare-agent` (031) |
| Engenheiro CLI | Time implementação | Crate + wiring `design --ai` |
| Usuário Final | Devs | `dare design "…" --ai --provider mock\|…` |
| CI | Pipelines | Mock + testes sem CLIs externos |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-ai` | MUST | Workspace member; deps: `dare-core`, processo 006 (crate existente), serde; **não** depende de `dare-agent` |
| RF-02 | Trait `AiProvider` | MUST | Método(s) tipados para enrich (request → result); async ou sync conforme Blueprint (alinhar 006) |
| RF-03 | Provider `mock` | MUST | Id canónico `"mock"`; saída determinística a partir do request; sem spawn |
| RF-04 | Um provider CLI real | MUST | Pelo menos **um** de: `codex` \| `claude-code` \| `cursor-cli` \| `antigravity-cli` (🟡 default produto TS = `codex` — confirmar no Blueprint) |
| RF-05 | Overrides env | MUST | `DARE_CODEX_COMMAND`, `DARE_CLAUDE_COMMAND`, `DARE_CURSOR_COMMAND`, `DARE_ANTIGRAVITY_COMMAND` (nomes alinhados ao TS); argv split seguro — **sem** shell |
| RF-06 | Timeout | MUST | **20 minutos** por invocação; ao expirar: matar árvore de processo (006), erro tipado, **sem** escrever enrich |
| RF-07 | Schemas de resposta | MUST | Validar payload do provider antes da injeção (JSON schema / tipos serde + regras); inválido → reject |
| RF-08 | Mapa secção → conteúdo | MUST | Resposta validada contém (ou mapeia para) bodies das secções ENRICHABLE do 023: `description`, `objectives`, `functional-requirements`, `stack` |
| RF-09 | Injeção markers | MUST | Substituir **somente** blocos BEGIN/END geridos; texto fora intacto (reuse `merge_preserve` / API design) |
| RF-10 | Ordem pipeline | MUST | Determinístico (023) **sempre**; enrich **só** se `--ai`; depois validate → inject → atomic write |
| RF-11 | Falha não corrompe | MUST | Provider fail / timeout / schema fail → não altera ficheiro já escrito deterministicamente **ou** restaura pré-enrich; critério: conteúdo unmanaged + markers pré-IA preservados |
| RF-12 | `dare design --ai` | MUST | Flag long `--ai`; requer descrição ou `--interactive` como 023 |
| RF-13 | `--provider <id>` | MUST | Valores: pelo menos `mock` + o CLI real implementado; desconhecido → Usage/InvalidInput |
| RF-14 | Default provider | MUST | Documentado (🟡 `codex` se CLI real for codex; senão o único real; em CI preferir `mock`) |
| RF-15 | Report | MUST | Human + `--json`: indicar se enrich rodou, provider, `enriched: true\|false`, warnings; schema report estendido ou campo no DesignReport (detalhe Blueprint) |
| RF-16 | Redact logs/erros | MUST | Não logar API keys, tokens, nem body completo do prompt/resposta em nível default |
| RF-17 | Cap tamanho | MUST | Caps de stdin/stdout do processo e de bodies injetados (valores no Blueprint); reject se excedido |
| RF-18 | Docs + DEC | MUST | `docs/compatibility/` (ex. `cli-design-ai.md` ou secção em `cli-design.md`) + entrada DEC no DECISION-LOG |
| RF-19 | Providers restantes | SHOULD | Stubs ou `Unsupported` tipado para CLIs não implementados neste ciclo (não silent no-op) |
| RF-20 | Schemas multi-comando | COULD | Preparar registry para blueprint/reverse/… — **só tipos/placeholder**; validação completa por comando = microplanos donos |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare design "<desc>" --ai [--provider mock|codex|…]
dare design --interactive --ai [--provider …]
# + --json / --no-color (004)
# Sem --ai → comportamento 023 inalterado
```

### Contrato de disco

- Continua a escrever **apenas** `DARE/DESIGN.md` (igual 023).
- Enrichment **não** cria ficheiros novos neste ciclo.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | `mock` bit-a-bit estável para mesmo request | Golden |
| RNF-02 | Offline (mock) | Sem rede / sem binário externo | Unit |
| RNF-03 | Isolação | `dare-ai` ≠ `dare-agent` (drivers execute) | Review deps |
| RNF-04 | Observabilidade | Span `design.ai` / `ai.enrich`; erros tipados 004 | Unit |
| RNF-05 | Cross-platform | Argv + paths via 005/006; Win/macOS/Linux | CI / testes path |
| RNF-06 | Performance | Timeout ceiling 20 min; mock ≪ 1 s | Informal |
| RNF-07 | Cancelamento | Respeitar cancel do runtime 006 quando aplicável | Unit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar provider id, caps, e corpo a injetar (sem path traversal nos markers) | OWASP A03 |
| RS-02 | Redact secrets/tokens em logs, tracing e mensagens de erro | OWASP A02 / 004 |
| RS-03 | Escrita atómica sob project root; falha → sem documento parcial enrich | 005 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Secrets só via env do **processo filho** se o CLI externo exigir; nunca hardcode; CLI DARE sem API key própria neste ciclo | Supply chain |
| RS-06 | **Sem shell concatenado** — argv separado (006) | Command injection |
| RS-07 | Path safety em leitura/escrita `DARE/DESIGN.md` | 005 |
| RS-08 | Tratar stdout do provider como **untrusted** até schema pass | Prompt/output injection |
| RS-09 | Não executar conteúdo injetado; markers continuam HTML comments | Injection |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-ai` | `0.1.0-alpha.0` (workspace) |
| Processo | crate segura 006 (`dare-process` ou nome existente) | workspace |
| Core | `dare-core` | erros / path |
| Design I/O | reutilizar APIs 023 (`merge`/markers) via `dare-cli` **ou** funções partilhadas sem ciclo — decisão no Blueprint |
| Serde / schema | `serde` + `serde_json` (+ validador se já no workspace) | workspace |
| CLI | clap **4.5.40** em `dare-cli` | flags `--ai` / `--provider` |
| Testes | mock + tempfile + fake command | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Provider CLI (codex/claude/…) | Processo local | argv stdout | Out→In | Prompt / JSON ou texto | Provider |
| Env `DARE_*_COMMAND` | Config | — | In | Override binário+args | Ops/dev |
| Filesystem | Local | r/w | In/Out | `DARE/DESIGN.md` | CLI |
| Mock | In-process | — | — | Fixture | Testes |
| Baseline TS 3.18.1 `src/ai/` | Referência | — | In | Paridade classificada | Compat |

> Sem HTTP SDK Anthropic neste ciclo (isso é **dare-agent** / 031).

---

## 9. RESTRIÇÕES

- Pré-requisitos: **006** e **023** concluídos.
- Enrichment **opcional** (`--ai`); default sem flag = 023 puro.
- Timeout fixo **20 minutos** (paridade TS / microplano).
- Um provider CLI real MUST; restantes MAY stub.
- Mensagens CLI en-US.
- Sem mudar contratos públicos sem ADR/DEC.
- Não misturar enrichment com `AgentDriver` de execute.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo / dono |
|------|----------------|
| `dare blueprint` (+ `--ai` blueprint) | **025** |
| Enrichment de reverse/dna/migrate/patterns/review/refine | Microplanos donos + RF-20 só hook |
| `dare ai doctor\|providers\|run\|prompt` | **050** |
| `AgentDriver` / execute `--agent` / worktrees | **030–031** |
| Claude API (`ANTHROPIC_API_KEY`) | **031** (driver separado) |
| Multi-path `DESIGN-*` / path alternativo | **025** |
| GraphRAG / MCP / dashboard | **040+ / 051+** |
| Self-update / package managers | **053** |
| Otimizações de custo/latência LLM | Sem requisito mensurável neste ciclo |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | CLI externo ausente em CI | Alta | Médio | Default testes = `mock`; doctor/skip tipado |
| R-02 | Schema frouxo injeta lixo | Média | Alto | Reject + O-09 sem write enrich |
| R-03 | Timeout deixa órfão | Média | Alto | Kill group 006; teste |
| R-04 | Ciclo de crates cli↔ai | Média | Médio | Trait em `dare-ai`; CLI só orquestra; extrair merge se preciso |
| R-05 | Diff vs TS nos argv/env | Alta | Médio | DEC + tabela classificação |
| R-06 | Prompt injection no stdout | Média | Alto | Schema allowlist de campos; não interpretar como código |
| R-07 | Escopo vazar para 025/031/050 | Média | Médio | Checklist Fora do Escopo; review |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo = checklist do microplano 024 (AiProvider, mock, 1 CLI real, overrides, timeout 20m, schema, inject markers, redact)
- [ ] Falha de provider **não** corrompe documento (critério aceite)
- [ ] Separação `dare-ai` vs `dare-agent` aceite
- [ ] Fora do Escopo deixa 025/031/050 explícitos
- [ ] RS-01…RS-09 ok
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-024-fundacao-de-enrichment-por-ia.md`

---

## Apêndice A — Paths (024)

| Path | Papel |
|------|-------|
| `crates/dare-ai/` | Trait, mock, provider CLI, schema, inject helpers |
| `crates/dare-cli/src/commands/design.rs` | Flags `--ai` / `--provider`; orquestração |
| `crates/dare-cli/src/main.rs` | Clap |
| `tests/fixtures/ai/` (ou similar) | Mock goldens / respostas inválidas |
| `docs/compatibility/cli-design.md` ou `cli-design-ai.md` | Docs |
| `docs/DECISION-LOG.md` | DEC (nº no Blueprint) |

## Apêndice B — Gap atual

| Item | Estado |
|------|--------|
| Markers + preserve (023) | ✅ |
| Process spawn seguro (006) | ✅ (pré-requisito) |
| Crate `dare-ai` | 🔴 |
| `AiProvider` + mock | 🔴 |
| Provider CLI real + overrides + timeout | 🔴 |
| Schema validate + inject | 🔴 |
| `design --ai` | 🔴 |
| Docs DEC | 🔴 |

## Apêndice C — Critérios de aceite (microplano)

- [ ] Falha de provider não corrompe documento  
- [ ] Mock permite testes deterministas  
- [ ] Retorno inválido é rejeitado  
- [ ] `cargo fmt --check`, `clippy`, `test` aprovados  
- [ ] Diferenças vs TS classificadas  
- [ ] Artefacto CI/Ralph verde (binário já coberto pelo pipeline 015 existente — sem release channel novo obrigatório neste Design)

## Apêndice D — Próximas etapas

1. Aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-024-fundacao-de-enrichment-por-ia.md`.  
3. `/dare-tasks` → `mp024-*`.  
4. Closeout → microplano **025** (blueprint).
