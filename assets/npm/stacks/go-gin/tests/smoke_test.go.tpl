// Smoke tests — exercise pure helpers without a DB.
package tests

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"{{moduleName}}/internal/config"
)

func TestConfigFromEnvDefaultsAreSane(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://u:p@localhost:5432/x?sslmode=disable")
	t.Setenv("APP_PORT", "")
	os.Unsetenv("APP_PORT")
	cfg, err := config.FromEnv()
	require.NoError(t, err)
	require.Equal(t, 8000, cfg.AppPort)
	require.GreaterOrEqual(t, cfg.RateLimitBurst, cfg.RateLimitPerSec)
	require.NotEmpty(t, cfg.CORSOrigins)
}
