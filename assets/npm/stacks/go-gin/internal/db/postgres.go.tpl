// Package db — pgx connection pool.
package db

import (
	"context"

	"github.com/jackc/pgx/v5/pgxpool"
)

// Connect returns a pgx pool. Caller closes via pool.Close().
func Connect(ctx context.Context, url string) (*pgxpool.Pool, error) {
	cfg, err := pgxpool.ParseConfig(url)
	if err != nil {
		return nil, err
	}
	pool, err := pgxpool.NewWithConfig(ctx, cfg)
	if err != nil {
		return nil, err
	}
	if err := pool.Ping(ctx); err != nil {
		pool.Close()
		return nil, err
	}
	return pool, nil
}
