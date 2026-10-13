// Smoke tests — exercise pure helpers without a DB.
package tests

import (
	"net/http"
	"net/http/httptest"
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"{{moduleName}}/internal/config"
	"{{moduleName}}/internal/middleware"
)

func TestConfigFromEnvDefaultsAreSane(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://u:p@localhost:5432/x?sslmode=disable")
	os.Unsetenv("APP_PORT")
	cfg, err := config.FromEnv()
	require.NoError(t, err)
	require.Equal(t, 8000, cfg.AppPort)
	require.GreaterOrEqual(t, cfg.RateLimitBurst, cfg.RateLimitPerSec)
}

func TestRateLimitBlocksAfterBurst(t *testing.T) {
	mw := middleware.RateLimit(1, 2)
	h := mw(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	// First 2 (burst) should pass.
	for i := 0; i < 2; i++ {
		rec := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodGet, "/", nil)
		req.RemoteAddr = "10.0.0.1:1234"
		h.ServeHTTP(rec, req)
		require.Equal(t, http.StatusOK, rec.Code)
	}
	// 3rd should be 429.
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	req.RemoteAddr = "10.0.0.1:1234"
	h.ServeHTTP(rec, req)
	require.Equal(t, http.StatusTooManyRequests, rec.Code)
}
