# {{projectName}}

DARE-shaped Go API on Gin. Layered Design (Handler → Service → Repository → Model).

## Setup

```bash
cp .env.example .env
docker compose up -d postgres
go mod tidy
go install github.com/sqlc-dev/sqlc/cmd/sqlc@latest
go install github.com/swaggo/swag/cmd/swag@latest
sqlc generate
swag init -g cmd/server/main.go -o docs
go run ./cmd/server
```

## Endpoints

- `POST /auth/login` — exchange email/password for JWT
- `GET /auth/me` — current user (Bearer JWT)
- `GET /users` — list users (paginated)
- `POST /users` — create user (admin only)
- `GET /ws` — WebSocket echo
- `GET /openapi.json` — OpenAPI surface
- `GET /swagger/index.html` — Swagger UI (after `swag init`)

## Commands

- `go run ./cmd/server` — dev server (port 8000)
- `go test ./...` — tests
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan
- `sqlc generate` — regenerate type-safe DB code

## DARE Method

See `.dare/skills.yml` for installed skills, `llms.txt` for agent context.
