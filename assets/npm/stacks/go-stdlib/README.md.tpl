# {{projectName}}

DARE-shaped Go API on **net/http** (no framework). Layered Design (Handler → Service → Repository → Model).

## Setup

```bash
cp .env.example .env
docker compose up -d postgres
go mod tidy
go install github.com/sqlc-dev/sqlc/cmd/sqlc@latest
sqlc generate
go run ./cmd/server
```

## Endpoints

- `POST /auth/login` — exchange email/password for JWT
- `GET /auth/me` — current user (Bearer JWT)
- `GET /users` — list users (paginated)
- `POST /users` — create user (admin only)
- `GET /ws` — WebSocket echo
- `GET /openapi.json` — OpenAPI surface (static)

## Commands

- `go run ./cmd/server` — dev server (port 8000)
- `go test ./...` — tests
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan
- `sqlc generate` — regenerate type-safe DB code

## Why no framework?

Go 1.22's `http.ServeMux` supports method + path patterns (e.g. `POST /auth/login`),
which removes the historical reason for using Gin / Echo / Chi. This template uses
stdlib for routing, middleware chaining, and JSON serialization.

## DARE Method

See `.dare/skills.yml`.
