-- name: FindUserByEmail :one
SELECT id, email, password, role, created_at, updated_at
FROM users
WHERE email = $1;

-- name: GetUserByID :one
SELECT id, email, password, role, created_at, updated_at
FROM users
WHERE id = $1;

-- name: ListUsers :many
SELECT id, email, password, role, created_at, updated_at
FROM users
ORDER BY created_at DESC
LIMIT $1 OFFSET $2;

-- name: CountUsers :one
SELECT COUNT(*) FROM users;

-- name: CreateUser :one
INSERT INTO users (id, email, password, role)
VALUES ($1, $2, $3, $4)
RETURNING id, email, password, role, created_at, updated_at;
