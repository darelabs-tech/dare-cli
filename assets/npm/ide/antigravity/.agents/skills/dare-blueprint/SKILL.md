---
name: dare-blueprint
description: Gera um Task List estruturado a partir do Design aprovado. Use quando o usuário aprovar o DESIGN.md. Cria um documento BLUEPRINT.md com arquitetura, endpoints, modelo de dados e plano de execução.
---

# DARE Blueprint Skill

Você é um arquiteto de software especializado em design de APIs e sistemas. Seu objetivo é transformar o Design aprovado em uma arquitetura detalhada que será a base para implementação.

## Quando usar esta skill

- Design.md foi aprovado pelo usuário
- Precisa-se detalhar a arquitetura técnica
- Necessário documentar endpoints e modelos
- Segunda fase do Método DARE

> **Equivalente no terminal:** `dare blueprint --ai`

## Como usar

### Passo 1: Ler o Design Aprovado
Leia o arquivo `DARE/DESIGN.md` que foi aprovado. Extraia:
- Stack técnica
- Funcionalidades principais
- Requisitos não-funcionais
- Restrições

### Passo 1b: Trade-offs (Architect)

Antes do scaffold, leia `DARE/PATTERNS.md` e `DARE/patterns-facts.json`. Formule perguntas de trade-off **ancoradas em padrões reais** — cada pergunta **cita o `id` do DiscoveredPattern**. **1 passagem sequencial**; **sem runtime multi-agente**. Não invente padrões: só referencie os 🟢 do CLI; conclusões 🟡.

### Passo 2: Analisar Contexto
Leia os arquivos de contexto:
- `.agents/rules/dare-workflow.md` (ou `.cursorrules` se Cursor)
- Exemplos em `examples/`
- Templates em `templates/`

### Passo 3: Integrar Segurança
Consulte `skill-security` para:
- Autenticação/Autorização
- Validação de entrada
- Criptografia
- Proteção contra vulnerabilidades OWASP

### Passo 4: Gerar a Arquitetura
Crie um documento `DARE/BLUEPRINT.md` com a seguinte estrutura:

```markdown
# Blueprint: [Nome do Projeto]

## Visão Geral da Arquitetura
[Descrição da arquitetura escolhida: Monolito, Microserviços, Hexagonal, etc]

## Segurança (OWASP)
### Autenticação e Autorização
- Método: JWT com RS256
- Armazenamento: Bearer token no header
- Validação: Middleware em todos os endpoints protegidos

### Proteção de Dados
- Senhas: Bcrypt com salt
- Dados sensíveis: Encriptados em repouso
- Transmissão: HTTPS obrigatório

### Validação
- Input: Whitelist de valores permitidos
- Output: Escape de caracteres especiais
- Rate Limiting: 5 req/min por IP

## Modelo de Dados
### Tabela: users
| Campo | Tipo | Restrições |
|-------|------|-----------|
| id | UUID | PK |
| email | VARCHAR(255) | UNIQUE, NOT NULL |
| password_hash | VARCHAR(255) | NOT NULL (Bcrypt) |
| name | VARCHAR(255) | NOT NULL |
| is_active | BOOLEAN | DEFAULT true |
| created_at | TIMESTAMP | DEFAULT NOW() |
| updated_at | TIMESTAMP | DEFAULT NOW() |

### Tabela: refresh_tokens
| Campo | Tipo | Restrições |
|-------|------|-----------|
| id | UUID | PK |
| user_id | UUID | FK users.id |
| token | VARCHAR(500) | UNIQUE |
| expires_at | TIMESTAMP | NOT NULL |
| revoked_at | TIMESTAMP | NULL |
| created_at | TIMESTAMP | DEFAULT NOW() |

## Endpoints da API

| Método | Endpoint | Autenticação | Descrição |
|--------|----------|--------------|-----------|
| POST | /api/auth/register | Não | Registrar novo usuário |
| POST | /api/auth/login | Não | Login e obter JWT |
| POST | /api/auth/refresh | Não | Renovar JWT com refresh token |
| POST | /api/auth/logout | JWT | Logout e revogar tokens |
| GET | /api/users/me | JWT | Obter dados do usuário logado |

### Detalhes dos Endpoints

#### POST /api/auth/register
**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "name": "John Doe"
}
```

**Response (201):**
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "name": "John Doe",
  "created_at": "2026-04-14T10:00:00Z"
}
```

**Validações:**
- Email válido e único
- Senha: mínimo 8 caracteres, 1 maiúscula, 1 número, 1 caractere especial
- Name: mínimo 3 caracteres

## Estrutura de Diretórios

> Mantenha esta seção **stack-agnóstica**. Liste os agrupamentos lógicos
> (domínio, infraestrutura, interfaces, testes, migrations) e use a
> nomenclatura **idiomática da stack escolhida** no `dare init`. Os exemplos
> abaixo cobrem as 5 stacks suportadas — use **apenas o bloco da stack do
> projeto**, não os 5 juntos.

<details>
<summary>Exemplo — Node.js / NestJS</summary>

```
projeto/
├── src/
│   ├── auth/
│   │   ├── auth.controller.ts
│   │   ├── auth.service.ts
│   │   ├── auth.module.ts
│   │   └── dto/{register,login}.dto.ts
│   ├── users/{users.entity.ts,users.service.ts}
│   └── main.ts
├── migrations/{001_users.ts,002_refresh_tokens.ts}
└── test/auth.e2e-spec.ts
```
</details>

<details>
<summary>Exemplo — Rust / Axum</summary>

```
projeto/
├── src/
│   ├── domain/{user.rs,refresh_token.rs}
│   ├── handlers/{register.rs,login.rs,refresh.rs,logout.rs}
│   ├── middleware/jwt.rs
│   └── main.rs
├── migrations/{001_users.sql,002_refresh_tokens.sql}
└── tests/auth_integration.rs
```
</details>

<details>
<summary>Exemplo — Python / FastAPI</summary>

```
projeto/
├── app/
│   ├── routers/auth.py
│   ├── models/{user.py,refresh_token.py}
│   ├── schemas/{register.py,login.py}
│   ├── services/auth.py
│   └── main.py
├── alembic/versions/{001_users.py,002_refresh_tokens.py}
└── tests/test_auth.py
```
</details>

<details>
<summary>Exemplo — PHP / Laravel</summary>

```
projeto/
├── app/Http/Controllers/AuthController.php
├── app/Http/Requests/{RegisterRequest,LoginRequest}.php
├── app/Models/{User,RefreshToken}.php
├── app/Services/AuthService.php
├── database/migrations/{create_users,create_refresh_tokens}_table.php
├── routes/api.php
└── tests/Feature/AuthTest.php
```
</details>

<details>
<summary>Exemplo — Go / Gin</summary>

```
projeto/
├── cmd/server/main.go
├── internal/
│   ├── handlers/{register,login,refresh,logout}.go
│   ├── models/{user,refresh_token}.go
│   └── middleware/jwt.go
├── migrations/{001_users.sql,002_refresh_tokens.sql}
└── handlers_test.go
```
</details>

## Plano de Execução

### Fase 1: Setup Inicial
- Criar migrations (users, refresh_tokens)
- Configurar autenticação JWT
- Setup de testes

### Fase 2: Autenticação
- Implementar RegisterController
- Implementar LoginController
- Implementar RefreshController

### Fase 3: Proteção
- Implementar Middleware de JWT
- Implementar Rate Limiting
- Implementar Logout

### Fase 4: Testes e Deploy
- Testes unitários
- Testes de integração
- Containerização com Docker

## Comandos de Setup

> Liste **somente os comandos da stack do projeto** (definida em
> `dare init` / `dare.config.json#backend`). Não inclua todos os blocos
> abaixo — use o que casa com a stack escolhida.

<details>
<summary>Node.js / NestJS</summary>

```bash
npm install
cp .env.example .env
npm run migration:run
npm test
npm run start:dev
```
</details>

<details>
<summary>Rust / Axum</summary>

```bash
cargo build
cp .env.example .env
sqlx migrate run
cargo test
cargo run
```
</details>

<details>
<summary>Python / FastAPI</summary>

```bash
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env
alembic upgrade head
pytest
uvicorn app.main:app --reload
```
</details>

<details>
<summary>PHP / Laravel</summary>

```bash
composer install
cp .env.example .env
php artisan key:generate
php artisan migrate
php artisan test
php artisan serve
```
</details>

<details>
<summary>Go / Gin</summary>

```bash
go mod download
cp .env.example .env
migrate -path ./migrations -database "$DATABASE_URL" up
go test ./...
go run ./cmd/server
```
</details>

## Próximas Etapas
1. Revisar e aprovar este Blueprint
2. Executar `/generate-tasks DARE/BLUEPRINT.md`
3. Continuar com o Método DARE
```

### Passo 5: Aplicar ANTI-STUB CONTRACT (regra inegociável)

> **Por que existe esta regra:** a skill `dare-tasks` que vem depois usa este Blueprint como **única fonte de verdade**. Se um endpoint, função ou regra ficar genérico aqui, o agente que implementar a task **será forçado a inventar** — e vai produzir mocks, stubs e esqueletos para "preencher o vazio". Detalhe agora.
>
> Tasks que produzem mock/stub/skeleton **falham** no `dare review` (v2.17+) e bloqueiam o `dare execute --complete`.

Para **cada** endpoint, função pública, evento ou job declarado no Blueprint, especifique de forma **executável**:

**Endpoints HTTP/RPC:**
- Assinatura completa (método, path, headers obrigatórios, content-type)
- Request schema (todos os campos com tipo, restrições, opcionalidade)
- Response schema **por status code** (2xx, 4xx, 5xx — não só "200 OK")
- Validações server-side (lista exaustiva: `email único`, `senha ≥ 8 chars + maiúscula + dígito`)
- Edge cases enumerados (input vazio, duplicado, expirado, sem permissão)
- Side effects (tabelas/filas/caches/emails tocados, em ordem)
- Exemplo concreto (payload real, response real — não placeholder)

**Funções de domínio / services:**
- Assinatura tipada (`fn name(args: Types) -> ReturnType`)
- Pré-condições e pós-condições verificáveis
- Estados de erro com tipo de exceção/Result esperado
- Comportamento em concorrência (idempotência, locking, retry)

**Jobs / event handlers / workers:**
- Trigger (evento/cron/fila — nome canônico)
- Payload schema tipado
- Retry policy (backoff, max attempts, DLQ)
- Idempotência (chave + estratégia)

**Modelos de dados:**
- Cada campo com tipo, nullable, default, constraints (unique, fk, check), índices
- Triggers ou hooks (soft-delete, audit, encryption-at-rest)

**Critério "Blueprint detalhado o suficiente"** (auto-validação antes de salvar):

- [ ] Para cada endpoint, um humano não-familiarizado consegue escrever request/response sem perguntar nada?
- [ ] Para cada função pública, está claro **o que retorna** em todos os caminhos (sucesso + erros enumerados)?
- [ ] Edge cases foram **enumerados** ou só listados como "tratar edge cases"?
- [ ] Cada validação tem uma regra concreta (não só "validar email")?
- [ ] Cada decisão arquitetural tem **justificativa** (não só "escolhemos X")?

**Anti-padrão a evitar:** seções como _"implementar autenticação"_ ou _"validar dados"_ — isso vira stub. Especifique algoritmo, campos, regras.

### Passo 6: Pedir Aprovação
Após gerar o Blueprint, peça ao usuário:
- Revisar a arquitetura
- Aprovar endpoints e modelos
- Confirmar antes de continuar

## Boas Práticas

1. **Detalhado:** Inclua exemplos de request/response
2. **Seguro:** Sempre implemente proteções OWASP
3. **Escalável:** Pense em crescimento futuro
4. **Testável:** Estruture para facilitar testes
5. **Documentado:** Deixe claro para implementação

## Dicas para Melhor Resultado

- **Contexto:** Leia exemplos em `examples/` para manter padrões
- **Segurança:** Consulte `skill-security` para requisitos
- **Templates:** Use `templates/BLUEPRINT-template.md` como referência
- **Docker:** Considere incluir informações para containerização
