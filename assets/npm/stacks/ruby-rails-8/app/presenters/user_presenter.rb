# frozen_string_literal: true

# DARE v3.0 — UserPresenter
# Convention (ADR-05): serialization concern isolated from model and controller
# Returns plain Hash; handler renders JSON from it

class UserPresenter
  def initialize(user)
    @user = user
  end

  # Full representation — used in show/create responses
  def as_json(*)
    {
      id:         @user.id,
      email:      @user.email,
      name:       @user.name,
      active:     @user.active?,
      created_at: @user.created_at&.iso8601,
      updated_at: @user.updated_at&.iso8601,
    }
  end

  def to_json(*)
    as_json.to_json
  end

  # Collection representation — used in index responses
  # Wraps an array of presenters into a standard envelope
  def self.collection(users, meta: {})
    {
      data: users.map { |u| new(u).as_json },
      meta: {
        count: users.size,
        **meta
      }
    }
  end

  # Minimal summary representation (e.g. for nested resources)
  def summary
    {
      id:    @user.id,
      email: @user.email,
      name:  @user.name
    }
  end
end
