// Package handler — net/http handlers.
package handler

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"{{moduleName}}/internal/config"
	"{{moduleName}}/internal/httpx"
	"{{moduleName}}/internal/middleware"
	"{{moduleName}}/internal/service"
)

type AuthHandler struct {
	auth  *service.AuthService
	users *service.UsersService
	cfg   *config.Config
}

func NewAuthHandler(a *service.AuthService, u *service.UsersService, cfg *config.Config) *AuthHandler {
	return &AuthHandler{auth: a, users: u, cfg: cfg}
}

type loginBody struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

func (h *AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	var body loginBody
	if err := httpx.DecodeJSON(r, &body); err != nil {
		httpx.WriteError(w, http.StatusBadRequest, err.Error())
		return
	}
	if !strings.Contains(body.Email, "@") || len(body.Password) < 8 {
		httpx.WriteError(w, http.StatusBadRequest, "invalid email or password")
		return
	}
	token, expiresIn, err := h.auth.Login(r.Context(), body.Email, body.Password)
	if errors.Is(err, service.ErrInvalidCredentials) {
		httpx.WriteError(w, http.StatusUnauthorized, "invalid credentials")
		return
	}
	if err != nil {
		httpx.WriteError(w, http.StatusInternalServerError, "internal error")
		return
	}
	httpx.WriteJSON(w, http.StatusOK, map[string]any{
		"accessToken": token,
		"tokenType":   "Bearer",
		"expiresIn":   expiresIn,
	})
}

func (h *AuthHandler) Me(w http.ResponseWriter, r *http.Request) {
	claims, ok := claimsFromContext(r.Context())
	if !ok {
		httpx.WriteError(w, http.StatusUnauthorized, "unauthorized")
		return
	}
	httpx.WriteJSON(w, http.StatusOK, claims)
}

// claimsFromContext extracts the JWT claims placed by middleware.JWT.
func claimsFromContext(ctx context.Context) (map[string]any, bool) {
	v := ctx.Value(middleware.ClaimsContextKey)
	c, ok := v.(map[string]any)
	return c, ok
}
