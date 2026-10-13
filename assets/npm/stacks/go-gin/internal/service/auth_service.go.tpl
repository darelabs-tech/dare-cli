// Package service — business logic. Calls repositories, never DB drivers.
package service

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"

	"{{moduleName}}/internal/config"
	"{{moduleName}}/internal/repository"
)

var ErrInvalidCredentials = errors.New("invalid credentials")

type AuthService struct {
	repo *repository.UsersRepository
	cfg  *config.Config
}

func NewAuthService(r *repository.UsersRepository, cfg *config.Config) *AuthService {
	return &AuthService{repo: r, cfg: cfg}
}

func (s *AuthService) Login(ctx context.Context, email, password string) (string, int64, error) {
	email = strings.TrimSpace(strings.ToLower(email))
	user, err := s.repo.FindByEmail(ctx, email)
	if err != nil {
		return "", 0, err
	}
	if user == nil {
		return "", 0, ErrInvalidCredentials
	}
	if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(password)); err != nil {
		return "", 0, ErrInvalidCredentials
	}

	expHours := s.cfg.JWTExpHours
	expiresAt := time.Now().Add(time.Duration(expHours) * time.Hour)
	claims := jwt.MapClaims{
		"sub":   user.ID.String(),
		"email": user.Email,
		"role":  user.Role,
		"exp":   expiresAt.Unix(),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	signed, err := token.SignedString([]byte(s.cfg.JWTSecret))
	if err != nil {
		return "", 0, err
	}
	return signed, int64(expHours) * 3600, nil
}
