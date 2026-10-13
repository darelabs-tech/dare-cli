# frozen_string_literal: true

# DARE v3.0 — SummarizeHandler (thin controller, delegates to service)
# Convention (ADR-01): HTTP concerns only — params, response codes, delegation
# No business logic here.
#
# POST /api/v1/documents/:id/summarize
# RFC 7807 errors via ProblemDetails concern (D-006)

class SummarizeHandler < ApplicationController
  include ProblemDetails

  before_action :authenticate_user!  # assume Devise or similar

  # POST /api/v1/documents/:id/summarize
  def create
    result = SummarizeDocumentService.new(
      document_repository: DocumentRepository.new,
      llm_provider:        LLM::Providers::LLMProvider.instance,
      event_publisher:     RealtimeService.instance
    ).execute(
      document_id: params[:id],
      user_id:     current_user.id
    )

    render json: { summary: result.summary, document_id: result.document_id }, status: :ok

  rescue SummarizeDocumentService::DocumentNotFoundError => e
    render_problem(status: 404, title: "Not Found", detail: e.message)
  rescue SummarizeDocumentService::SummarizationError => e
    render_problem(status: 422, title: "Summarization Failed", detail: e.message)
  end
end
