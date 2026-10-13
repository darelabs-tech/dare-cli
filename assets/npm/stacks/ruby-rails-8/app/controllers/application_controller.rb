# frozen_string_literal: true

# DARE v3.0 — ApplicationController
# - CSRF protection active (RS-01)
# - RFC 7807 Problem Details included (D-006)
# - Thin controller: HTTP concerns only, delegates to Services

class ApplicationController < ActionController::API
  include ProblemDetails

  # Enforce JSON responses for all API endpoints
  before_action :set_default_response_format

  private

  def set_default_response_format
    request.format = :json
  end

  # Override in specific controllers if authentication is needed
  def current_user
    @current_user ||= authenticate_user_from_token
  end

  def authenticate_user_from_token
    token = request.headers["Authorization"]&.sub(/\ABearer /, "")
    return nil unless token

    # TODO: implement your token strategy here (JWT, session, etc.)
    # User.find_by_token(token)
    nil
  end

  def require_authentication!
    return if current_user

    render_problem(
      status: :unauthorized,
      title:  "Unauthorized",
      detail: "You must be authenticated to access this resource.",
      type:   "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401"
    )
  end
end
