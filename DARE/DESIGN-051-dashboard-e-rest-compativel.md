# DESIGN: Dashboard e REST compatível (Microplano 051)

> **Versão:** v1.0 | **Data:** 2026-07-30 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/051-dashboard-e-rest-compativel.md`  
> **Referência:** Documento Mestre §40 Ciclo 22 · §6.1–6.2 servidores · baseline TS `@dewtech/dare-cli@3.18.1` (`dare-mcp-server` Express REST + `dare dashboard`) · DAG **026** · GraphRAG **040** · verify/bench **049** · path/process **005/006** · skill `/dare-dashboard` · próximo **052** (MCP real)  
> **Posição:** 51 de 56  
> **Arquivo:** `DARE/DESIGN-051-dashboard-e-rest-compativel.md`  
> **Escopo deste ciclo:** crate **`dare-server`** (Axum) · app compartilhado · dashboard HTML read-only · `/api/telemetry` · REST legado (`/health`, `/tools`, `/context/query`, `/blueprint`, `/dag`, `/tasks`, `/graph/*`, `/project`, `/steering`) · auth/body-limit/path-safety · bind loopback · open browser · graceful shutdown · CLI **`dare dashboard`** + **`dare server --protocol rest`** · docs + **DEC-052**.  
> **Não** transporte MCP JSON-RPC/stdio/SSE (**052**). **Não** `dare ai` (**050** já feito). DEC proposto: **DEC-052** (DEC-051 = comandos ai **050**).

---

## 1. DESCRIÇÃO

Portar para Rust/Axum o **dashboard local de telemetria** e o **servidor HTTP REST legado** que no TypeScript vive no binário mal nomeado `dare-mcp-server` (Express puro, sem protocolo MCP). Um único app Axum compartilhado serve o modo dashboard (UI + telemetry, read-only) e o modo REST (rotas de contexto/DAG/tasks/graph/project/steering) com o mesmo hardening de segurança.

O problema: sem isso, o CLI Rust não oferece superfície HTTP para IDE/ops inspecionar DAG, telemetria e contexto do projeto, nem paridade com o servidor TS 3.18.1. Quem usa: developers locais, agentes IDE e CI via contract tests. Entrega verificável: `crates/dare-server` + comandos CLI + fixtures HTTP + DEC-052.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-server` | Member workspace; sem ciclo indevido com `dare-cli` | `cargo test -p dare-server` |
| O-02 | App Axum compartilhado | Factory única; modos dashboard \| rest | Unit + integration |
| O-03 | Dashboard HTML/assets | `GET /dashboard` + static anti-traversal | HTTP smoke |
| O-04 | `GET /api/telemetry` | `TelemetrySnapshot` JSON estável | Contract test |
| O-05 | REST legado | Rotas health/tools/context/blueprint/dag/tasks/graph/project/steering | Contract suite |
| O-06 | Path safety | Escape / `..` → **403** | Security test |
| O-07 | Auth | Loopback sem token OK; non-loopback exige Bearer | Unit + HTTP |
| O-08 | Body limit | Exceder limite → **413** (ou 400 documentado) | HTTP |
| O-09 | Bind loopback | Default `127.0.0.1`; portas 4100 (dashboard) / 3000 (REST) | Smoke |
| O-10 | Open browser | Cross-platform; `--no-open` desliga | Unit stub + manual |
| O-11 | Graceful shutdown | SIGINT/Ctrl+C encerra sem panic | Integration |
| O-12 | CLI | `dare dashboard` + `dare server --protocol rest` | Help + smokes |
| O-13 | Docs + DEC-052 | `docs/compatibility/cli-dashboard-rest.md` + DECISION-LOG | Review |
| O-14 | Ralph close | clippy/test server+cli + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Paridade Ciclo 22 REST/dashboard com TS 3.18.1 |
| Tech Lead | DARE CLI Rust | Axum shared app; DEC-052; MCP fora |
| Engenheiro | Consumidor | `dare dashboard` / contract tests |
| Agente IDE | Claude/Cursor | `POST /context/query` + tools list |
| Segurança | — | Loopback default; token off-loopback; 403 path |
| CI / Release | Pipelines | HTTP contracts sem UI browser |
| Compat | Baseline TS | Diffs A/B/C de paths/status/JSON |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-server` | MUST | Workspace member; deps: axum, tower-http, tokio, dare-core, dare-dag, dare-graph, dare-steering, serde; **não** depende de `dare-agent`/`dare-ai` |
| RF-02 | App compartilhado | MUST | `create_app(mode, cfg, state) -> Router`; dashboard e REST compartilham middleware |
| RF-03 | `dare dashboard` | MUST | Bind `127.0.0.1:4100` default; flags `--port`, `--no-open`, `-d`/`--dir` |
| RF-04 | `dare server --protocol rest` | MUST | Bind `127.0.0.1:3000` default; `--protocol rest` obrigatório nesta v1 (outros → usage/2); flags `--port`, `--bind`, `-d` |
| RF-05 | Dashboard HTML | MUST | `GET /dashboard` serve HTML vanilla (embed ou assets); CSS/JS sob path jail |
| RF-06 | Assets anti-traversal | MUST | Qualquer `..` / absoluto → **403** com mensagem en-US estável |
| RF-07 | Telemetry | MUST | `GET /api/telemetry` → `TelemetrySnapshot` (dag/gates/cost/bestOfN/guard/drift — campos Blueprint) |
| RF-08 | Dashboard read-only | MUST | Nenhuma rota do modo dashboard muta grafo/estado/TASKS |
| RF-09 | `GET /health` | MUST | 200 + JSON health mínimo (ok, version) |
| RF-10 | `GET /tools` | MUST | Lista anúncio das “tools”/rotas (paridade TS — **não** é MCP) |
| RF-11 | `POST /context/query` | MUST | Body JSON tipado; query architecture/task/dependency (ou subset documentado); path-safe |
| RF-12 | `GET /blueprint` | MUST | Lê `DARE/BLUEPRINT.md` (ou path cfg) sob jail |
| RF-13 | `GET /dag` | MUST | Lê/serializa `DARE/dare-dag.yaml` estado |
| RF-14 | Tasks | MUST | `GET /tasks/:id`; `PUT /tasks/:id` **só no modo REST** (dashboard não expõe PUT) — PUT atualiza linha STATUS em TASKS.md com path safety |
| RF-15 | Graph | MUST | `POST /graph/locate` (e map-requirement/traverse se já existirem APIs em `dare-graph` — Blueprint congela subset v1) |
| RF-16 | `GET /project` | MUST | Snapshot projeto (root, stack detectada se disponível) |
| RF-17 | `GET /steering` | MUST | Query `?file=` resolve via `dare-steering` + deny `.env*` |
| RF-18 | Auth Bearer | MUST | Header `Authorization: Bearer <token>`; loopback **isento** por default; non-loopback **exige** token |
| RF-19 | Token source | MUST | Env `DARE_MCP_TOKEN` (nome legado TS) ou UUID gerado na subida; impresso no log/startup human (não em JSON público) |
| RF-20 | Body limit | MUST | Default **1 MiB** (`DARE_MCP_BODY_LIMIT` override); excesso rejeitado |
| RF-21 | Security headers | MUST | Pelo menos: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, CSP restritiva documentada |
| RF-22 | CORS | MUST | Hand-rolled / tower-http explícito; defaults loopback-friendly; documentar origins |
| RF-23 | Open browser | MUST | Ao subir dashboard (sem `--no-open`), abrir URL cross-platform via SafeCommand/argv (não shell string) |
| RF-24 | Graceful shutdown | MUST | Ctrl+C / signal → drain + exit 0 |
| RF-25 | Env bind | MUST | `DARE_MCP_BIND`, `DARE_MCP_PORT`, `DARE_PROJECT_PATH` (ou `-d`) alinhados ao TS |
| RF-26 | Contract tests | MUST | Suite HTTP (tower/axum test ou hyper) cobre happy + 403 escape + auth off-loopback |
| RF-27 | Mensagens en-US | MUST | Erros HTTP body/message em inglês |
| RF-28 | Capability | MUST | Capability dashboard/server → `cli_commands:["dashboard","server"]` (ou split documentado) + manifest hash |
| RF-29 | Docs + DEC-052 | MUST | Compat doc + DEC-052 append-only; matriz 051 Concluído |
| RF-30 | Alias binário | COULD | Wrapper/`dare-mcp-server` alias apontando para `dare server --protocol rest` — só se trivial |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare dashboard [--port <n>] [--no-open] [-d <dir>]
dare server --protocol rest [--bind <ip>] [--port <n>] [-d <dir>]
```

### Superfície HTTP (resumo)

| Modo | Rotas principais |
|------|------------------|
| dashboard | `GET /dashboard`, static assets, `GET /api/telemetry`, `GET /health` (opcional) |
| rest | Todas as acima relevantes + `/tools`, `/context/query`, `/blueprint`, `/dag`, `/tasks/:id`, `/graph/*`, `/project`, `/steering` |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | Default só loopback | Bind 127.0.0.1 |
| RNF-02 | Segurança | Token obrigatório off-loopback | 401 sem Bearer |
| RNF-03 | Performance | Telemetry/health locais | p95 < 200 ms tipico |
| RNF-04 | Disponibilidade | Processo single-user local | N/A SLA cloud |
| RNF-05 | Observabilidade | Startup log: bind, port, token present (não valor) | Human |
| RNF-06 | Manutenibilidade | Domínio em `dare-server`; CLI thin | Sem lógica HTTP em main |
| RNF-07 | Compat | Linux / macOS / Windows (browser open + bind) | Smokes onde possível |
| RNF-08 | Determinismo | JSON keys/order estável onde aplicável | Contract |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths/query/body antes de I/O; schema JSON em POST | OWASP A03 |
| RS-02 | Não logar token completo nem secrets; redact | OWASP A02 |
| RS-03 | Path jail ProjectRoot; escape → 403; steering deny `.env*` | OWASP A01 |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Token/bind/port via env — nunca hardcoded | Supply chain |
| RS-06 | Open browser via argv SafeCommand | Process safety 006 |
| RS-07 | Body limit 1 MiB default | DoS |
| RS-08 | Headers security + CSP dashboard | Clickjacking/XSS |
| RS-09 | PUT tasks só com auth quando não-loopback; validar id path-safe | A01/A03 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | `rust-toolchain.toml` |
| HTTP | Axum + Tokio | pins no Blueprint / workspace |
| Middleware | tower-http (limit, trace, set-header) | workspace |
| Domínio | `dare-server` (novo) | `rest.rs`, `dashboard.rs`, `auth.rs`, `telemetry.rs` |
| DAG / Graph / Steering | crates existentes | 026 / 040+ / 048 |
| CLI | `dare-cli` thin commands | clap |
| Assets | rust-embed **ou** files sob `assets/dashboard/` | Blueprint escolhe |
| Testes | axum `oneshot` / hyper | contract |
| Docs | `docs/compatibility/cli-dashboard-rest.md` | + DEC-052 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Browser local | UI | HTTP | Saída | Dashboard HTML | open browser |
| Filesystem projeto | Local | FS | R/(W PUT tasks) | DARE/*, .dare/* | path jail |
| Env `DARE_MCP_*` | Config | env | In | bind/port/token/limit | server |
| Baseline TS 3.18.1 | Referência | — | Comp. | rotas/status | Compat |
| MCP JSON-RPC | — | — | — | **Fora** | 052 |

---

## 9. RESTRIÇÕES

- Pré-requisitos: **026** DAG, **040** graph storage, **049** verify (telemetria gates/bestOf), path/process **005/006**.  
  - **017** `info` na matriz ainda “Pendente”, mas `dare info` 🟢 já existe no CLI — tratar como satisfeito para snapshot `/project`.
- Um DEC (**052**); não reabrir DEC-051 (ai).
- Sem `@modelcontextprotocol/sdk` / JSON-RPC neste ciclo.
- Docker fase omitida (padrão CLI 046–050).
- Servidor é **local/dev** — não produção multi-tenant.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| MCP real (stdio / streamable-http) | **052** |
| Reescrever GraphRAG / hooks / ai | Já feitos |
| Auth OAuth / multi-user | Fora do produto CLI local |
| TLS / reverse proxy | Ops externo |
| WebSocket live canvas | Não no TS 3.18.1 dashboard |
| Publicar `dare-server` no crates.io | N/A |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Confundir REST legado com MCP | Alta | Alto | Docs + nome `protocol rest`; 052 separado |
| R-02 | Path traversal em assets/steering | Média | Alto | SafeRelativePath; testes 403 |
| R-03 | Token vazado em logs/URL | Média | Alto | Redact; não colocar token na query string |
| R-04 | Divergência TelemetrySnapshot vs TS | Alta | Médio | Congelar schema no Blueprint + contracts |
| R-05 | PUT tasks corrompe TASKS.md | Média | Alto | Atomic write; validar id; tests |
| R-06 | Browser open falha no CI | Alta | Baixo | `--no-open` default em testes |
| R-07 | Axum/tokio pins conflict workspace | Média | Médio | Workspace deps únicas |
| R-08 | Pré-req 017 matriz desatualizada | Baixa | Baixo | Usar `dare info` existente |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Shared Axum app + modos dashboard/rest alinhados ao Mestre §6/§40
- [ ] Aceite: contract HTTP + 403 escape + token off-loopback
- [ ] Dashboard read-only vs PUT só em REST alinhado
- [ ] Subset de rotas `/graph/*` v1 aceito (ou fechado no Blueprint)
- [ ] DEC id **052** confirmado (051 = ai)
- [ ] Fora de escopo MCP **052** alinhado
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-051-dashboard-e-rest-compativel.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Dashboard + REST Axum; MCP fora | 🟢 Mestre §40 · microplano 051 |
| ambiguity | Campos exactos de `TelemetrySnapshot` | 🔴 Blueprint |
| ambiguity | Subset v1 de `/graph/*` (locate only vs locate+map+traverse) | 🟡 proposta: locate + traverse se API pronta |
| ambiguity | Embed assets vs filesystem `assets/dashboard/` | 🟡 Blueprint (prefer embed para binário único) |
| gap | OpenAPI/contract fixture inventory from TS | 🔴 Blueprint |
| gap | Pré-req 017 matriz vs `dare info` existente | 🟢 CLI já tem Info |

### PM

- Aceite v1: contracts HTTP verdes; 403 path escape; token obrigatório fora de loopback; dashboard read-only; `dare dashboard` + `dare server --protocol rest`; Ralph verde; DEC-052.
- Preferir **embed** de assets dashboard para distribuição single-binary.
- MCP fica explicitamente no 052 — zero JSON-RPC neste ciclo.

---

## Próximas etapas

1. Revisar e aprovar este Design (especialmente RF-14 PUT, RF-15 graph subset, schema telemetry).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-051-dashboard-e-rest-compativel.md`.
