package middleware

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"

	"github.com/golang-jwt/jwt/v5"

	"{{moduleName}}/internal/config"
)

// JWT returns a middleware that validates a Bearer token and stashes the
// claims as a map[string]any in request context under ClaimsContextKey.
func JWT(cfg *config.Config) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			raw := r.Header.Get("Authorization")
			token := strings.TrimPrefix(raw, "Bearer ")
			if token == raw || token == "" {
				writeAuthError(w, http.StatusUnauthorized, "missing bearer")
				return
			}
			parsed, err := jwt.Parse(token, func(t *jwt.Token) (any, error) {
				if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
					return nil, jwt.ErrSignatureInvalid
				}
				return []byte(cfg.JWTSecret), nil
			})
			if err != nil || !parsed.Valid {
				writeAuthError(w, http.StatusUnauthorized, "invalid token")
				return
			}
			claims, _ := parsed.Claims.(jwt.MapClaims)
			ctx := context.WithValue(r.Context(), ClaimsContextKey, map[string]any(claims))
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func writeAuthError(w http.ResponseWriter, status int, detail string) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]any{
		"type":   "urn:problem:Unauthorized",
		"title":  http.StatusText(status),
		"status": status,
		"detail": detail,
	})
}
