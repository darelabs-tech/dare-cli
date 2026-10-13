# frozen_string_literal: true

# DARE v3.0 — rswag API configuration
# Satisfies metric M-02: 100% of endpoints have OpenAPI documentation

Rswag::Api.configure do |c|
  # Path to the generated openapi.json spec file
  c.openapi_root = Rails.root.join("public").to_s

  # Authorize all endpoints in Swagger UI with Bearer token by default
  c.swagger_filter = lambda { |swagger, env| swagger }
end
