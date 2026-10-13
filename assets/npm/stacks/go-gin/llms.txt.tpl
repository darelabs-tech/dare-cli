# {{projectName}}

DARE-shaped Go API on Gin 1.10+. Layered Design: Handler → Service → Repository → Model.

## Setup

```bash
cp .env.example .env
docker compose up -d postgres
go mod tidy
sqlc generate          # type-safe DB code from db/queries/*.sql
swag init -g cmd/server/main.go -o docs   # OpenAPI from annotations
go run ./cmd/server
```

## Commands

- `go run ./cmd/server` — dev server (port 8000)
- `go test ./...` — full test suite
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan
- `sqlc generate` — regenerate repos from SQL
- `swag init -g cmd/server/main.go -o docs` — refresh OpenAPI

## Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/auth/login` | none | Exchange email+password for JWT |
| GET | `/auth/me` | Bearer | Current user |
| GET | `/users` | Bearer | List (query: page, limit) |
| POST | `/users` | Bearer (admin) | Create user |
| GET | `/ws` | upgrade | WebSocket echo |
| GET | `/openapi.json` | none | OpenAPI document |
| GET | `/swagger/index.html` | none | Swagger UI |

## Layered Design

- `internal/handler/` — Gin handlers (HTTP)
- `internal/service/` — business logic
- `internal/repository/` — sqlx/pgx queries via sqlc-generated code
- `internal/model/` — domain structs
- `internal/middleware/` — JWT + rate limit
- `internal/llm/` — provider abstraction

Handlers never call pgx directly.

## Env vars

See `.env.example`. Required: `DATABASE_URL`, `JWT_SECRET`.

## DARE skills

See `.dare/skills.yml`.
