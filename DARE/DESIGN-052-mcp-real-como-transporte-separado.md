# DESIGN: MCP real como transporte separado (Microplano 052)

> **Versão:** v1.0 | **Data:** 2026-07-30 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/052-mcp-real-como-transporte-separado.md`  
> **Referência:** Documento Mestre §40 Ciclo 22 · ADR-004 · baseline TS `@dewtech/dare-cli@3.18.1` · REST/dashboard **051** (DEC-052) · path/process **005/006** · GraphRAG **040+** · steering **048** · skill MCP stacks scaffold (**046**) · próximo **053** (self-update)  
> **Posição:** 52 de 56  
> **Arquivo:** `DARE/DESIGN-052-mcp-real-como-transporte-separado.md`  
> **Escopo deste ciclo:** transporte **MCP real** (JSON-RPC) em `dare-server` · SDK **`rmcp`** · services de domínio compartilhados com REST · tools MCP · **stdio** · **streamable-http** · mapeamento de erros · testes com cliente MCP · CLI **`dare mcp serve`** · janela de transição alias **`dare-mcp-server`** (ADR-004) · docs + **DEC-053**.  
> **Não** reescrever REST/dashboard (**051**). **Não** substituir silenciosamente REST↔MCP. **Não** self-update (**053**). DEC proposto: **DEC-053** (DEC-052 = dashboard/REST **051**).

---

## 1. DESCRIÇÃO

Adicionar o **Model Context Protocol (MCP)** de verdade ao CLI Rust — JSON-RPC sobre **stdio** e, se aprovado no Blueprint, **streamable HTTP** — sem alterar nem reutilizar o contrato HTTP REST legado entregue no microplano 051.

O problema: o binário histórico `dare-mcp-server` e o REST Axum atual **não** implementam MCP; agentes IDE que esperam tools/resources MCP não conseguem falar o protocolo correto. Quem usa: developers e agentes (Claude/Cursor/Codex) que conectam um servidor MCP ao projeto DARE. Entrega verificável: módulo MCP em `dare-server`, services compartilhados, CLI `dare mcp serve`, testes com cliente MCP, e **DEC-053** — com REST 051 intacto (ADR-004).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Escolher SDK | `rmcp` (ou equivalente documentado) no workspace | Pin no Blueprint |
| O-02 | Services de domínio | Project/Graph/Dag/Task/Steering service APIs tipadas | Unit |
| O-03 | Tools MCP | Cliente lista e invoca tools canónicas | MCP client test |
| O-04 | Transporte stdio | `dare mcp serve --transport stdio` | Smoke + client |
| O-05 | Streamable HTTP | `dare mcp serve --transport streamable-http` (se MUST no Blueprint) | Integration |
| O-06 | Erros → MCP | Domínio → códigos/mensagens MCP estáveis en-US | Unit |
| O-07 | REST intacto | Suite HTTP 051 continua verde | Regression |
| O-08 | Services compartilhados | REST e MCP chamam a mesma camada de domínio (não duplicar lógica) | Review + tests |
| O-09 | Path safety | Escape / `.env*` / id inválido rejeitados no MCP | Security test |
| O-10 | Alias transição | Wrapper/`dare-mcp-server` com janela ADR-004 documentada | Docs + smoke |
| O-11 | Docs + DEC-053 | `docs/compatibility/cli-mcp.md` + DECISION-LOG | Review |
| O-12 | Ralph close | test/clippy server+cli + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | MCP real sem quebrar REST 051 |
| Tech Lead | DARE CLI Rust | ADR-004; DEC-053; rmcp; services |
| Engenheiro | Consumidor | `dare mcp serve --transport stdio` |
| Agente IDE | Claude/Cursor/Codex | tools/list + tools/call estáveis |
| Segurança | — | loopback HTTP; path jail; redact |
| Compat | Baseline / ADR-004 | Sem substituição silenciosa |
| CI / Release | Pipelines | testes MCP sem IDE externa |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | SDK MCP | MUST | Adotar **`rmcp`** (oficial modelcontextprotocol/rust-sdk) com features `server` + transportes necessários; pin exato no Blueprint; alternativa só com DEC explícito |
| RF-02 | Fronteira crate | MUST | MCP em `crates/dare-server` (`mcp.rs` + services); CLI thin; **sem** dep `dare-ai`/`dare-agent` |
| RF-03 | Feature Cargo | SHOULD | Feature `mcp` em `dare-server` (default **on** ou off — Blueprint congela) para isolar `rmcp` se necessário |
| RF-04 | `ProjectService` | MUST | Snapshot projeto (root, dareDir, config, backend, graphPresent) — paridade lógica com `GET /project` |
| RF-05 | `GraphService` | MUST | locate / traverse / map-requirement (mesmo subset 051) |
| RF-06 | `DagService` | MUST | Carregar/serializar DAG (`dare-dag.yaml`) |
| RF-07 | `TaskService` | MUST | get + put status TASKS.md (mesmas regras emoji/wire de 051) |
| RF-08 | `SteeringService` | MUST | show/list via `dare-steering`; deny `.env*` |
| RF-09 | Camada compartilhada | MUST | Handlers REST 051 passam a delegar aos services (refactor mínimo) **ou** services encapsulam as mesmas funções já usadas — zero divergência de regras de negócio |
| RF-10 | Tools MCP | MUST | Expor tools canónicas alinhadas ao domínio (não às rotas HTTP): pelo menos project, blueprint, dag, task_get, task_put, context_query, graph_locate, graph_traverse, graph_map_requirement, steering_show — lista congelada no Blueprint |
| RF-11 | `tools/list` | MUST | Cliente MCP descobre todas as tools v1 com schemas JSON |
| RF-12 | `tools/call` | MUST | Invocação com args validados; resultado estruturado (JSON text content) |
| RF-13 | Resources/Prompts | COULD | Resources MCP opcionais v1; prompts MCP fora ou mínimo |
| RF-14 | Transporte **stdio** | MUST | `dare mcp serve --transport stdio` (default se omitido — Blueprint confirma) |
| RF-15 | Transporte **streamable-http** | MUST | `dare mcp serve --transport streamable-http` com bind loopback default; token/auth alinhado ADR-004 / padrões 051 quando HTTP |
| RF-16 | CLI `dare mcp` | MUST | Subcomando `serve`; help lista transports; flags `-d`, `--bind`, `--port` onde aplicável |
| RF-17 | Mapear erros | MUST | `CoreError` → erro MCP tipado (invalid params / not found / internal); mensagens en-US; sem stack/secrets |
| RF-18 | Path safety | MUST | SafeRelativePath / ProjectRoot em todo I/O dos services |
| RF-19 | Regression REST | MUST | Testes HTTP 051 (health/tools/403/auth/413/telemetry) permanecem verdes |
| RF-20 | Testes cliente MCP | MUST | Suite com cliente `rmcp` (ou mock transport) cobre list + call happy + erro |
| RF-21 | Alias `dare-mcp-server` | SHOULD | Wrapper/binário de transição **não** substitui silenciosamente REST por MCP; comportamento congelado no Blueprint (proposta Analyst: default REST + stderr deprecation apontando `dare mcp serve` / `dare server --protocol rest`) |
| RF-22 | Capability | MUST | Capability `dare-mcp` **ou** atualização documentada → `cli_commands:["mcp"]` + manifest hash (Blueprint escolhe id sem quebrar matriz) |
| RF-23 | Docs + DEC-053 | MUST | `docs/compatibility/cli-mcp.md` + DEC-053 append-only; matriz 052 Concluído |
| RF-24 | Mensagens en-US | MUST | Erros/tool descriptions em inglês |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare mcp serve --transport stdio [-d <dir>]
dare mcp serve --transport streamable-http [--bind <ip>] [--port <n>] [-d <dir>]
```

### Princípio ADR-004 (inegociável)

| Proibido | Exigido |
|----------|---------|
| Trocar REST por MCP no mesmo URL/comando sem opt-in | Transportes nomeados (`rest` vs `mcp` / `stdio` / `streamable-http`) |
| Fazer `dare-mcp-server` virar MCP sem janela | Deprecação + guia + testes de paridade se alias existir |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | Streamable-http default loopback | Bind 127.0.0.1 |
| RNF-02 | Segurança | Token off-loopback em HTTP MCP | Paridade 051 |
| RNF-03 | Performance | tools/call locais | p95 < 500 ms tipico |
| RNF-04 | Disponibilidade | Processo local single-user | N/A cloud SLA |
| RNF-05 | Observabilidade | Logs human: transport, bind/port; sem token value default | Startup |
| RNF-06 | Manutenibilidade | Services testáveis sem spawn MCP | Unit isolado |
| RNF-07 | Compat | Linux / macOS / Windows (stdio) | Smokes |
| RNF-08 | Determinismo | Ordem de tools/list estável | Contract |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar args de tools (schema/tipos/limites) antes de I/O | OWASP A03 |
| RS-02 | Não logar tokens/secrets; redact erros MCP | OWASP A02 |
| RS-03 | Path jail; steering deny `.env*`; task id safe | OWASP A01 |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL (incl. `rmcp`) | OWASP A06 |
| RS-05 | Bind/token/port via env/flags — nunca hardcoded | Supply chain |
| RS-06 | Sem shell concatenado em qualquer spawn | Process 006 |
| RS-07 | Body/arg size limits em streamable-http | DoS |
| RS-08 | Stdio: não echo secrets; não escrever fora do ProjectRoot | Isolamento |
| RS-09 | Separação REST≠MCP documentada (ADR-004) | Governance |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | `rust-toolchain.toml` / 1.85+ |
| MCP SDK | `rmcp` | pin no Blueprint (oficial; features server + stdio + streamable-http) |
| Runtime | Tokio | já workspace (051) |
| HTTP (streamable) | Axum / hyper via rmcp transports | alinhar pins 051 |
| Domínio | `dare-server` services + `mcp.rs` | novo |
| Graph / DAG / Steering | crates existentes | 040+ / 026 / 048 |
| CLI | `dare-cli` thin `commands/mcp.rs` | clap |
| Testes | cliente rmcp + unit services | |
| Docs | `docs/compatibility/cli-mcp.md` | + DEC-053 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Cliente MCP (IDE) | Agent | MCP JSON-RPC stdio | Bidirecional | tools/list/call | `dare mcp serve` |
| Cliente MCP HTTP | Agent | Streamable HTTP | Bidirecional | idem | streamable-http |
| Filesystem projeto | Local | FS | R/W (tasks) | DARE/*, .dare/* | services |
| REST Axum 051 | Interno | HTTP | — | **Não** redirecionar | ADR-004 |
| Alias `dare-mcp-server` | Compat | process | — | Janela transição | RF-21 |
| Baseline TS | Referência | — | Comp. | nome histórico apenas | Compat |

---

## 9. RESTRIÇÕES

- Pré-requisitos: **051** DONE · **ADR-004** Accepted.
- Um DEC (**053**); não reabrir DEC-052 (dashboard/REST).
- Proibido breaking silencioso REST↔MCP.
- Docker fase omitida (padrão CLI).
- Servidor local/dev — não multi-tenant cloud.
- Pin `rmcp` pode puxar deps extras — auditar no Ralph.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| Reimplementar REST/dashboard | Já **051** |
| OAuth MCP / remote auth cloud | Fora produto CLI local |
| Self-update / package managers | **053** |
| Prompts/Resources MCP completos | COULD diferido |
| Substituir `dare server --protocol rest` | ADR-004 |
| Publicar crate MCP separado no crates.io | N/A |
| Paridade pixel com SDK TS `@modelcontextprotocol` | Class B/C documentado |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Confusão REST vs MCP (nome legado) | Alta | Alto | ADR-004; docs; help; alias com deprecation |
| R-02 | Duplicar lógica REST/MCP | Alta | Médio | Services obrigatórios RF-09 |
| R-03 | `rmcp` pin / breaking API | Média | Médio | Pin `=`; feature flag; audit |
| R-04 | Streamable-http auth frouxa | Média | Alto | Reusar padrões 051 loopback/token |
| R-05 | Alias vira MCP sem aviso | Média | Alto | RF-21 + janela documentada |
| R-06 | Matriz capabilities 49 entries | Média | Baixo | Atualizar id existente ou exception documentada |
| R-07 | Stdio flaky em CI Windows | Média | Médio | Timeouts; mocks; skip condicional documentado |
| R-08 | Tool schema drift vs REST JSON | Alta | Médio | Congelar schemas no Blueprint + golden |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] `rmcp` como SDK MUST alinhado
- [ ] stdio MUST + streamable-http MUST (ou rebaixar streamable a SHOULD) alinhado
- [ ] Lista de tools v1 aceita (ou fechada no Blueprint)
- [ ] Compartilhamento de services com REST 051 alinhado
- [ ] Comportamento do alias `dare-mcp-server` (RF-21) aceito
- [ ] DEC id **053** confirmado (052 = dashboard)
- [ ] ADR-004 respeitado (sem substituição silenciosa)
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-052-mcp-real-como-transporte-separado.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | MCP real separado; REST 051 intacto | 🟢 Microplano 052 · ADR-004 · Mestre §40 |
| ambiguity | Lista exacta de tool names/schemas | 🔴 Blueprint |
| ambiguity | Alias `dare-mcp-server` → REST+deprecation vs MCP+janela | 🟡 proposta: REST+deprecation (mais seguro ADR-004) |
| ambiguity | Feature Cargo `mcp` default on/off | 🟡 Blueprint (prefer default on se binário único) |
| ambiguity | Capability id novo vs reutilizar | 🟡 proposta: `dare-mcp` → `["mcp"]` se matriz permitir |
| gap | Inventário tools vs anúncio `/tools` REST | 🔴 Blueprint (não precisam 1:1 de nomes) |
| gap | Versão protocolo MCP alvo (2025-11-25 vs draft) | 🟡 seguir default estável do `rmcp` pin |

### PM

- Aceite v1: cliente MCP lista+chama tools; REST regression verde; services compartilhados; stdio + streamable-http; docs DEC-053; Ralph verde.
- Preferir alias **não** virar MCP silenciosamente.
- Congelar tool set e schemas no Blueprint antes de `/dare-tasks`.

---

## Próximas etapas

1. Revisar e aprovar este Design (especialmente RF-15 streamable-http MUST, RF-21 alias, tool set).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-052-mcp-real-como-transporte-separado.md`.
