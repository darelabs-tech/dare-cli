package handler

import (
	"errors"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"

	"{{moduleName}}/internal/service"
)

type UsersHandler struct {
	svc *service.UsersService
}

func NewUsersHandler(s *service.UsersService) *UsersHandler {
	return &UsersHandler{svc: s}
}

func (h *UsersHandler) List(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 100 {
		limit = 20
	}
	items, total, err := h.svc.Page(c.Request.Context(), page, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "internal"})
		return
	}
	c.JSON(http.StatusOK, gin.H{"items": items, "total": total, "page": page})
}

type createUserBody struct {
	Email    string  `json:"email" binding:"required,email"`
	Password string  `json:"password" binding:"required,min=8"`
	Role     *string `json:"role" binding:"omitempty,oneof=USER ADMIN"`
}

func (h *UsersHandler) Create(c *gin.Context) {
	claimsAny, _ := c.Get("claims")
	claims, _ := claimsAny.(map[string]any)
	if role, _ := claims["role"].(string); role != "ADMIN" {
		c.JSON(http.StatusForbidden, gin.H{"error": "admin role required"})
		return
	}

	var body createUserBody
	if err := c.ShouldBindJSON(&body); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	role := "USER"
	if body.Role != nil {
		role = *body.Role
	}

	user, err := h.svc.Create(c.Request.Context(), body.Email, body.Password, role)
	if errors.Is(err, service.ErrEmailExists) {
		c.JSON(http.StatusConflict, gin.H{"error": "email already in use"})
		return
	}
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "internal"})
		return
	}
	c.JSON(http.StatusCreated, user)
}
