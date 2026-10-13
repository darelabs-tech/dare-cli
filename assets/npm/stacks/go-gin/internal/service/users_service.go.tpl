package service

import (
	"context"
	"errors"
	"strings"

	"golang.org/x/crypto/bcrypt"

	"{{moduleName}}/internal/config"
	"{{moduleName}}/internal/model"
	"{{moduleName}}/internal/repository"
)

var ErrEmailExists = errors.New("email already in use")

type UsersService struct {
	repo *repository.UsersRepository
	cfg  *config.Config
}

func NewUsersService(r *repository.UsersRepository, cfg *config.Config) *UsersService {
	return &UsersService{repo: r, cfg: cfg}
}

func (s *UsersService) Page(ctx context.Context, page, limit int) ([]*model.User, int64, error) {
	offset := int64((page - 1) * limit)
	items, err := s.repo.Page(ctx, offset, int64(limit))
	if err != nil {
		return nil, 0, err
	}
	total, err := s.repo.Count(ctx)
	if err != nil {
		return nil, 0, err
	}
	return items, total, nil
}

func (s *UsersService) Create(ctx context.Context, email, password, role string) (*model.User, error) {
	email = strings.TrimSpace(strings.ToLower(email))
	existing, err := s.repo.FindByEmail(ctx, email)
	if err != nil {
		return nil, err
	}
	if existing != nil {
		return nil, ErrEmailExists
	}
	hash, err := bcrypt.GenerateFromPassword([]byte(password), s.cfg.BcryptCost)
	if err != nil {
		return nil, err
	}
	return s.repo.Create(ctx, email, string(hash), role)
}
