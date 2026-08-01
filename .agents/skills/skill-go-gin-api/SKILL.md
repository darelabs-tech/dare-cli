---
name: skill-go-gin-api
description: Padrões DARE para APIs REST em Go + Gin (ou stdlib net/http) + sqlc + PostgreSQL. Handlers, services, repositories, middleware, validação com go-playground/validator, JWT, rate limit, swag (OpenAPI), testes com testify e httptest.
---

# DARE Go/Gin API Skill

Você é um desenvolvedor sênior Go especialista em APIs REST com Gin (ou stdlib `net/http`). Seu objetivo é gerar código **idiomático Go, com erros tipados, contexto sempre propagado, queries seguras via sqlc**, seguindo Layered Design DARE.

## Quando usar

- Projeto Go novo via DARE
- Adicionar feature em API Go existente
- Migrar de Echo/Fiber para Gin/stdlib
- Auditar projeto Go para conformidade DARE

## Stack canônica

- **Go 1.22+** (com `slog`, `errors.Is/As`, generics)
- **Gin** (preferido) ou **stdlib net/http** (Go 1.22+ tem roteamento OK)
- **sqlc** para queries seguras (gera código tipado a partir de SQL)
- **pgx v5** como driver Postgres
- **go-playground/validator** para DTOs
- **golang-jwt/jwt v5**
- **swaggo/swag** para OpenAPI auto-gerado
- **testify** para asserts + **httptest** para handlers
- **golangci-lint** com config estrita
- **govulncheck** para CVEs

## Layered Design em Go

```
.
├── cmd/server/main.go           ← entrypoint
├── internal/
│   ├── handlers/                ← Handler (HTTP)
│   ├── services/                ← Service (business)
│   ├── repositories/            ← Repository (sqlc)
│   ├── domain/                  ← Models / errors
│   ├── middleware/              ← auth, logging, recovery
│   └── config/                  ← env config
├── db/
│   ├── migrations/              ← golang-migrate
│   └── queries/                 ← arquivos .sql para sqlc
├── sqlc.yaml
└── docs/                        ← gerado pelo swag
```

## Handlers

```go
package handlers

import (
    "errors"
    "net/http"
    "github.com/gin-gonic/gin"
    "myapp/internal/domain"
    "myapp/internal/services"
)

type UserHandler struct {
    register *services.RegisterUser
}

func NewUserHandler(r *services.RegisterUser) *UserHandler {
    return &UserHandler{register: r}
}

// @Summary Create user
// @Tags users
// @Accept json
// @Produce json
// @Param user body CreateUserDTO true "user payload"
// @Success 201 {object} UserResponse
// @Router /users [post]
func (h *UserHandler) Create(c *gin.Context) {
    var dto CreateUserDTO
    if err := c.ShouldBindJSON(&dto); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        return
    }
    user, err := h.register.Execute(c.Request.Context(), dto.ToService())
    if err != nil {
        if errors.Is(err, domain.ErrUserAlreadyExists) {
            c.JSON(http.StatusConflict, gin.H{"error": "USER_EXISTS"})
            return
        }
        c.JSON(http.StatusInternalServerError, gin.H{"error": "INTERNAL"})
        return
    }
    c.JSON(http.StatusCreated, ToUserResponse(user))
}
```

## DTOs (validator)

```go
package handlers

import "myapp/internal/services"

type CreateUserDTO struct {
    Email    string `json:"email" binding:"required,email"`
    Name     string `json:"name" binding:"required,min=1,max=255"`
    Password string `json:"password" binding:"required,min=12"`
}

func (d CreateUserDTO) ToService() services.RegisterUserInput {
    return services.RegisterUserInput{Email: d.Email, Name: d.Name, Password: d.Password}
}

type UserResponse struct {
    ID        int64     `json:"id"`
    Email     string    `json:"email"`
    Name      string    `json:"name"`
    CreatedAt time.Time `json:"createdAt"`
}
```

## Services

```go
package services

import (
    "context"
    "myapp/internal/domain"
    "myapp/internal/repositories"
    "golang.org/x/crypto/argon2"
)

type RegisterUser struct {
    repo *repositories.UserRepository
}

func NewRegisterUser(repo *repositories.UserRepository) *RegisterUser {
    return &RegisterUser{repo: repo}
}

type RegisterUserInput struct {
    Email, Name, Password string
}

func (s *RegisterUser) Execute(ctx context.Context, in RegisterUserInput) (*domain.User, error) {
    exists, err := s.repo.ExistsByEmail(ctx, in.Email)
    if err != nil { return nil, err }
    if exists { return nil, domain.ErrUserAlreadyExists }
    return s.repo.Create(ctx, repositories.CreateUserParams{
        Email: in.Email,
        Name:  in.Name,
        PasswordHash: hashPassword(in.Password),
    })
}
```

## Repositories (sqlc)

```sql
-- db/queries/users.sql
-- name: ExistsUserByEmail :one
SELECT EXISTS(SELECT 1 FROM users WHERE email = $1);

-- name: CreateUser :one
INSERT INTO users (email, name, password_hash)
VALUES ($1, $2, $3)
RETURNING id, email, name, created_at;
```

```go
// internal/repositories/users.go
package repositories

import "myapp/internal/repositories/sqlc"  // gerado pelo sqlc

type UserRepository struct {
    q *sqlc.Queries
}

func NewUserRepository(q *sqlc.Queries) *UserRepository { return &UserRepository{q: q} }

func (r *UserRepository) ExistsByEmail(ctx context.Context, email string) (bool, error) {
    return r.q.ExistsUserByEmail(ctx, email)
}
```

## Domain (Models + erros tipados)

```go
package domain

import "errors"

var (
    ErrUserAlreadyExists = errors.New("user already exists")
    ErrUserNotFound      = errors.New("user not found")
)

type User struct {
    ID        int64
    Email     string
    Name      string
    CreatedAt time.Time
}
```

## Middleware

```go
package middleware

import (
    "github.com/gin-gonic/gin"
    "github.com/golang-jwt/jwt/v5"
)

func AuthRequired(secret []byte) gin.HandlerFunc {
    return func(c *gin.Context) {
        tokenStr := c.GetHeader("Authorization")
        // strip "Bearer "
        token, err := jwt.Parse(tokenStr, func(t *jwt.Token) (any, error) {
            return secret, nil
        })
        if err != nil || !token.Valid {
            c.AbortWithStatusJSON(401, gin.H{"error": "Unauthorized"})
            return
        }
        c.Set("user_id", token.Claims.(jwt.MapClaims)["sub"])
        c.Next()
    }
}
```

## Rate limiting

```go
import "github.com/gin-contrib/limit"

router := gin.Default()
router.Use(limit.MaxAllowed(20))   // global

login := router.Group("/auth").Use(myRateLimiter("5/15m"))
login.POST("/login", authHandler.Login)
```

## OpenAPI (swag)

```bash
swag init -g cmd/server/main.go -o docs
```

```go
// main.go
// @title Projeto API
// @version 1.0
import _ "myapp/docs"
import "github.com/swaggo/gin-swagger"

router.GET("/swagger/*any", ginSwagger.WrapHandler(swaggerFiles.Handler))
router.GET("/openapi.json", func(c *gin.Context) {
    c.File("docs/swagger.json")
})
```

## Config (env)

```go
package config

import "os"

type Config struct {
    DatabaseURL string
    JWTSecret   []byte
}

func Load() (*Config, error) {
    return &Config{
        DatabaseURL: os.Getenv("DATABASE_URL"),
        JWTSecret:   []byte(os.Getenv("JWT_SECRET")),
    }, nil
}
```

## Testes

```go
func TestCreateUser(t *testing.T) {
    repo := &mocks.UserRepository{}
    repo.On("ExistsByEmail", mock.Anything, "jane@example.com").Return(false, nil)
    repo.On("Create", mock.Anything, mock.Anything).Return(&domain.User{ID: 1, Email: "jane@example.com"}, nil)

    svc := services.NewRegisterUser(repo)
    user, err := svc.Execute(ctx, services.RegisterUserInput{Email: "jane@example.com", ...})
    require.NoError(t, err)
    assert.Equal(t, int64(1), user.ID)
}

// Handler test via httptest
func TestUserHandlerCreate(t *testing.T) {
    r := gin.New()
    r.POST("/users", h.Create)
    w := httptest.NewRecorder()
    body := `{"email":"jane@example.com","name":"Jane","password":"longsecret123"}`
    req := httptest.NewRequest("POST", "/users", strings.NewReader(body))
    r.ServeHTTP(w, req)
    assert.Equal(t, 201, w.Code)
}
```

## Antipatterns

| AP | Antipattern | Correção |
|---|---|---|
| AP-01 | `database/sql` direto no handler | Repository + sqlc |
| AP-02 | Validação manual | `binding:` tags + validator |
| AP-03 | Lógica no handler | Service |
| AP-04 | Retornar `db.User` direto | `UserResponse` DTO |
| AP-05 | Secret hardcoded | `os.Getenv` ou viper |
| AP-06 | Sem rate limit em login | middleware |
| AP-07 | `panic` em handler | `recovery` middleware + erro tratado |
| AP-08 | Context não propagado | sempre passe `ctx context.Context` |
| AP-09 | SQL com `fmt.Sprintf` | sqlc / parametrizado |
| AP-10 | `error.New` por toda parte | tipos sentinela em `domain/errors.go` |

## Segurança

- Senhas: `argon2.IDKey` (golang.org/x/crypto/argon2)
- JWT: HS256 com secret ≥ 32 bytes, ou RS256
- Headers: middleware com `secure` headers
- CORS: `github.com/gin-contrib/cors` com origens específicas
- Rate limit em login (5/15min)

## CI

```bash
go vet ./...
golangci-lint run
go test ./... -race -cover
govulncheck ./...
```

## Como aplicar

### Passo 1: Audit

```bash
grep -rn "database/sql" internal/handlers/        # AP-01
grep -rn "json.Unmarshal" internal/handlers/      # AP-02
grep -rn "fmt.Sprintf.*SELECT\|INSERT" internal/  # AP-09
```

### Passo 2: Migrar para DTOs com `binding:`

Para cada `json.Unmarshal` → struct com `binding:"required,..."`.

### Passo 3: Extrair Services

Lógica > 5 linhas no handler → Service.

### Passo 4: Adicionar sqlc

Crie `db/queries/*.sql`, configure `sqlc.yaml`, rode `sqlc generate`. Repositories importam `sqlc.Queries`.

### Passo 5: Middleware + rate limit + OpenAPI

Adicionar middleware de auth, rate limit em login, gerar OpenAPI com swag.

## Dicas

- **Combine** com `dare-docker` (scratch ou alpine multi-stage)
- **Use** `slog` para logs estruturados (Go 1.22+ stdlib)
- **Para realtime**, `gorilla/websocket` ou `melody`

---

Esta skill é parte do DARE Method e está sob licença MIT.
