// Package middleware — Gin middlewares (JWT, rate limit, CORS).
package middleware

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"

	"{{moduleName}}/internal/config"
)

func JWT(cfg *config.Config) gin.HandlerFunc {
	return func(c *gin.Context) {
		raw := c.GetHeader("Authorization")
		token := strings.TrimPrefix(raw, "Bearer ")
		if token == raw {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing bearer"})
			return
		}
		parsed, err := jwt.Parse(token, func(t *jwt.Token) (any, error) {
			if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
				return nil, jwt.ErrSignatureInvalid
			}
			return []byte(cfg.JWTSecret), nil
		})
		if err != nil || !parsed.Valid {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "invalid token"})
			return
		}
		claims, _ := parsed.Claims.(jwt.MapClaims)
		c.Set("claims", map[string]any(claims))
		c.Next()
	}
}

// CORS — comma-separated whitelist of origins from env.
func CORS(allow []string) gin.HandlerFunc {
	set := make(map[string]bool, len(allow))
	for _, o := range allow {
		set[o] = true
	}
	return func(c *gin.Context) {
		origin := c.Request.Header.Get("Origin")
		if origin != "" && set[origin] {
			c.Header("Access-Control-Allow-Origin", origin)
			c.Header("Vary", "Origin")
			c.Header("Access-Control-Allow-Headers", "Authorization, Content-Type")
			c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		}
		if c.Request.Method == http.MethodOptions {
			c.AbortWithStatus(http.StatusNoContent)
			return
		}
		c.Next()
	}
}
