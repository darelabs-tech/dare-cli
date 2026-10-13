# frozen_string_literal: true

# DARE v3.0 — UserRepository
# Convention (ADR-05): data access is isolated here — services never call AR directly
# This enables easy mocking in unit tests and future DB swaps

class UserRepository
  # ── Query ──────────────────────────────────────────────────────────────

  def find(id)
    User.find_by(id: id)
  end

  def find!(id)
    User.find(id)
  rescue ActiveRecord::RecordNotFound
    raise ActiveRecord::RecordNotFound, "User with id=#{id} was not found."
  end

  def find_by_email(email)
    User.find_by(email: email.downcase.strip)
  end

  def all_active
    User.active.order(created_at: :desc)
  end

  def paginated(page: 1, per: 25)
    User.active
        .order(created_at: :desc)
        .offset((page.to_i - 1) * per.to_i)
        .limit(per.to_i)
  end

  def count
    User.count
  end

  # ── Persistence ────────────────────────────────────────────────────────

  # Builds a new User (not persisted). Let the service validate input first.
  def build(attributes = {})
    User.new(attributes)
  end

  # Persist (create or update). Raises ActiveRecord::RecordInvalid on failure.
  def save!(user)
    user.save!
    user
  end

  # Persist without raising. Returns false on failure.
  def save(user)
    user.save
  end

  def update!(user, attributes)
    user.update!(attributes)
    user
  end

  def destroy!(user)
    user.destroy!
  end

  # ── Bulk ───────────────────────────────────────────────────────────────

  def deactivate_all_inactive(since:)
    User.where("last_active_at < ?", since).update_all(active: false)
  end
end
