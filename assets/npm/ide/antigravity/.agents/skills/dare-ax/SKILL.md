---
name: dare-ax
description: Agent Experience (AX) — codifica padrões para desenvolvimento assistido por IA em três planos (Discovery, Usage, Defense). Garante que todo projeto DARE exponha sinais estruturados (llms.txt, OpenAPI, --json, rate limit) que agentes de código precisam para trabalhar sem refactor desnecessário.
---

# DARE AX Skill (Agent Experience)

Você é um especialista em integração entre projetos e agentes de código (Claude Code, Cursor, Antigravity, etc.). Seu papel é garantir que todo projeto DARE seja **navegável, usável e defensivo** quando consumido por uma IA.

## Quando usar esta skill

- Está bootstrapando um projeto novo via `dare init` e precisa do baseline AX
- Está auditando um projeto existente e quer saber se ele é "AX-friendly"
- Está adicionando uma feature que muda a superfície pública (endpoints HTTP, CLI, docs)
- Vai integrar o projeto com um agente externo (LangChain, OpenAI Agents, MCP server)

## Os três planos AX

### Plano 1: Discovery — o agente acha o que precisa

| Sinal | Onde mora | Conteúdo mínimo |
|---|---|---|
| `llms.txt` | raiz do repositório | nome do projeto, descrição em 1 parágrafo, links para docs principais, comandos `make`/`npm run`, endpoints expostos, exemplos |
| `README.md` | raiz | bootstrap em 5 minutos, link para llms.txt, link para OpenAPI |
| Estrutura previsível | `src/`, `tests/`, `docs/`, `DARE/` | sem pastas exóticas (`thing/`, `stuff/`) |

### Plano 2: Usage — o agente consegue operar

| Sinal | Obrigatório quando |
|---|---|
| `openapi.json` em `/openapi.json` ou `public/openapi.json` | projeto expõe HTTP |
| Flag `--json` em todos os comandos CLI | projeto expõe CLI |
| `Dockerfile` + `docker-compose.yml` validados | qualquer projeto que toque serviços externos |
| `.env.example` versionado | qualquer projeto |

### Plano 3: Defense — o agente não pode quebrar produção

| Defesa | Implementação típica |
|---|---|
| Rate limit | `rack-attack` (Rails), `express-rate-limit` (Node), `tower-governor` (Rust), `slowapi` (FastAPI) |
| Validação de input | FormRequests (Laravel), Pydantic (FastAPI), Zod (Node), serde + validator (Rust) |
| Secrets só via env | `.env` no `.gitignore`, `.env.example` sem valores reais |
| `llms.txt` sem secrets | scan automático no CI bloqueando padrões `api_key=`, `password=`, `DATABASE_URL=` |

## Métricas obrigatórias (todas Tipo A — binárias, verificáveis em CI)

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | `llms.txt` existe e é válido (sem secrets, seções obrigatórias presentes) | grep + parse do arquivo |
| M-02 | `public/openapi.json` ou `openapi.json` existe | `test -f` |
| M-03 | CLI suporta `--json` | rodar `cli --help \| grep '\-\-json'` |
| M-04 | Rate limit configurado | grep por `rack-attack \| express-rate-limit \| tower-governor \| slowapi` |

Se M-01 ou M-04 falhar em produção, falha o pipeline.

## Antipatterns explícitos

| ID | Antipattern | Por que evitar |
|---|---|---|
| AP-01 | Docs fora do código | Divergem rapidamente |
| AP-02 | OpenAPI escrito à mão | Fica desatualizado; sempre auto-gerar |
| AP-03 | CLI sem `--json` | Agentes precisam de parsing regex — brittle |
| AP-04 | Rate limit em dev, esquecido em prod | Produção vira target de abuse |
| AP-05 | `llms.txt` com secrets | Expõe credenciais no repositório público |
| AP-06 | CORS wildcard (`*`) em produção | Permite qualquer origem |
| AP-07 | Validação só em API, esquecida em CLI | Agentes usam ambos |
| AP-08 | Configs opcionais sem default | Agentes precisam de defaults previsíveis |

## Template `llms.txt` (gerar sempre)

```
# <nome-do-projeto>

> <descricao em 1-2 frases>

## Stack

- <linguagem + framework>
- <banco de dados>
- <cache, fila, etc.>

## Bootstrap

- `make setup` — instala deps + sobe Postgres/Redis
- `make dev` — sobe aplicação em modo dev
- `make test` — roda suite completa

## Endpoints

- `GET /healthz` — liveness
- `GET /openapi.json` — spec OpenAPI completa
- `POST /api/login` — autenticação
- ...

## Docs

- DARE/DESIGN.md — requisitos
- DARE/BLUEPRINT.md — arquitetura
- docs/RUNBOOK.md — operações
```

## Como aplicar passo a passo

### Passo 1: Auditar o projeto

Rode mentalmente o checklist:
- [ ] `llms.txt` existe na raiz?
- [ ] `openapi.json` é gerado a cada build?
- [ ] Todos os comandos CLI aceitam `--json`?
- [ ] Existe rate limit em endpoints públicos?
- [ ] `.env.example` está versionado e sem valores reais?

### Passo 2: Gerar `llms.txt` se faltar

Use o template acima. Liste **apenas** comandos e endpoints que realmente existem — não invente.

### Passo 3: Validar OpenAPI

Se a stack for:
- **Node/NestJS** — `@nestjs/swagger` decorators + `OpenAPIModule.createDocument`
- **FastAPI** — automático via Pydantic, exposto em `/openapi.json`
- **Rails/Grape** — `grape-swagger` ou `rswag`
- **Rust/Axum** — `utoipa` + `utoipa-swagger-ui`
- **Laravel** — `darkaonline/l5-swagger`

### Passo 4: Validar CLI --json

Em qualquer CLI do projeto, `--json` deve:
- Imprimir JSON puro em stdout (sem cores ANSI, sem prompt)
- Manter exit code = 0 em sucesso, ≠ 0 em erro
- Schema documentado no `llms.txt`

### Passo 5: Validar rate limit

Procurar por gem/package específico da stack. Se ausente em endpoint público autenticado → adicionar antes de release.

## Boas práticas

1. **AX como cidadão de primeira classe** — não é "feature opcional", é parte do design
2. **Single source of truth** — OpenAPI gerado a partir do código, nunca manual
3. **Defaults previsíveis** — agentes precisam adivinhar menos
4. **Logs estruturados (JSON)** — facilita debug por agente que lê output

## Dicas para melhor resultado

- **Consulte** `docs/design/skills/dare-ax/DESIGN.md` para o "porquê" detalhado
- **Use** a skill `dare-security` em conjunto — AX e segurança são primos próximos
- **Antes de release**, rode os 4 checks de M-01 a M-04 em CI

---

Esta skill é parte do DARE Method e está sob licença MIT.
