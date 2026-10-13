# frozen_string_literal: true

# DARE v3.0 — rswag swagger_helper
# Defines OpenAPI spec structure; schemas are auto-generated from rswag specs

require "rails_helper"

RSpec.configure do |config|
  # Specify a root folder where Swagger JSON files are generated
  config.openapi_root = Rails.root.join("public").to_s

  config.openapi_specs = {
    "openapi.json" => {
      openapi: "3.0.1",
      info: {
        title: "API",
        version: "v1",
        description: "Rails 8 API built with DARE v3.0 methodology",
        contact: {
          name: "API Support",
          url:  "https://github.com/dewtech-technologies/dare-method"
        }
      },
      paths:      {},
      servers: [
        { url: "http://localhost:3000", description: "Development" },
        { url: "https://api.example.com", description: "Production" }
      ],
      components: {
        schemas: {
          ProblemDetails: {
            type:       :object,
            required:   %w[type title status],
            properties: {
              type:     { type: :string, description: "URI reference identifying problem type" },
              title:    { type: :string, description: "Short, human-readable summary" },
              status:   { type: :integer, description: "HTTP status code" },
              detail:   { type: :string, description: "Human-readable explanation" },
              instance: { type: :string, description: "URI reference to the specific occurrence" }
            },
            example: {
              type:     "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404",
              title:    "Not Found",
              status:   404,
              detail:   "User with id=99 was not found.",
              instance: "/api/users/99"
            }
          },
          User: {
            type:       :object,
            properties: {
              id:         { type: :integer },
              email:      { type: :string },
              name:       { type: :string },
              active:     { type: :boolean },
              created_at: { type: :string, format: "date-time" },
              updated_at: { type: :string, format: "date-time" }
            }
          }
        },
        securitySchemes: {
          bearerAuth: {
            type:         :http,
            scheme:       :bearer,
            bearerFormat: "JWT"
          }
        }
      }
    }
  }

  config.openapi_format = :json
end
