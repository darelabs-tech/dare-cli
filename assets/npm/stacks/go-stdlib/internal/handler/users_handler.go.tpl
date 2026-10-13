package handler

import (
	"errors"
	"net/http"
	"strconv"

	"{{moduleName}}/internal/httpx"
	"{{moduleName}}/internal/service"
)

type UsersHandler struct {
	svc *service.UsersService
}

func NewUsersHandler(s *service.UsersService) *UsersHandler {
	return &UsersHandler{svc: s}
}

func (h *UsersHandler) List(w http.ResponseWriter, r *http.Request) {
	page, _ := strconv.Atoi(r.URL.Query().Get("page"))
	limit, _ := strconv.Atoi(r.URL.Query().Get("limit"))
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 100 {
		limit = 20
	}
	items, total, err := h.svc.Page(r.Context(), page, limit)
	if err != nil {
		httpx.WriteError(w, http.StatusInternalServerError, "internal")
		return
	}
	httpx.WriteJSON(w, http.StatusOK, map[string]any{
		"items": items, "total": total, "page": page,
	})
}

type createUserBody struct {
	Email    string  `json:"email"`
	Password string  `json:"password"`
	Role     *string `json:"role,omitempty"`
}

func (h *UsersHandler) Create(w http.ResponseWriter, r *http.Request) {
	claims, ok := claimsFromContext(r.Context())
	if !ok {
		httpx.WriteError(w, http.StatusUnauthorized, "unauthorized")
		return
	}
	if role, _ := claims["role"].(string); role != "ADMIN" {
		httpx.WriteError(w, http.StatusForbidden, "admin role required")
		return
	}

	var body createUserBody
	if err := httpx.DecodeJSON(r, &body); err != nil {
		httpx.WriteError(w, http.StatusBadRequest, err.Error())
		return
	}
	if len(body.Password) < 8 {
		httpx.WriteError(w, http.StatusBadRequest, "password must be at least 8 chars")
		return
	}
	role := "USER"
	if body.Role != nil {
		if *body.Role != "USER" && *body.Role != "ADMIN" {
			httpx.WriteError(w, http.StatusBadRequest, "role must be USER or ADMIN")
			return
		}
		role = *body.Role
	}

	user, err := h.svc.Create(r.Context(), body.Email, body.Password, role)
	if errors.Is(err, service.ErrEmailExists) {
		httpx.WriteError(w, http.StatusConflict, "email already in use")
		return
	}
	if err != nil {
		httpx.WriteError(w, http.StatusInternalServerError, "internal")
		return
	}
	httpx.WriteJSON(w, http.StatusCreated, user)
}
