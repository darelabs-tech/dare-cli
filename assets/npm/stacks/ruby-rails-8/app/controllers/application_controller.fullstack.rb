# frozen_string_literal: true

# DARE v3.0 — ApplicationController (full-stack MVC)
# - Inherits ActionController::Base: view rendering, CSRF, cookies, flash
# - RFC 7807 Problem Details still available for JSON/API endpoints (D-006)
# - Thin controller: HTTP concerns only, delegates to Services

class ApplicationController < ActionController::Base
  include ProblemDetails

  # CSRF protection for HTML forms (RS-01)
  protect_from_forgery with: :exception

  # Rails 8 default — serve modern browsers only
  allow_browser versions: :modern

  private

  # Override in specific controllers if authentication is needed
  def current_user
    @current_user ||= authenticate_user_from_session
  end

  def authenticate_user_from_session
    return nil unless session[:user_id]

    # TODO: implement your session strategy here
    # User.find_by(id: session[:user_id])
    nil
  end

  def require_authentication!
    return if current_user

    respond_to do |format|
      format.html { redirect_to root_path, alert: "You must be signed in to continue." }
      format.json do
        render_problem(
          status: :unauthorized,
          title:  "Unauthorized",
          detail: "You must be authenticated to access this resource.",
          type:   "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401"
        )
      end
    end
  end
end
