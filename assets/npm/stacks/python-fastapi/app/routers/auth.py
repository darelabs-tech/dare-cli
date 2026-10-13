"""Auth router: POST /auth/login, GET /auth/me."""
from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from sqlalchemy.orm import Session

from app.core.security import decode_access_token
from app.db.session import get_db
from app.repositories.user_repository import UserRepository
from app.schemas.user import LoginIn, LoginOut, UserOut
from app.services.auth_service import AuthService, InvalidCredentials

router = APIRouter()
bearer = HTTPBearer(auto_error=True)


@router.post("/login", response_model=LoginOut)
def login(dto: LoginIn, db: Session = Depends(get_db)) -> LoginOut:
    service = AuthService(UserRepository(db))
    try:
        return service.login(dto)
    except InvalidCredentials as exc:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Invalid credentials") from exc


@router.get("/me", response_model=UserOut)
def me(
    creds: HTTPAuthorizationCredentials = Depends(bearer),
    db: Session = Depends(get_db),
) -> UserOut:
    payload = decode_access_token(creds.credentials)
    if not payload or "sub" not in payload:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Invalid token")
    repo = UserRepository(db)
    user = repo.find_by_email(payload.get("email", ""))
    if user is None:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "User not found")
    return UserOut.model_validate(user)
