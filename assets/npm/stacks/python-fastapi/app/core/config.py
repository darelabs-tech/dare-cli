"""App settings — Pydantic Settings reads from .env / environment."""
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")

    app_port: int = 8000
    log_level: str = "info"

    database_url: str = "postgresql+psycopg://user:pass@localhost:5432/dbname"

    jwt_secret: str = "replace-me-in-prod"
    jwt_algorithm: str = "HS256"
    jwt_expires_min: int = 15

    bcrypt_rounds: int = 12

    rate_limit_per_min: int = 60

    cors_origins: list[str] = ["http://localhost:3000"]


settings = Settings()
