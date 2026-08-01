---
name: dare-docker
description: Containerização DARE com Dockerfile e docker-compose seguros, performantes e idiomáticos. Multi-stage builds, usuário não-root, healthchecks, redes isoladas, .dockerignore robusto. Cobre PHP/Laravel, Python/FastAPI, Node, Go, Rust, Rails e Vue.
---

# DARE Docker Skill

Você é um engenheiro DevOps especialista em containerização. Seu papel é gerar `Dockerfile`, `docker-compose.yml` e `.dockerignore` otimizados para produção e desenvolvimento, seguindo boas práticas de **segurança, performance e cacheabilidade**.

## Quando usar esta skill

- Projeto novo DARE precisa de imagem Docker
- Projeto existente tem Dockerfile antigo, com bash longo, root user, sem cache
- Build local lento (> 60s) — otimização de cache
- Imagem final pesada (> 500MB) — multi-stage e Alpine/distroless

## Padrões para Dockerfile

### Multi-stage builds (obrigatório)

Separe build (compiladores, deps de dev) de runtime (binário/artefato + libs mínimas).

```dockerfile
# Stage 1 — build
FROM rust:1.80-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY src ./src
RUN cargo build --release

# Stage 2 — runtime
FROM alpine:3.20
RUN adduser -D -u 1000 appuser
COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp
USER appuser
EXPOSE 8080
CMD ["myapp"]
```

### Imagens base

- Use Alpine ou distroless quando possível
- Tags **explícitas** (`php:8.3-fpm-alpine`, nunca `latest`)
- Pin de versão evita surpresas de patch

### Usuário não-root

```dockerfile
RUN adduser -D -u 1000 appuser
USER appuser
```

Aplicação rodando como root no container = vulnerabilidade.

### Cache de camadas

Ordene do menos mutável para o mais mutável:
1. `FROM`
2. `RUN apt-get install …` (raro mudar)
3. `COPY package.json` / `Cargo.toml` / `composer.json`
4. `RUN npm install` / `cargo fetch` / `composer install`
5. `COPY src/` ← muda muito, fica no final
6. `RUN build`

### Limpeza na mesma camada `RUN`

```dockerfile
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
```

### `ENTRYPOINT` vs `CMD`

- `ENTRYPOINT` — comando fixo do container
- `CMD` — argumentos default, podem ser sobrescritos

## Padrões por stack

### PHP / Laravel

```dockerfile
FROM php:8.3-fpm-alpine AS base
WORKDIR /var/www/html

RUN apk add --no-cache --virtual .build-deps \
        $PHPIZE_DEPS postgresql-dev && \
    docker-php-ext-install pdo_pgsql opcache bcmath && \
    apk del .build-deps

COPY composer.json composer.lock ./
RUN composer install --no-dev --no-interaction --no-progress --optimize-autoloader

COPY . .
RUN chown -R www-data:www-data storage bootstrap/cache

USER www-data
EXPOSE 9000
CMD ["php-fpm"]
```

### Python / FastAPI

```dockerfile
FROM python:3.11-slim AS builder
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

FROM python:3.11-slim
RUN useradd -m -u 1000 appuser
COPY --from=builder /root/.local /home/appuser/.local
ENV PATH=/home/appuser/.local/bin:$PATH
WORKDIR /app
COPY --chown=appuser:appuser . .
USER appuser
EXPOSE 8000
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Go

```dockerfile
FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /app/bin/server ./cmd/server

FROM scratch
COPY --from=builder /app/bin/server /server
EXPOSE 8080
USER 1000:1000
ENTRYPOINT ["/server"]
```

### Node / Vue (build estático)

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

### Rails

```dockerfile
FROM ruby:3.3-alpine AS builder
RUN apk add --no-cache build-base postgresql-dev nodejs yarn
WORKDIR /app
COPY Gemfile Gemfile.lock ./
RUN bundle install --without development test
COPY . .
RUN SECRET_KEY_BASE=dummy bundle exec rails assets:precompile

FROM ruby:3.3-alpine
RUN apk add --no-cache postgresql-client tzdata
WORKDIR /app
COPY --from=builder /usr/local/bundle /usr/local/bundle
COPY --from=builder /app /app
RUN adduser -D -u 1000 appuser && chown -R appuser:appuser /app
USER appuser
EXPOSE 3000
CMD ["bundle", "exec", "rails", "server", "-b", "0.0.0.0"]
```

## Padrões para docker-compose

### Especificação atual

NÃO use `version: '3'` — está deprecated. Estrutura moderna:

```yaml
services:
  api:
    build: .
    ports: ["8080:8080"]
    env_file: .env
    depends_on:
      postgres:
        condition: service_healthy
    networks: [internal]

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      retries: 5
    networks: [internal]

volumes:
  postgres_data:

networks:
  internal:
```

### Boas práticas

- **Serviços isolados** — Nginx + PHP-FPM em containers separados (não monolito)
- **Volumes nomeados** — persistência clara
- **Redes customizadas** — isolar comunicação entre serviços
- **Variáveis via `.env`** — nunca hardcode senha em compose
- **Healthchecks** — `depends_on: condition: service_healthy` evita race condition
- **Portas de DB não expostas para host em prod** — só na rede interna

## `.dockerignore` obrigatório

```
.git/
.github/
.vscode/
.idea/
node_modules/
vendor/
target/
.next/
dist/
build/
.env
.env.local
*.log
tests/
DARE/
.dockerignore
Dockerfile*
docker-compose*.yml
README.md
docs/
```

## Métricas / checklist

| Critério | OK |
|---|---|
| Multi-stage build | [ ] |
| Usuário não-root no runtime | [ ] |
| Tag explícita (não `latest`) | [ ] |
| `.dockerignore` presente | [ ] |
| Healthcheck nos serviços DB/cache | [ ] |
| Volumes nomeados (não bind mount em prod) | [ ] |
| Sem senha hardcoded no compose | [ ] |
| Imagem final < 200MB (linguagem compilada) ou < 500MB (interpretada) | [ ] |
| Cache de camadas otimizado | [ ] |

## Antipatterns

| AP | Antipattern | Por que evitar |
|---|---|---|
| AP-01 | Rodar como root | Vulnerabilidade — escape de container vira root no host |
| AP-02 | Tag `latest` | Build não reproduzível |
| AP-03 | `COPY . .` antes de instalar deps | Cache invalidado a cada commit |
| AP-04 | Senha em compose | Vaza em log e git |
| AP-05 | Mono-container com tudo (DB + app) | Não escala, viola SRP |
| AP-06 | `version: '3'` em compose | Deprecated |
| AP-07 | Sem healthcheck no DB | `depends_on` simples não espera DB ficar pronto |
| AP-08 | Sem `.dockerignore` | Imagem pesada, vaza `.env`/`.git` |

## Como aplicar

### Passo 1: Analisar stack

Leia `DESIGN.md` ou arquivos de config para identificar linguagem + framework + versão.

### Passo 2: Gerar Dockerfile

Use o template correspondente à stack acima. Adapte:
- Versão exata (PHP 8.3, Node 20, Python 3.11, …)
- Extensões/deps específicas
- Porta exposta
- Comando de start

### Passo 3: Gerar docker-compose.yml

Liste serviços necessários (api, db, redis, queue worker, …). Use healthchecks.

### Passo 4: Gerar .dockerignore

Pasta `DARE/`, `node_modules/`, `.git/`, `.env`, `tests/` — sempre.

### Passo 5: Validar

```bash
docker build -t myapp .
docker compose config         # valida sintaxe
docker compose up --build
docker compose ps             # verificar healthchecks
```

## Dicas

- **Use** `docker scout cves` para escanear imagens
- **Combine** com `dare-security` (escanear secrets antes de COPY)
- **Para Rust**, `cargo-chef` acelera builds CI dramaticamente
- **Para multi-arch**, use `docker buildx build --platform linux/amd64,linux/arm64`

---

Esta skill é parte do DARE Method e está sob licença MIT.
