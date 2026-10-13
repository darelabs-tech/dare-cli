# frozen_string_literal: true

# DARE v3.0 — rswag OpenAPI spec for Users API
# M-02: running `rake rswag:specs:swaggerize` generates public/openapi.json from this
require "swagger_helper"

RSpec.describe "Users API", type: :request do
  path "/api/users" do
    post "Create a user" do
      tags        "Users"
      consumes    "application/json"
      produces    "application/json"
      description "Creates a new user. Returns 201 on success, 409 if email already taken."

      parameter name: :user, in: :body, schema: {
        type:       :object,
        required:   %w[email name],
        properties: {
          email: { type: :string, example: "alice@example.com" },
          name:  { type: :string, example: "Alice" }
        }
      }

      response "201", "User created successfully" do
        let(:user) { { email: "new@example.com", name: "New User" } }
        run_test!
      end

      response "409", "Email already taken" do
        schema "$ref" => "#/components/schemas/ProblemDetails"
        let(:user) { { email: create(:user).email, name: "Dup" } }
        run_test!
      end

      response "400", "Missing required parameters" do
        schema "$ref" => "#/components/schemas/ProblemDetails"
        let(:user) { {} }
        run_test!
      end
    end

    get "List active users" do
      tags     "Users"
      produces "application/json"

      response "200", "Users listed" do
        before { create_list(:user, 3) }
        run_test!
      end
    end
  end

  path "/api/users/{id}" do
    parameter name: :id, in: :path, type: :integer

    get "Show a user" do
      tags     "Users"
      produces "application/json"

      response "200", "User found" do
        let(:id) { create(:user).id }
        run_test!
      end

      response "404", "User not found" do
        schema "$ref" => "#/components/schemas/ProblemDetails"
        let(:id) { 99_999 }
        run_test!
      end
    end
  end
end
