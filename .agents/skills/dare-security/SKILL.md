---
name: dare-security
description: Diretrizes de Segurança DARE — OWASP Top 10, supply chain, secrets, dependências vulneráveis. Aplicável em todas as fases (Design → Blueprint → Tasks → Execute). Inclui validation gates por stack (npm audit, cargo audit, pip-audit, composer audit, govulncheck).
---

# DARE Security Skill

Você é um especialista em AppSec. Seu papel é garantir que **Design → Blueprint → Tasks → Execução** sigam rigorosamente práticas OWASP, supply chain seguro, gestão de secrets e auditoria contínua de dependências.

## Quando usar

- Início de projeto — definir RS-* (requisitos de segurança) no DESIGN.md
- Adição de dependência nova — auditar CVE
- PR mexe em autenticação, autorização, criptografia ou input externo
- Audit de produção — varredura periódica de toda a base

## Aplicação por fase DARE

### Fase 1 — Design (`dare-design`)

Requisitos obrigatórios em seção RS-*:

| ID | Requisito |
|---|---|
| RS-01 | Validação de entrada (OWASP A03) |
| RS-02 | Hash de senhas / proteção de dados sensíveis (A02) |
| RS-03 | Controle de acesso por recurso (A01) |
| RS-04 | Auditoria de dependências sem CVE HIGH/CRITICAL (A06) |
| RS-05 | Secrets via env, nunca em código |

Identifique vetores de ataque na ideia inicial e mitigações em **Riscos**.

### Fase 2 — Blueprint (`dare-blueprint`)

- Endpoints: coluna `Auth` (JWT/apiKey/público) + middleware de rate limit
- Modelo de dados: marque campos sensíveis (PII, tokens, hashes) e como são protegidos
- Fase N-1 = **Auditoria de Segurança e Dependências** com critério DONE
- Validation gates por stack incluem comando de audit

### Fase 3 — Tasks (`dare-tasks`)

- Toda task que adiciona dep → validation gate inclui `npm audit` / `cargo audit` / etc.
- Task dedicada: headers de segurança, rate limit, scan de secrets
- Seção "Considerações de Segurança" obrigatória em cada `EXECUTION/task-*.md`

### Fase 4 — Execute (`dare-execute`)

Aplique as proteções abaixo ao implementar.

## OWASP Top 10 — Implementação

### A01 — Broken Access Control

- Verifique permissão no **recurso**, não só na rota
- Princípio do menor privilégio (tokens com escopos mínimos)
- IDs sequenciais expostos = ruim — use UUID/ULID
- Multi-tenant: **sempre** filtre por `tenant_id`/`org_id`

```rust
// ✅ Rust/Axum — extractor verifica ownership
async fn update_post(
    State(db): State<Pool<Postgres>>,
    claims: Claims,
    Path(post_id): Path<Uuid>,
    Json(body): Json<UpdatePostBody>,
) -> Result<Json<Post>, AppError> {
    let post = sqlx::query_as!(Post,
        "SELECT * FROM posts WHERE id = $1 AND author_id = $2",
        post_id, claims.sub
    ).fetch_one(&db).await?;
    // ...
}
```

### A02 — Cryptographic Failures

- Senhas: **Argon2id** preferido, ou Bcrypt cost ≥ 12 — nunca MD5/SHA1/SHA256 puro
- Dados sensíveis at rest: AES-256-GCM
- Trânsito: HTTPS + HSTS
- Nunca logue: senha, token, chave de API, cartão, CPF completo
- JWT: RS256 (chave assimétrica) para tokens públicos, HS256 + segredo ≥ 256 bits para internos

```python
# Python — passlib Argon2
from passlib.hash import argon2
hashed = argon2.hash(password)
valid = argon2.verify(password, hashed)
```

```rust
// Rust — argon2 crate
use argon2::{Argon2, PasswordHasher};
let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
```

### A03 — Injection

```typescript
// ✅ Prisma — parametrizado por padrão
const user = await prisma.user.findFirst({ where: { email } });

// ❌ NestJS — QueryBuilder com interpolação
.where(`user.email = '${email}'`)  // VULNERÁVEL
.where('user.email = :email', { email })  // OK
```

```python
# SQLAlchemy — sempre parametrizado
db.execute(select(User).where(User.email == email))
```

**XSS:** escape de saída no front, CSP no back, sem `innerHTML` / `dangerouslySetInnerHTML` com user data.

**Command injection:**
```go
// ✅ Go — args lista, não shell string
cmd := exec.Command("convert", inputFile, outputFile)
// ❌ exec.Command("sh", "-c", "convert "+userInput)
```

**Prompt injection (LLM):**
- Separe instrução de dados com delimitadores
- Sanitize entrada antes de inserir no prompt
- Valide output do LLM com schema

### A04 — Insecure Design

- Valide no servidor sempre
- Allowlists > blocklists
- Rate limit ANTES da lógica de negócio em endpoints públicos

### A05 — Security Misconfiguration

- Stack traces detalhados só em dev
- Headers obrigatórios em prod:
  ```
  Strict-Transport-Security: max-age=31536000; includeSubDomains
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Content-Security-Policy: default-src 'self'
  Referrer-Policy: strict-origin-when-cross-origin
  ```
- CORS: nunca `*` para endpoints autenticados

### A06 — Vulnerable Components (crítico Ralph Loop)

```bash
# Por stack
npm audit --audit-level=high         # Node
cargo audit                          # Rust
pip-audit                            # Python
composer audit                       # PHP
govulncheck ./...                    # Go
docker scout cves [imagem]           # Docker
```

**Inegociável:** nenhuma dep com CVE HIGH/CRITICAL em produção sem justificativa documentada e plano de upgrade.

### A07 — Authentication Failures

- Login: máx 5 tentativas / 15 min por IP **e** por usuário
- JWT access token: `exp` ≤ 15 min
- Refresh token com rotação no servidor
- Logout invalida refresh no DB
- Senha: mín 12 chars, bloquear HaveIBeenPwned
- MFA TOTP para contas sensíveis

### A08 — Software/Data Integrity

- Valide checksum/signature de artefatos
- Nunca confie em dados do cliente para autorização
- Pin actions CI (`actions/checkout@v4`, não `@main`)
- Lockfiles commitados (`package-lock.json`, `Cargo.lock`, `composer.lock`)

### A09 — Security Logging & Monitoring

Logue (JSON estruturado, sem dados sensíveis):
- Auth: login OK/FAIL, logout, refresh, MFA challenge
- Authz: 403 com recurso + userId
- 5xx em prod com trace-id (sem stack trace completo)
- Destrutivas: delete, disable, role change

**Nunca logue:** senhas, tokens, API keys, cartões, CPF/SSN completo.

### A10 — SSRF

Para apps que fazem requests a URLs do usuário:
- Allowlist de domínios
- Bloqueie IPs privados (`127.x`, `10.x`, `172.16-31.x`, `192.168.x`, `169.254.x`)
- Bloqueie metadados de cloud (`169.254.169.254`)
- Timeout ≤ 5s, sem redirects automáticos

## Gestão de secrets

### Nunca em código

```
password = "..."
api_key = "..."
DATABASE_URL = "postgres://user:password@..."
AWS_SECRET_ACCESS_KEY = "..."
```

Configure scanner pré-commit:
```bash
pip install detect-secrets
detect-secrets scan > .secrets.baseline
detect-secrets audit .secrets.baseline
```

### Estrutura

- Dev: `.env` no `.gitignore`, `.env.example` commitado sem valores
- CI: secrets do pipeline (GitHub Actions Secrets)
- Produção: vault (HashiCorp, AWS Secrets Manager, GCP Secret Manager)
- Rotação: tokens de serviço a cada 90 dias

## Validation Gates no Ralph Loop

```bash
# 1. Audit de deps (se mudou deps)
npm audit --audit-level=high
cargo audit
pip-audit
composer audit
govulncheck ./...

# 2. Scan de secrets (tasks de config/infra/CI)
detect-secrets scan --baseline .secrets.baseline

# 3. Headers de segurança (tasks de config de server)
curl -I https://staging.example.com | grep -E "Strict-Transport|X-Frame|X-Content|Content-Security"
```

> **Gate obrigatório:** CVE HIGH/CRITICAL = task FAILED até corrigir.

## Como aplicar

### Passo 1: Audit do projeto

Rode tudo de §A06 e capture estado atual.

### Passo 2: Adicionar audit ao CI

```yaml
- name: Security audit
  run: |
    npm audit --audit-level=high     # ou cargo/pip/composer/govulncheck
```

### Passo 3: Scanner de secrets

```bash
detect-secrets scan > .secrets.baseline
# adicionar pre-commit hook ou step no CI
```

### Passo 4: Headers em prod

Adicionar middleware da stack (helmet, secure_headers, SetResponseHeaderLayer).

### Passo 5: Rate limit nos endpoints públicos

Login: 5/15min IP+user. APIs gerais: configurar limites apropriados.

## Dicas

- **Combine** com `dare-ax` (M-04 = rate limit configurado)
- **Combine** com `dare-llm-integration` para prompt injection
- **Use** `dare-quality-telemetry` para rastrear M-04 (CVE count) ao longo do tempo

---

Esta skill é parte do DARE Method e está sob licença MIT.
