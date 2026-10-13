"""Authentication service — login flow."""
from app.core.security import create_access_token, verify_password
from app.repositories.user_repository import UserRepository
from app.schemas.user import LoginIn, LoginOut


class InvalidCredentials(Exception):
    """Raised when email or password don't match."""


class AuthService:
    def __init__(self, repo: UserRepository) -> None:
        self.repo = repo

    def login(self, dto: LoginIn) -> LoginOut:
        user = self.repo.find_by_email(dto.email.lower().strip())
        if user is None or not verify_password(dto.password, user.password):
            raise InvalidCredentials
        token, expires_in = create_access_token(
            str(user.id), claims={"email": user.email, "role": user.role}
        )
        return LoginOut(access_token=token, expires_in=expires_in)
