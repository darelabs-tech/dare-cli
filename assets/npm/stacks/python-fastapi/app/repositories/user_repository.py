"""User repository — encapsulates SQLAlchemy queries."""
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from app.models.user import User


class UserRepository:
    def __init__(self, db: Session) -> None:
        self.db = db

    def find_by_email(self, email: str) -> User | None:
        return self.db.execute(select(User).where(User.email == email)).scalar_one_or_none()

    def page(self, page: int, limit: int) -> tuple[list[User], int]:
        page = max(1, page)
        limit = min(100, max(1, limit))
        offset = (page - 1) * limit
        items = list(
            self.db.execute(
                select(User).order_by(User.created_at.desc()).offset(offset).limit(limit)
            )
            .scalars()
            .all()
        )
        total = self.db.execute(select(func.count()).select_from(User)).scalar_one()
        return items, total

    def create(self, *, email: str, password: str, role: str) -> User:
        user = User(email=email, password=password, role=role)
        self.db.add(user)
        self.db.commit()
        self.db.refresh(user)
        return user
