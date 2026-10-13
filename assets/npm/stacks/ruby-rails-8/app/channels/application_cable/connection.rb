# frozen_string_literal: true

# DARE v3.0 — Action Cable Connection with cookie-based auth
# Convention (ADR-03): Every connection validates authentication
# Unauthorized connections are rejected at the connection level

module ApplicationCable
  class Connection < ActionCable::Connection::Base
    identified_by :current_user

    def connect
      self.current_user = find_verified_user
    end

    private

    def find_verified_user
      # Strategy 1: cookie-based session (most common for Rails apps)
      if (user_id = cookies.signed[:user_id])
        user = User.find_by(id: user_id)
        return user if user&.active?
      end

      # Strategy 2: token-based (for mobile/API clients)
      if (token = request.params[:token].presence || env["HTTP_X_ACTION_CABLE_TOKEN"])
        # Implement your token lookup here:
        # user = User.find_by_auth_token(token)
        # return user if user&.active?
      end

      reject_unauthorized_connection
    end
  end
end
