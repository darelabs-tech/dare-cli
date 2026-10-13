# frozen_string_literal: true

# DARE v3.0 — RFC 7807 Problem Details concern
# Decision: D-006 — all HTTP errors use this format
#
# Usage: include ProblemDetails in ApplicationController
#
# Format:
#   {
#     "type":     "https://example.com/problems/not-found",
#     "title":    "Resource Not Found",
#     "status":   404,
#     "detail":   "User with id=42 was not found.",
#     "instance": "/api/users/42"
#   }

module ProblemDetails
  extend ActiveSupport::Concern

  included do
    rescue_from StandardError,                    with: :render_internal_error
    rescue_from ActiveRecord::RecordNotFound,     with: :render_not_found
    rescue_from ActiveRecord::RecordInvalid,      with: :render_unprocessable_entity
    rescue_from ActionController::ParameterMissing, with: :render_bad_request
    rescue_from ArgumentError,                    with: :render_bad_request
  end

  # ── Public helpers ──────────────────────────────────────────────────────────

  # Render a Problem Details response manually.
  #
  #   render_problem(status: :not_found, title: "User Not Found", detail: "...")
  def render_problem(status:, title:, detail: nil, type: nil, **extra)
    http_status = Rack::Utils.status_code(status)

    body = {
      type:     type || default_type_uri(http_status),
      title:    title,
      status:   http_status,
      detail:   detail,
      instance: request.path,
    }.merge(extra).compact

    render json: body,
           status: http_status,
           content_type: "application/problem+json"
  end

  private

  def render_not_found(exception)
    render_problem(
      status: :not_found,
      title:  "Not Found",
      detail: exception.message.presence || "The requested resource does not exist."
    )
  end

  def render_unprocessable_entity(exception)
    errors = exception.record&.errors&.full_messages || [exception.message]
    render_problem(
      status:  :unprocessable_entity,
      title:   "Validation Failed",
      detail:  "One or more fields failed validation.",
      errors:  errors
    )
  end

  def render_bad_request(exception)
    render_problem(
      status: :bad_request,
      title:  "Bad Request",
      detail: exception.message
    )
  end

  def render_internal_error(exception)
    # Never leak internals in production
    detail = Rails.env.production? ? "An unexpected error occurred." : exception.message

    Rails.logger.error "[ProblemDetails] #{exception.class}: #{exception.message}\n#{exception.backtrace&.first(10)&.join("\n")}"

    render_problem(
      status: :internal_server_error,
      title:  "Internal Server Error",
      detail: detail
    )
  end

  def default_type_uri(status_code)
    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/#{status_code}"
  end
end
