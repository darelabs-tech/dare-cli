// Package config — runtime configuration from environment via caarlos0/env.
package config

import (
	"strings"

	"github.com/caarlos0/env/v11"
)

type Config struct {
	AppPort          int      `env:"APP_PORT" envDefault:"8000"`
	DatabaseURL      string   `env:"DATABASE_URL,required"`
	JWTSecret        string   `env:"JWT_SECRET" envDefault:"replace-me-in-prod"`
	JWTExpHours      int      `env:"JWT_EXP_HOURS" envDefault:"1"`
	BcryptCost       int      `env:"BCRYPT_COST" envDefault:"12"`
	RateLimitPerSec  int      `env:"RATE_LIMIT_PER_SEC" envDefault:"10"`
	RateLimitBurst   int      `env:"RATE_LIMIT_BURST" envDefault:"20"`
	CORSOriginsRaw   string   `env:"CORS_ORIGINS" envDefault:"http://localhost:3000"`
	LLMProvider      string   `env:"LLM_PROVIDER" envDefault:"dummy"`
	CORSOrigins      []string `env:"-"`
}

func FromEnv() (*Config, error) {
	cfg := &Config{}
	if err := env.Parse(cfg); err != nil {
		return nil, err
	}
	cfg.CORSOrigins = splitAndTrim(cfg.CORSOriginsRaw)
	return cfg, nil
}

func splitAndTrim(s string) []string {
	parts := strings.Split(s, ",")
	out := make([]string, 0, len(parts))
	for _, p := range parts {
		if t := strings.TrimSpace(p); t != "" {
			out = append(out, t)
		}
	}
	return out
}
