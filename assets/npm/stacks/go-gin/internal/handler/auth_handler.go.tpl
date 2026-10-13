// Package handler — HTTP layer (Gin handlers).
package handler

import (
	"errors"
	"net/http"

	"github.com/gin-gonic/gin"

	"{{moduleName}}/internal/config"
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
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=8"`
}

// Login godoc
// @Summary Exchange email+password for JWT
// @Accept  json
// @Produce json
// @Param   body body loginBody true "credentials"
// @Success 200 {object} map[string]any
// @Failure 401 {object} map[string]string
// @Router  /auth/login [post]
func (h *AuthHandler) Login(c *gin.Context) {
	var body loginBody
	if err := c.ShouldBindJSON(&body); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	token, expiresIn, err := h.auth.Login(c.Request.Context(), body.Email, body.Password)
	if errors.Is(err, service.ErrInvalidCredentials) {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid credentials"})
		return
	}
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "internal error"})
		return
	}
	c.JSON(http.StatusOK, gin.H{
		"accessToken": token,
		"tokenType":   "Bearer",
		"expiresIn":   expiresIn,
	})
}

// Me godoc
// @Summary Current user info
// @Security BearerAuth
// @Produce json
// @Success 200 {object} map[string]any
// @Router  /auth/me [get]
func (h *AuthHandler) Me(c *gin.Context) {
	claims, ok := c.Get("claims")
	if !ok {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "unauthorized"})
		return
	}
	c.JSON(http.StatusOK, claims)
}
