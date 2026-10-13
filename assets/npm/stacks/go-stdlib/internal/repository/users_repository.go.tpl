// Package repository — the only layer with SQL strings. Production setups can
// swap in sqlc-generated code from db/queries/users.sql.
package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"{{moduleName}}/internal/model"
)

type UsersRepository struct {
	pool *pgxpool.Pool
}

func NewUsersRepository(pool *pgxpool.Pool) *UsersRepository {
	return &UsersRepository{pool: pool}
}

func (r *UsersRepository) FindByEmail(ctx context.Context, email string) (*model.User, error) {
	row := r.pool.QueryRow(ctx,
		`SELECT id, email, password, role, created_at, updated_at FROM users WHERE email = $1`, email)
	u := &model.User{}
	if err := row.Scan(&u.ID, &u.Email, &u.Password, &u.Role, &u.CreatedAt, &u.UpdatedAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	return u, nil
}

func (r *UsersRepository) Page(ctx context.Context, offset, limit int64) ([]*model.User, error) {
	rows, err := r.pool.Query(ctx,
		`SELECT id, email, password, role, created_at, updated_at
		 FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2`, limit, offset)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var out []*model.User
	for rows.Next() {
		u := &model.User{}
		if err := rows.Scan(&u.ID, &u.Email, &u.Password, &u.Role, &u.CreatedAt, &u.UpdatedAt); err != nil {
			return nil, err
		}
		out = append(out, u)
	}
	return out, rows.Err()
}

func (r *UsersRepository) Count(ctx context.Context) (int64, error) {
	var total int64
	err := r.pool.QueryRow(ctx, `SELECT COUNT(*) FROM users`).Scan(&total)
	return total, err
}

func (r *UsersRepository) Create(ctx context.Context, email, passwordHash, role string) (*model.User, error) {
	u := &model.User{}
	err := r.pool.QueryRow(ctx,
		`INSERT INTO users (id, email, password, role)
		 VALUES ($1, $2, $3, $4)
		 RETURNING id, email, password, role, created_at, updated_at`,
		uuid.New(), email, passwordHash, role,
	).Scan(&u.ID, &u.Email, &u.Password, &u.Role, &u.CreatedAt, &u.UpdatedAt)
	if err != nil {
		return nil, err
	}
	return u, nil
}
