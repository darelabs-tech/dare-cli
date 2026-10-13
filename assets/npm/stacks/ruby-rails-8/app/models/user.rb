# frozen_string_literal: true

# DARE v3.0 — User domain model
# Convention (ADR-05): models are lean data objects — no callbacks, no fat model
# Business logic lives in app/services/; data access in app/repositories/

class User < ApplicationRecord
  # ── Validations ─────────────────────────────────────────────────────────
  validates :email, presence: true,
                    uniqueness: { case_sensitive: false },
                    format: { with: URI::MailTo::EMAIL_REGEXP }
  validates :name,  presence: true, length: { minimum: 2, maximum: 100 }

  # ── Scopes ───────────────────────────────────────────────────────────────
  scope :active, -> { where(active: true) }

  # ── DARE: NO callbacks, NO fat model logic ───────────────────────────────
  # Use Services for: password hashing, welcome emails, event publishing, etc.
  # Use Repositories for: queries, bulk updates, custom SQL
  #
  # WRONG (fat model):
  #   before_create :send_welcome_email
  #   after_save    :publish_event
  #
  # RIGHT (service):
  #   CreateUserService.new(...).execute(params)

  # ── Simple domain helpers (pure data concerns only) ──────────────────────
  def display_name
    name.presence || email.split("@").first
  end

  def can_view?(user_id)
    id == user_id.to_i || admin?
  end
end
