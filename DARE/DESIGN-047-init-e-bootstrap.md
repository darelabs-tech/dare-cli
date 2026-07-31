# DESIGN: Init e bootstrap greenfield (Microplano 047)

> **Versão:** v1.0 | **Data:** 2026-07-26 | **Status:** APPROVED (blueprint gerado via `/dare-blueprint` — aguarda aprovação humana do BLUEPRINT antes de `/dare-tasks`)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/047-init-e-bootstrap.md`  
> **Referência:** Documento Mestre §36 Ciclo 18 · DEC-047 / `dare-scaffold` (046) · harnesses 011–014 · discover install 019 · update 022 · baseline TS `@dewtech/dare-cli@3.18.1` · pré-requisitos **011–015**, **022**, **046** · próximo **048**  
> **Posição:** 47 de 56  
> **Arquivo:** `DARE/DESIGN-047-init-e-bootstrap.md`  
> **Escopo deste ciclo:** CLI **`dare init [nome]`** + **`dare bootstrap`** sobre `dare-scaffold` + instalação de harnesses IDE + composição `--fullstack` / `--mcp` / `--transport` / `--toolchain` + `--non-interactive` + idempotência + rollback + golden trees + docs + **DEC-048**.  
> **Não** hooks/steering (**048**). **Não** dashboard/MCP server (**051/052**). **Não** self-update (**053**). DEC proposto: **DEC-048** (DEC-047 já consumido pelo scaffold).

---

## 1. DESCRIÇÃO

Entregar a UX greenfield do DARE CLI Rust: criar um projeto novo com `dare init` (interativo ou `--non-interactive`) e reaplicar / completar o scaffold oficial com `dare bootstrap`, consumindo a API congelada de `dare-scaffold` (046) e os adapters de harness (011–014).

O problema: sem init/bootstrap, o rewrite Rust só cobre brownfield (`discover`) e deixa agentes/devs sem caminho canónico para gerar as 11 stacks + 7 AX. Quem usa: desenvolvedores e agentes IDE que precisam de um tree greenfield reproduzível. Entrega verificável: comandos CLI, reports JSON, golden trees por stack, docs de compatibilidade + DEC-048.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | `dare init` greenfield | Cria dir + scaffold + `dare.config.json` + 7 AX | Integration × ≥3 stacks |
| O-02 | Modo interativo | Prompts TTY; cancel → zero writes parciais órfãs | Unit + TTY mock |
| O-03 | `--non-interactive` | Reproduzível sem stdin; flags obrigatórias presentes | Golden CLI |
| O-04 | Flags de composição | `--stack`, `--fullstack`, `--mcp`, `--transport`, `--toolchain` | Unit parse + apply |
| O-05 | Alias `rails` → `ruby-rails-8` | CLI aceita hint/map; registry permanece sem id `rails` | Unit |
| O-06 | Instalação de harnesses | Pós-scaffold: 4 IDEs (ou seleção documentada) validados | Integration |
| O-07 | `dare bootstrap` | Lê stack de `dare.config.json`; re-scaffold idempotente | Integration |
| O-08 | `bootstrap --force` | Replace paths existentes via `force=true` no scaffold | Integration |
| O-09 | Rollback | Falha mid-init → journal/rollback; dir alvo limpo ou restaurado | Integration FS |
| O-10 | Golden trees | Snapshot/assert árvore mínima por stack (11 ou subset CI + matrix) | Golden tests |
| O-11 | Docs + DEC-048 | `docs/compatibility/cli-init.md` + `cli-bootstrap.md` (ou unificado) + DECISION-LOG | Review |
| O-12 | Ralph | `cargo test`/`clippy` workspace afetado + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Greenfield parity com TS 3.18.1 no Ciclo 18 |
| Tech Lead | DARE CLI Rust | CLI fino; domínio em `dare-scaffold` / `dare-harness`; DEC-048 |
| Engenheiro | Consumidor | `dare init my-app --stack rust-axum --non-interactive` |
| Agente IDE | Cursor/Claude/Codex/Antigravity | Harnesses instalados + AX Discovery |
| Compat | Baseline TS | Diffs Classe A/B/C em flags/prompts/paths |
| Segurança | — | Path jail; sem secrets; rollback; non-interactive sem prompts inseguros |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Comando `dare init [nome]` | MUST | Registado no CLI; help en-US; capability matrix atualizada |
| RF-02 | Nome do projeto | MUST | Arg ou prompt; valida `PROJECT_NAME_RE` (`^[a-z][a-z0-9_-]{0,63}$`); InvalidInput se inválido |
| RF-03 | Diretório alvo | MUST | `./{nome}` relativo ao cwd (ou `-d` se Blueprint alinhar a padrões globais); fail-fast se existe e `!force` |
| RF-04 | Modo interativo | MUST | Em TTY sem `--non-interactive`: prompts para stack (e fullstack/mcp/transport/toolchain quando aplicável) |
| RF-05 | `--non-interactive` | MUST | Sem prompts; exige `--stack` **ou** `--mcp` (Blueprint congela exclusividade); falta → Usage exit 2 |
| RF-06 | `--stack <id>` | MUST | Um dos 11 ids canónicos; `rails` mapeado para `ruby-rails-8` na camada CLI com mensagem/log claro |
| RF-07 | `--mcp <lang>` | MUST | Resolve para `mcp-node-ts` \| `mcp-python` \| `mcp-rust` \| `mcp-go` (mapa Blueprint); incompatível com `--stack` backend sem regra documentada |
| RF-08 | `--transport` | MUST | `stdio` \| `http` \| `sse`; default MCP = `stdio`; backend ignora ou rejeita (Blueprint congela) |
| RF-09 | `--toolchain` | MUST | `none` \| `docker` (alinhado a `dare-scaffold::Toolchain`); default `none` |
| RF-10 | `--fullstack <frontend>` | MUST | Compõe backend `--stack` + frontend `react` \| `vue` (campo `frontend: Some` em `ScaffoldRequest`); sem `--stack` → InvalidInput |
| RF-11 | Desbloquear frontend em scaffold | MUST | Remover/ajustar rejeição `frontend composition reserved for 047` em `dare-scaffold` para valores suportados; artefactos frontend mínimos definidos no Blueprint |
| RF-12 | Pipeline init | MUST | Ordem: validate → create root → `run_scaffold` → write/merge config → install harnesses → validate AX → report |
| RF-13 | Instalação de harnesses | MUST | Reutiliza `dare-harness` (claude/cursor/codex/antigravity); política de quais IDEs (all vs detect) congelada no Blueprint |
| RF-14 | Report init | MUST | JSON schema versionado (camelCase): paths created, stackId, harnesses, rolledBack, check/dryRun se aplicável |
| RF-15 | `dare bootstrap` | MUST | Exige `dare.config.json` com `stack` (ou campo equivalente documentado); `ProjectRoot` = cwd/`-d` |
| RF-16 | Bootstrap sem `--force` | MUST | Idempotente: paths existentes → skip/fail-fast alinhado a scaffold (`path already exists` **ou** skip só meta — Blueprint congela uma política) |
| RF-17 | `bootstrap --force` | MUST | `ScaffoldRequest.force = true` → Replace; backups via journal scaffold |
| RF-18 | `bootstrap --toolchain` | MUST | Sobrescreve toolchain do request; persiste em config se Blueprint exigir |
| RF-19 | Rollback init | MUST | Qualquer falha após primeira escrita: rollback scaffold + remover dirs/ficheiros criados na sessão; exit ≠ 0 |
| RF-20 | Idempotência bootstrap | MUST | Segunda execução sem `--force` não corrompe tree; report `skipped`/`created` coerente |
| RF-21 | Golden trees | MUST | Fixtures por stack (≥3 em CI obrigatório; 11 no closeout ou matrix documentada): assert paths MUST (AX + `dare.config.json` + skeleton) |
| RF-22 | Exit codes | MUST | Alinhados DEC-005: 0 ok; 2 usage; 3 not found; 4 invalid input; 5 io; 1 internal/rollback grave |
| RF-23 | `--json` / `--no-color` | MUST | Globals 004; erros JSON em stdout quando `--json` |
| RF-24 | Docs | MUST | `docs/compatibility/cli-init-bootstrap.md` (ou `cli-init.md` + `cli-bootstrap.md`) + pointer em scaffold-contracts |
| RF-25 | DEC-048 | MUST | Append-only `docs/DECISION-LOG.md`; matriz 047 → Concluído no closeout |
| RF-26 | Capabilities | MUST | IDs `dare-init` / `dare-bootstrap` (ou um capability) com `cli_commands: ["init"]` / `["bootstrap"]` |
| RF-27 | Dry-run / check | SHOULD | `init --check` e/ou `bootstrap --check` zero-write (reusa `run_scaffold` check) |
| RF-28 | Banner | COULD | Banner welcome só em TTY (paridade TS); respeita `DARE_NO_BANNER` / `--no-banner` |

### 4.1 Fluxo proposto (Blueprint congela)

```text
dare init [nome] [--stack|--mcp] [--fullstack] [--transport] [--toolchain] [--non-interactive] [--force] [--check]
  → resolve InitRequest
  → ensure empty/force target dir
  → dare_scaffold::run_scaffold(ProjectRoot, ScaffoldRequest)
  → persist dare.config.json (schemaVersion 1 + projectName + stack + toolchain + frontend?)
  → install harnesses (dare-harness / shared install helpers)
  → validate_stack_output
  → InitReport

dare bootstrap [--force] [--toolchain] [--check] [-d DIR]
  → load dare.config.json → stack_id (+ frontend/toolchain)
  → run_scaffold / apply
  → BootstrapReport
```

### 4.2 Mapa `--mcp` → stack id (proposta)

| `--mcp` | Stack id |
|---------|----------|
| `ts` / `node` / `typescript` | `mcp-node-ts` |
| `python` / `py` | `mcp-python` |
| `rust` | `mcp-rust` |
| `go` | `mcp-go` |

🟡 Aliases exactos TS 3.18.1 a confirmar no Blueprint; diffs Classe B documentados.

### Fora de escopo (ver §10)

- Hooks on-save / steering files (**048**)
- Download/`npm install`/`cargo generate` da stack alvo além do skeleton embutido
- Frontend-only init sem backend (`react`/`vue` como único `--stack`)
- Neo4j / GraphRAG / dashboard

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Mesmas flags → mesma árvore (paths ASC, JSON canónico) | Golden |
| RNF-02 | Performance | `init --non-interactive` de 1 stack em tempdir < 5 s CI típico | Soft smoke |
| RNF-03 | Portabilidade | Win/macOS/Linux; paths via `SafeRelativePath` | Cross-plat |
| RNF-04 | UX | Mensagens de erro en-US; help lista stacks ASC | Snapshot help |
| RNF-05 | Observabilidade | Reports camelCase; tracing spans `init`/`bootstrap` | Unit |
| RNF-06 | Compat | Paridade observável com TS 3.18.1 em flags e árvore mínima | Diff table DEC |
| RNF-07 | Isolamento | Lógica pesada em libs; `commands/*.rs` orquestra | Review |
| RNF-08 | Idempotência | Bootstrap ×2 sem `--force` estável | Integration |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar nome, stack id, transport, toolchain, paths antes de escrever | OWASP A03 |
| RS-02 | Não gerar secrets reais; `.env.example` só placeholders; redigir logs | OWASP A02 |
| RS-03 | Toda escrita sob `ProjectRoot` / path jail; negar escape | OWASP A01 |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL novas no closeout | OWASP A06 |
| RS-05 | Sem secrets em código/fixtures; env só para config de processo se necessário | Supply chain |
| RS-06 | Reusar secret scan do scaffold em AX/templates | dare-ax Defense |
| RS-07 | Rollback obrigatório em falha parcial de init/bootstrap | Integridade |
| RS-08 | Sem shell concatenado; harness install via APIs FS / argv separado | Process safety (006) |
| RS-09 | `--non-interactive` não lê stdin (evita hang/injection em CI) | Hardening |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | workspace `1.85+` |
| CLI | `dare-cli` (`clap`) | `0.1.0-alpha.0` |
| Scaffold | `dare-scaffold` | 046 / DEC-047 |
| Harnesses | `dare-harness` | 011–014 |
| Config / contracts | `dare-contracts`, `dare-config` | workspace |
| Project helpers | `dare-project` (install patterns) | 018–019 |
| Assets | `dare-assets` | workspace |
| FS / erros | `dare-core` | workspace |
| Testes | `tempfile`, assert FS / snapshots | workspace |
| Baseline | `@dewtech/dare-cli` | 3.18.1 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem projeto alvo | Local | FS | R/W | Tree scaffold + harnesses | CLI |
| `dare-scaffold` | Crate | Rust API | Chamada | Plan/apply/validate | Workspace |
| `dare-harness` | Crate | Rust API | Chamada | Install/validate IDE files | Workspace |
| Stdin (TTY) | Local | Interactive | Entrada | Respostas de prompt | CLI init |
| Baseline TS 3.18.1 | Referência | — | Comp. | Golden trees / flags | Compat |
| npm/crates.io/PyPI | — | — | — | **Fora** (só skeleton embed) | — |

---

## 9. RESTRIÇÕES

- Pré-requisitos **011–015**, **022**, **046** considerados satisfeitos na linha Rust atual.
- Não alterar contratos de disco breaking sem ADR + migration note.
- `frontend` em scaffold limitado a `react` \| `vue` como composição; sem stack frontend-only neste ciclo.
- Conteúdo gerado = skeleton MVP + 7 AX (046), não app de produção completa.
- Um DEC (**048**); classificar diffs vs TS (prompts inquirer → clap/dialoguer ou similar).
- Orçamento de UX: interativo mínimo (stack + opções condicionais); sem wizard multi-página.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| Hooks / steering | Microplano **048** |
| `dare new` como alias | Não no Mestre §36 |
| Scaffold de stacks fora das 11 | Registry fechado 046 |
| Instalação de deps da stack alvo (`npm i`, `cargo add`) | Complexidade / rede; skeleton only |
| Dashboard / MCP server DARE | **051/052** |
| Self-update do binário | **053** |
| Enrichment IA no init | **050** / fora |
| Migração brownfield | Já coberto por `discover` / `migrate` |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Divergência flags/prompts vs TS 3.18.1 | Alta | Médio | Tabela Classe A/B/C no DEC-048 + golden CLI |
| R-02 | `--fullstack` exige templates frontend ainda inexistentes | Média | Alto | Blueprint define artefactos mínimos React/Vue ou reduz MUST→SHOULD com stub |
| R-03 | Conflito política bootstrap (fail-fast vs skip) vs discover | Média | Médio | Congelar uma política; testes de idempotência |
| R-04 | Half-written project dir em falha | Baixa | Alto | Journal + rollback + delete session root |
| R-05 | Harness install duplica lógica de `discover` | Média | Médio | Extrair helper partilhado; não copiar-colar |
| R-06 | Non-interactive incompleto em CI agentes | Média | Alto | Validação estrita de flags + exit 2 claro |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Requisitos funcionais revisados e priorizados (esp. RF-10/11 fullstack e RF-16 política bootstrap)
- [ ] Requisitos de segurança validados pelo Tech Lead
- [ ] Stack técnica aprovada (CLI + reuse scaffold/harness)
- [ ] Integrações externas confirmadas (sem registries externos)
- [ ] Fora do escopo alinhado com Product Owner
- [ ] Riscos críticos com mitigação definida
- [ ] Confirmar DEC id **048** (não reutilizar 047)
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-047-init-e-bootstrap.md`

---

## Notas Analyst → PM (passagem única)

### Analyst 🟡/🔴

| Kind | Item | Marcação |
|------|------|----------|
| scope | Init cria projeto **novo**; bootstrap opera em projeto **existente** com config | 🟢 Mestre §36 + microplano |
| ambiguity | Exact CLI library for prompts (dialoguer vs custom) | 🟡 Blueprint escolhe |
| ambiguity | Política bootstrap sem force: fail-fast (como scaffold) vs skip | 🔴 Precisa decisão no Blueprint |
| gap | Inventário golden TS 3.18.1 para cada flag `--mcp`/`--fullstack` | 🔴 Confirmar na fase Blueprint |
| gap | Lista exacta de ficheiros frontend sob `--fullstack` | 🔴 Blueprint / assets novos |

### PM

- Aceite v1: **cada** das 11 stacks gera árvore válida via non-interactive; bootstrap ×2 sem corrupção; Ralph verde.
- Fullstack: MUST se templates mínimos forem entregáveis no ciclo; senão rebaixar RF-10/11 para SHOULD com DEC explícito — **não silencioso**.
