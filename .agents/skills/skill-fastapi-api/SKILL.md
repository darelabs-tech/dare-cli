---
name: skill-fastapi-api
description: Padrões DARE para APIs REST em Python + FastAPI + Pydantic + uvicorn. Routers, dependency injection, Pydantic v2 schemas, async SQLAlchemy 2.0, autenticação OAuth2 + JWT, rate limit com slowapi, pytest + httpx, OpenAPI auto-gerado.
---

# DARE FastAPI Skill

Você é um desenvolvedor sênior Python especialista em APIs REST com FastAPI. Seu objetivo é gerar código **assíncrono, fortemente tipado (Pydantic v2), com OpenAPI auto-gerado, auth/autz robustos**, seguindo Layered Design DARE.

## Quando usar

- Projeto FastAPI novo via DARE
- Adicionar feature em API FastAPI existente
- Migrar de Flask/Django para FastAPI
- Auditar projeto FastAPI para conformidade DARE

## Stack canônica

- **Python 3.11+** com type hints obrigatórios
- **FastAPI 0.115+** com async/await
- **Pydantic v2** para schemas
- **SQLAlchemy 2.0** async + **asyncpg** (PostgreSQL)
- **alembic** para migrations
- **passlib + argon2** para hash de senhas
- **python-jose** ou **PyJWT** para JWT
- **slowapi** para rate limiting
- **pytest + pytest-asyncio + httpx** para testes
- **ruff** para lint + format
- **mypy** para type checking

## Layered Design em FastAPI

```
app/
├── main.py                       ← FastAPI app + middlewares
├── core/
│   ├── config.py                 ← Settings via pydantic-settings
│   └── security.py               ← hash, JWT
├── api/
│   ├── deps.py                   ← Depends() comuns
│   └── v1/
│       ├── users.py              ← Handler (router)
│       └── auth.py
├── services/
│   └── register_user.py          ← Service
├── repositories/
│   └── users.py                  ← Repository
├── models/                       ← SQLAlchemy ORM
│   └── user.py
├── schemas/                      ← Pydantic DTOs
│   ├── user.py
│   └── auth.py
└── tests/
```

## Routers (Handler)

```python
from fastapi import APIRouter, Depends, HTTPException, status
from app.api.deps import get_current_user
from app.schemas.user import UserCreate, UserOut
from app.services.register_user import RegisterUser, UserAlreadyExistsError

router = APIRouter(prefix="/users", tags=["users"])

@router.post("", response_model=UserOut, status_code=status.HTTP_201_CREATED)
async def create_user(
    payload: UserCreate,
    service: RegisterUser = Depends(),
    _current: User = Depends(get_current_user),
):
    try:
        return await service.execute(payload)
    except UserAlreadyExistsError:
        raise HTTPException(status_code=409, detail="User already exists")
```

Regras:
- Apenas: valida via Pydantic → chama Service → retorna response_model
- NUNCA: SQLAlchemy direto, lógica de negócio

## Schemas (Pydantic v2)

```python
from pydantic import BaseModel, EmailStr, Field
from datetime import datetime

class UserCreate(BaseModel):
    email: EmailStr
    name: str = Field(min_length=1, max_length=255)
    password: str = Field(min_length=12)

class UserOut(BaseModel):
    id: int
    email: EmailStr
    name: str
    created_at: datetime

    model_config = {"from_attributes": True}  # Pydantic v2 / ex-orm_mode
```

## Services

```python
from app.repositories.users import UsersRepository
from app.core.security import hash_password
from app.schemas.user import UserCreate
from app.models.user import User

class UserAlreadyExistsError(Exception): ...

class RegisterUser:
    def __init__(self, repo: UsersRepository = Depends()):
        self.repo = repo

    async def execute(self, payload: UserCreate) -> User:
        if await self.repo.exists_by_email(payload.email):
            raise UserAlreadyExistsError()
        return await self.repo.create(
            email=payload.email,
            name=payload.name,
            password_hash=hash_password(payload.password),
        )
```

## Repositories (SQLAlchemy async 2.0)

```python
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import Depends
from app.api.deps import get_db
from app.models.user import User

class UsersRepository:
    def __init__(self, db: AsyncSession = Depends(get_db)):
        self.db = db

    async def exists_by_email(self, email: str) -> bool:
        result = await self.db.execute(select(User.id).where(User.email == email))
        return result.scalar_one_or_none() is not None

    async def create(self, *, email: str, name: str, password_hash: str) -> User:
        user = User(email=email, name=name, password_hash=password_hash)
        self.db.add(user)
        await self.db.commit()
        await self.db.refresh(user)
        return user
```

## Models (SQLAlchemy)

```python
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column
from datetime import datetime

class Base(DeclarativeBase): ...

class User(Base):
    __tablename__ = "users"
    id: Mapped[int] = mapped_column(primary_key=True)
    email: Mapped[str] = mapped_column(unique=True, index=True)
    name: Mapped[str]
    password_hash: Mapped[str]
    created_at: Mapped[datetime] = mapped_column(default=datetime.utcnow)
```

## Auth (OAuth2 + JWT)

```python
# app/core/security.py
from passlib.hash import argon2
from datetime import datetime, timedelta
from jose import jwt
from app.core.config import settings

def hash_password(p: str) -> str: return argon2.hash(p)
def verify_password(p: str, h: str) -> bool: return argon2.verify(p, h)

def create_access_token(sub: str) -> str:
    expire = datetime.utcnow() + timedelta(minutes=15)
    payload = {"sub": sub, "exp": expire}
    return jwt.encode(payload, settings.JWT_SECRET, algorithm="HS256")
```

```python
# app/api/deps.py
from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from jose import jwt, JWTError
from app.core.config import settings

oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/v1/auth/login")

async def get_current_user(token: str = Depends(oauth2_scheme)) -> User:
    try:
        payload = jwt.decode(token, settings.JWT_SECRET, algorithms=["HS256"])
        sub: str = payload.get("sub")
    except JWTError:
        raise HTTPException(status_code=401, detail="Invalid token")
    # buscar user no DB ...
    return user
```

## Rate limiting (slowapi)

```python
from slowapi import Limiter
from slowapi.util import get_remote_address

limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter
app.add_middleware(SlowAPIMiddleware)

@router.post("/login")
@limiter.limit("5/15minute")
async def login(request: Request, ...):
    ...
```

## OpenAPI

FastAPI auto-gera em `/openapi.json`. Customize title/version/auth:

```python
app = FastAPI(
    title="Projeto API",
    version="1.0",
    openapi_url="/openapi.json",
    docs_url="/docs",
    redoc_url="/redoc",
)
```

## Settings (pydantic-settings)

```python
# app/core/config.py
from pydantic_settings import BaseSettings, SettingsConfigDict

class Settings(BaseSettings):
    DATABASE_URL: str
    JWT_SECRET: str
    JWT_EXPIRE_MINUTES: int = 15
    model_config = SettingsConfigDict(env_file=".env")

settings = Settings()
```

## Testes

```python
# tests/test_users.py
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_create_user(client: AsyncClient, admin_token: str):
    response = await client.post(
        "/v1/users",
        json={"email": "jane@example.com", "name": "Jane", "password": "longsecret123"},
        headers={"Authorization": f"Bearer {admin_token}"},
    )
    assert response.status_code == 201
    assert response.json()["email"] == "jane@example.com"

@pytest.mark.asyncio
async def test_duplicate_email(client: AsyncClient, admin_token: str):
    # cria primeiro
    await client.post("/v1/users", json={...}, headers={...})
    # tenta de novo
    response = await client.post("/v1/users", json={...}, headers={...})
    assert response.status_code == 409
```

## Antipatterns

| AP | Antipattern | Correção |
|---|---|---|
| AP-01 | SQLAlchemy direto no router | Repository injetado via Depends |
| AP-02 | Validação manual no router | Pydantic schema |
| AP-03 | Lógica no router | Service |
| AP-04 | `return user` (ORM) sem response_model | `response_model=UserOut` |
| AP-05 | Secret hardcoded | `pydantic-settings` + `.env` |
| AP-06 | Login sem rate limit | `@limiter.limit("5/15minute")` |
| AP-07 | `password=user.password` em response | `response_model=UserOut` sem password |
| AP-08 | Sem `from_attributes=True` | response_model não converte ORM |

## Segurança

- Senhas: `argon2.hash()` (passlib)
- JWT: HS256 com `JWT_SECRET ≥ 32 bytes`, ou RS256 com par de chaves
- Headers: middleware com HSTS, X-Frame-Options, CSP
- CORS específico, nunca `*`
- Rate limit em login + APIs públicas
- Refresh token com rotação

## CI

```bash
ruff check .
ruff format --check .
mypy app
pytest --cov=app --cov-fail-under=80
pip-audit
alembic upgrade head --sql > /dev/null   # valida migrations
```

## Como aplicar

### Passo 1: Audit

```bash
grep -rn "db\.execute\|db\.query" app/api/        # AP-01
grep -rn "request\.json()" app/api/               # AP-02
grep -rn "JWT_SECRET\s*=\s*['\"]" app/            # AP-05
```

### Passo 2: Migrar para Pydantic schemas

Para cada `request.json()` ou validação inline → schema Pydantic.

### Passo 3: Extrair Services

Lógica > 5 linhas no router → Service injetável.

### Passo 4: Adicionar Repositories

Encapsular SQLAlchemy. Router chama Service via Depends, Service chama Repository.

### Passo 5: Configurar slowapi + auth

`app.add_middleware(SlowAPIMiddleware)` global + `@limiter.limit` em login.

## Dicas

- **Combine** com `dare-docker` (Python 3.11-slim, não-root, multi-stage)
- **Use** `dare-llm-integration` se houver Gemini/Claude (FastAPI + httpx async)
- **Para realtime**, FastAPI tem `WebSocket` nativo + `sse-starlette`

---

Esta skill é parte do DARE Method e está sob licença MIT.
