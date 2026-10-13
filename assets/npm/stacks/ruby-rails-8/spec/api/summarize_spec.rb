# frozen_string_literal: true

# DARE v3.0 — rswag OpenAPI spec for Summarize Documents API
# M-02: running `rake rswag:specs:swaggerize` generates public/openapi.json from this
require "swagger_helper"

RSpec.describe "Summarize Documents API", type: :request do
  path "/api/v1/documents/{id}/summarize" do
    parameter name: :id, in: :path, type: :string, description: "Document ID"

    post "Summarize a document with LLM" do
      tags        "Documents"
      produces    "application/json"
      security    [{ bearerAuth: [] }]
      description <<~DESC
        Summarizes the document identified by :id using the configured LLM provider.
        Returns the generated summary and the document ID on success.
        RFC 7807 Problem Details on errors (D-006).
      DESC

      response "200", "Document summarized successfully" do
        schema type: :object,
               required: %w[summary document_id],
               properties: {
                 summary:     { type: :string, example: "This document discusses..." },
                 document_id: { type: :string, example: "doc-abc-123" }
               }

        let(:id) { create(:document, content: "Long content to summarize.").id }
        run_test!
      end

      response "401", "Unauthorized — missing or invalid authentication token" do
        schema "$ref" => "#/components/schemas/ProblemDetails"

        let(:id) { "any-id" }
        # No auth header provided
        run_test!
      end

      response "404", "Document not found" do
        schema "$ref" => "#/components/schemas/ProblemDetails"

        let(:id) { "non-existent-document-id" }
        run_test!
      end

      response "422", "Summarization failed (LLM returned empty result)" do
        schema "$ref" => "#/components/schemas/ProblemDetails"

        let(:id) { create(:document, content: "").id }
        run_test!
      end
    end
  end
end
