---
name: dare-layered-design
description: Enforce arquitetura estrita de 4 camadas (Handlers, Services, Repositories, Models) em todos os projetos DARE, independente de linguagem ou framework. Inspirado em "Layered Design for Ruby on Rails Applications" de Vladimir Dementyev (Evil Martians).
---

# DARE Layered Design Skill

Você é um especialista em arquitetura de software e Layered Design. Seu papel é garantir que todo projeto DARE — independente da stack — siga estritamente o pipeline **Handler → Service → Repository → Model**, sem atalhos.

## Quando usar esta skill

- Você está revisando um Pull Request e quer verificar se as camadas foram respeitadas
- Você está gerando código de scaffold (CRUD, novo módulo, novo endpoint)
- Você está auditando um projeto legado que vai começar a usar DARE
- Você está em `/dare-blueprint` decidindo a estrutura de pastas

## As 4 camadas (regra única, sem exceção)

```
┌──────────────────────────────────────────┐
│  Handler  (HTTP, gRPC, CLI, queue worker) │ ← entrada
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  Service  (lógica de negócio, 1 operação) │ ← coração
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  Repository  (acesso a dados, abstração)  │ ← I/O
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  Model  (entidade pura, sem HTTP/DB)      │ ← domínio
└──────────────────────────────────────────┘
```

### Handlers (controllers, routers, routes)

**Responsabilidades**
- Receber request HTTP/gRPC/CLI
- Validar input (delegar para FormRequest/Pydantic/Zod/serde)
- Chamar **um** service
- Retornar response formatada

**NUNCA**
- Acessar Repository diretamente
- Conter lógica de negócio
- Fazer query em banco
- Instanciar Service com `new` (use injeção)

### Services (use_cases, interactors, commands)

**Responsabilidades**
- Implementar **uma** operação de negócio (`RegisterUser`, `RefundPayment`)
- Orquestrar Repositories
- Validar regras de negócio
- Lançar exceções de domínio (`UserAlreadyExists`, `InsufficientFunds`)

**NUNCA**
- Saber sobre HTTP (status code, headers)
- Saber sobre framework web
- Conter SQL inline (usa Repository)

### Repositories (repos, data_access, stores)

**Responsabilidades**
- Abstrair persistência (Postgres, Redis, S3, Stripe API)
- Retornar Models ou primitivos
- Esconder detalhes de SQL/HTTP externo

**NUNCA**
- Retornar HTTP status code
- Lançar exceções de domínio (retorna `Option<Model>` ou `null`)
- Conter regra de negócio

### Models (entities, domain, structs)

**Responsabilidades**
- Representar entidade do domínio
- Conter invariantes (`Email` válido, `Money` positivo)

**NUNCA**
- Conter referência a HTTP, DB, framework
- Importar de Repository/Service/Handler

## Métricas obrigatórias

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | 100% dos Services têm testes unitários (sem DB/HTTP real) | linter checa `tests/services/*` |
| M-02 | 0% de chamadas Handler→Repository diretas | linter AST/grep |
| M-03 | 100% dos Handlers usam injeção (sem `new Service()` inline) | linter AST/grep |
| M-04 | 100% dos Repositories são agnósticos das camadas superiores (não retornam HTTP status) | linter |

## Tabela tradução por linguagem

| Camada DARE | Laravel | NestJS | FastAPI | Rails | Rust/Axum | Go/Gin |
|---|---|---|---|---|---|---|
| Handler | `Http/Controllers/` | `*.controller.ts` | `routers/*.py` | `app/controllers/` | `handlers/*.rs` | `handlers/*.go` |
| Service | `Services/` | `*.service.ts` | `services/*.py` | `app/services/` | `services/*.rs` | `services/*.go` |
| Repository | `Repositories/` | `*.repository.ts` | `repositories/*.py` | `app/repositories/` | `repositories/*.rs` | `repositories/*.go` |
| Model | `Models/` | `entities/*.ts` | `models/*.py` | `app/models/` | `domain/*.rs` | `models/*.go` |

## Antipatterns

| AP | Antipattern | Sinal | Correção |
|---|---|---|---|
| AP-01 | Handler chama Repository | `controller.repo.find()` | Criar Service entre eles |
| AP-02 | Service faz SQL inline | `db.query("SELECT...")` em Service | Mover para Repository |
| AP-03 | Repository lança exceção de domínio | `throw UserNotFound` em Repo | Retornar `null`/`Option`/`Maybe` |
| AP-04 | Model importa Service | `Model.payment_service` | Refatorar: Service usa Model, não o contrário |
| AP-05 | God Service (tudo em uma classe) | `UserService` com 30 métodos | Quebrar em `RegisterUser`, `ResetPassword`, ... |
| AP-06 | Fat Controller | Controller > 100 linhas | Mover lógica para Service |

## Como aplicar na prática

### Passo 1: Identificar as camadas no projeto

Mapear pastas existentes para as 4 camadas. Se faltarem (ex: projeto Laravel só com Controllers + Models), planejar refactor incremental.

### Passo 2: Adicionar lint

Use ESLint plugin custom, Rubocop cop custom, clippy lint custom, ou apenas grep em CI:

```bash
# Detecta Handler→Repository direto
grep -rn "import.*Repository" src/controllers/ && exit 1
```

### Passo 3: Cobrir Services com testes unitários

Service NÃO depende de DB nem HTTP real — usa Repository mockado. Suite roda em milissegundos.

```typescript
// Exemplo NestJS
describe('RegisterUserService', () => {
  it('falha se email já existe', async () => {
    const repo = { findByEmail: jest.fn().mockResolvedValue({id: 1}) };
    const sut = new RegisterUserService(repo as any);
    await expect(sut.execute({email: 'x@y.com'})).rejects.toThrow('email exists');
  });
});
```

### Passo 4: Validar com Ralph Loop

A cada task:
- `npm test` / `cargo test` / `pytest` — Services 100% verde
- `npm run lint` — sem AP-01 a AP-06

## Boas práticas

1. **Uma operação = um Service** — facilita teste, reuso e nomeação
2. **Repository devolve Model, não DTO** — DTO é responsabilidade do Handler/Presenter
3. **Models são imutáveis quando possível** — mudanças passam por Service
4. **Presenter como 5ª camada opcional** — converte Model → JSON/XML

## Dicas para melhor resultado

- **Leia** `docs/design/skills/dare-layered-design/DESIGN.md` para o "porquê" e exemplos detalhados
- **Combine** com `dare-feature-design` ao adicionar feature em código legado — siga as 4 camadas mesmo se o legado não seguir
- **Templates de scaffold** estão em `packages/skills/dare-layered-design/` por linguagem

---

Esta skill é parte do DARE Method e está sob licença MIT.
