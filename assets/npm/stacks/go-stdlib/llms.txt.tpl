# {{projectName}}

DARE-shaped Go API on **net/http stdlib** (no framework). Layered Design.

## Setup

```bash
cp .env.example .env
docker compose up -d postgres
go mod tidy
sqlc generate
go run ./cmd/server
```

## Commands

- `go run ./cmd/server` — dev server (port 8000)
- `go test ./...` — tests
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan
- `sqlc generate` — regenerate type-safe DB code from db/queries/*.sql

## Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/auth/login` | none | Exchange email+password for JWT |
| GET | `/auth/me` | Bearer | Current user |
| GET | `/users` | Bearer | List (query: page, limit) |
| POST | `/users` | Bearer (admin) | Create user |
| GET | `/ws` | upgrade | WebSocket echo |
| GET | `/openapi.json` | none | OpenAPI document |

## Layered Design

- `internal/handler/` — HTTP layer (net/http handlers)
- `internal/service/` — business logic
- `internal/repository/` — pgx queries (sqlc-compatible)
- `internal/model/` — domain structs
- `internal/middleware/` — JWT + rate limit + CORS + chain helper
- `internal/llm/` — provider abstraction
- `internal/httpx/` — JSON helpers (encode/decode)

Handlers never call pgx directly.

## Why net/http stdlib?

Go 1.22 added method+pattern routing to `http.ServeMux`:
`mux.HandleFunc("POST /auth/login", h.Login)`. That covers 80% of what
Gin / Echo / Chi were used for, with zero deps. Frameworks only buy
nicer middleware DX and a slightly faster path tree — both replaceable
with the helpers in `internal/middleware/`.

## Env vars

See `.env.example`. Required: `DATABASE_URL`, `JWT_SECRET`.

## DARE skills

See `.dare/skills.yml`.
