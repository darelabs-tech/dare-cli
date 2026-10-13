"""Smoke tests for auth router using httpx + FastAPI's TestClient."""
from fastapi.testclient import TestClient

from app.main import app

client = TestClient(app)


def test_healthz_returns_ok() -> None:
    res = client.get("/healthz")
    assert res.status_code == 200
    assert res.json() == {"status": "ok"}


def test_login_rejects_bad_email() -> None:
    res = client.post("/auth/login", json={"email": "not-an-email", "password": "Str0ngPass"})
    assert res.status_code == 422


def test_login_rejects_short_password() -> None:
    res = client.post("/auth/login", json={"email": "u@example.com", "password": "short"})
    assert res.status_code == 422
