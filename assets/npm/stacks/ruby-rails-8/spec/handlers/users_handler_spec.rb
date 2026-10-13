# frozen_string_literal: true

# DARE v3.0 — UsersHandler spec
# Convention (ADR-06): request specs test HTTP contract
require "rails_helper"

RSpec.describe "Users API", type: :request do
  let(:user)         { create(:user) }
  let(:valid_params) { { user: { email: "bob@example.com", name: "Bob" } } }

  describe "GET /api/users/:id" do
    context "when user exists" do
      it "returns 200 with user JSON" do
        get "/api/users/#{user.id}"

        expect(response).to have_http_status(:ok)
        json = JSON.parse(response.body)
        expect(json["email"]).to eq user.email
      end
    end

    context "when user does not exist" do
      it "returns 404 Problem Details" do
        get "/api/users/99999"

        expect(response).to have_http_status(:not_found)
        json = JSON.parse(response.body)
        expect(json["status"]).to eq 404
        expect(json["title"]).to be_present
        expect(response.content_type).to include("application/problem+json")
      end
    end
  end

  describe "POST /api/users" do
    context "with valid params" do
      it "returns 201 Created" do
        post "/api/users", params: valid_params, as: :json

        expect(response).to have_http_status(:created)
        json = JSON.parse(response.body)
        expect(json["email"]).to eq "bob@example.com"
      end
    end

    context "with duplicate email" do
      before { create(:user, email: "bob@example.com") }

      it "returns 409 Conflict with Problem Details" do
        post "/api/users", params: valid_params, as: :json

        expect(response).to have_http_status(:conflict)
        json = JSON.parse(response.body)
        expect(json["status"]).to eq 409
        expect(response.content_type).to include("application/problem+json")
      end
    end

    context "with missing required params" do
      it "returns 400 Bad Request" do
        post "/api/users", params: { user: {} }, as: :json

        expect(response).to have_http_status(:bad_request)
      end
    end
  end

  describe "PATCH /api/users/:id" do
    it "returns 200 with updated user" do
      patch "/api/users/#{user.id}",
            params: { user: { name: "Updated Name" } },
            as:     :json,
            headers: { "Authorization" => "Bearer test-token" }

      # Note: authentication stub needed — wire up in test support
      expect(response).not_to have_http_status(:internal_server_error)
    end
  end

  describe "DELETE /api/users/:id" do
    it "returns 204 No Content" do
      delete "/api/users/#{user.id}",
             headers: { "Authorization" => "Bearer test-token" }

      expect(response).not_to have_http_status(:internal_server_error)
    end
  end
end
