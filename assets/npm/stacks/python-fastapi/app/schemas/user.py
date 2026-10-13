"""Pydantic IN/OUT schemas for user resource."""
import re
import uuid
from datetime import datetime
from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, EmailStr, Field, field_validator


class LoginIn(BaseModel):
    email: EmailStr
    password: Annotated[str, Field(min_length=8)]

    @field_validator("password")
    @classmethod
    def password_complexity(cls, v: str) -> str:
        if not re.search(r"[A-Z]", v):
            raise ValueError("password must contain an uppercase letter")
        if not re.search(r"\d", v):
            raise ValueError("password must contain a digit")
        return v


class LoginOut(BaseModel):
    access_token: str
    expires_in: int
    token_type: Literal["bearer"] = "bearer"


class UserOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    id: uuid.UUID
    email: EmailStr
    role: str
    created_at: datetime


class UserCreate(BaseModel):
    email: EmailStr
    password: Annotated[str, Field(min_length=8)]
    role: Literal["USER", "ADMIN"] = "USER"

    @field_validator("password")
    @classmethod
    def password_complexity(cls, v: str) -> str:
        if not re.search(r"[A-Z]", v):
            raise ValueError("password must contain an uppercase letter")
        if not re.search(r"\d", v):
            raise ValueError("password must contain a digit")
        return v


class UserPage(BaseModel):
    items: list[UserOut]
    total: int
    page: int
