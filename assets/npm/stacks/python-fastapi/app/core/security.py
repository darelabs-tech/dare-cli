"""JWT + password hashing helpers."""
from datetime import datetime, timedelta, timezone
from typing import Any

from jose import JWTError, jwt
from passlib.context import CryptContext

from app.core.config import settings

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto", bcrypt__rounds=settings.bcrypt_rounds)


def hash_password(plain: str) -> str:
    return pwd_context.hash(plain)


def verify_password(plain: str, hashed: str) -> bool:
    return pwd_context.verify(plain, hashed)


def create_access_token(sub: str, claims: dict[str, Any] | None = None) -> tuple[str, int]:
    expires = datetime.now(timezone.utc) + timedelta(minutes=settings.jwt_expires_min)
    payload: dict[str, Any] = {"sub": sub, "exp": expires}
    if claims:
        payload.update(claims)
    token = jwt.encode(payload, settings.jwt_secret, algorithm=settings.jwt_algorithm)
    return token, settings.jwt_expires_min * 60


def decode_access_token(token: str) -> dict[str, Any] | None:
    try:
        return jwt.decode(token, settings.jwt_secret, algorithms=[settings.jwt_algorithm])
    except JWTError:
        return None
