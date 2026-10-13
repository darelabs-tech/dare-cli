"""Users router: GET /users, POST /users."""
from fastapi import APIRouter, Depends, HTTPException, Query, status
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from sqlalchemy.orm import Session

from app.core.security import decode_access_token
from app.db.session import get_db
from app.repositories.user_repository import UserRepository
from app.schemas.user import UserCreate, UserOut, UserPage
from app.services.user_service import EmailAlreadyInUse, UserService

router = APIRouter()
bearer = HTTPBearer(auto_error=True)


def _require_admin(creds: HTTPAuthorizationCredentials) -> None:
    payload = decode_access_token(creds.credentials)
    if not payload or payload.get("role") != "ADMIN":
        raise HTTPException(status.HTTP_403_FORBIDDEN, "admin role required")


@router.get("", response_model=UserPage)
def list_users(
    page: int = Query(1, ge=1),
    limit: int = Query(20, ge=1, le=100),
    creds: HTTPAuthorizationCredentials = Depends(bearer),
    db: Session = Depends(get_db),
) -> UserPage:
    payload = decode_access_token(creds.credentials)
    if not payload:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Invalid token")
    return UserService(UserRepository(db)).list(page, limit)


@router.post("", response_model=UserOut, status_code=status.HTTP_201_CREATED)
def create_user(
    dto: UserCreate,
    creds: HTTPAuthorizationCredentials = Depends(bearer),
    db: Session = Depends(get_db),
) -> UserOut:
    _require_admin(creds)
    service = UserService(UserRepository(db))
    try:
        return service.create(dto)
    except EmailAlreadyInUse as exc:
        raise HTTPException(status.HTTP_409_CONFLICT, "email already in use") from exc
