# frozen_string_literal: true

# DARE v3.0 — DocumentRepository (interface + ActiveRecord implementation)
# Convention (ADR-05): data access is isolated here — services never call AR directly
# This enables easy mocking in unit tests and future DB swaps

class DocumentRepository
  # ── Query ──────────────────────────────────────────────────────────────

  def find(id)
    Document.find_by(id: id)
  end

  def find!(id)
    Document.find(id)
  rescue ActiveRecord::RecordNotFound
    raise ActiveRecord::RecordNotFound, "Document with id=#{id} was not found."
  end

  def all
    Document.order(created_at: :desc)
  end

  def by_user(user_id)
    Document.where(user_id: user_id).order(created_at: :desc)
  end

  # ── Persistence ────────────────────────────────────────────────────────

  # Builds a new Document (not persisted).
  def build(attrs = {})
    Document.new(attrs)
  end

  # Persist (create or update). Raises ActiveRecord::RecordInvalid on failure.
  def save!(document)
    document.save!
    document
  end

  # Update summary and timestamp after LLM processing.
  def update_summary!(document, summary)
    document.update!(summary: summary, summarized_at: Time.current)
    document
  end

  def update!(document, attributes)
    document.update!(attributes)
    document
  end

  def destroy!(document)
    document.destroy!
  end
end
