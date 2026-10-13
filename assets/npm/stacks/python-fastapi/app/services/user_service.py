"""User service — create/list business logic."""
from app.core.security import hash_password
from app.repositories.user_repository import UserRepository
from app.schemas.user import UserCreate, UserOut, UserPage


class EmailAlreadyInUse(Exception):
    """Raised when a user with the same email already exists."""


class UserService:
    def __init__(self, repo: UserRepository) -> None:
        self.repo = repo

    def list(self, page: int, limit: int) -> UserPage:
        items, total = self.repo.page(page, limit)
        return UserPage(
            items=[UserOut.model_validate(u) for u in items],
            total=total,
            page=max(1, page),
        )

    def create(self, dto: UserCreate) -> UserOut:
        if self.repo.find_by_email(dto.email.lower().strip()) is not None:
            raise EmailAlreadyInUse
        created = self.repo.create(
            email=dto.email.lower().strip(),
            password=hash_password(dto.password),
            role=dto.role,
        )
        return UserOut.model_validate(created)
