# frozen_string_literal: true

# DARE v3.0 — CreateUserService
# Convention (ADR-01): one service per operation; injected dependencies for testability
#
# Usage:
#   result = CreateUserService.new(
#     user_repository: UserRepository.new,
#     event_publisher: RealtimeService.instance
#   ).execute(email: "user@example.com", name: "Alice")
#
# Returns: User (persisted)
# Raises:  CreateUserService::EmailTakenError
#          ActiveRecord::RecordInvalid (validation failure)

class CreateUserService
  # ── Domain errors ──────────────────────────────────────────────────────
  class EmailTakenError < StandardError
    def initialize(email)
      super("Email '#{email}' is already registered.")
    end
  end

  # ── Constructor ────────────────────────────────────────────────────────
  def initialize(user_repository:, event_publisher:)
    @user_repository = user_repository
    @event_publisher = event_publisher
  end

  # ── Main operation ─────────────────────────────────────────────────────
  def execute(email:, name:, password: nil, **extra_attrs)
    email = email.to_s.downcase.strip

    # Guard: check for duplicate email
    if @user_repository.find_by_email(email)
      raise EmailTakenError, email
    end

    # Build and persist
    user = @user_repository.build(
      email:      email,
      name:       name.to_s.strip,
      active:     true,
      **extra_attrs
    )

    # Hash password if provided (example using has_secure_password)
    user.password = password if password.present?

    @user_repository.save!(user)

    # Publish real-time event
    @event_publisher.publish(
      type: "user.created",
      data: {
        user_id: user.id,
        email:   user.email,
        name:    user.name,
        timestamp: Time.now.to_i
      }
    )

    user
  end
end
