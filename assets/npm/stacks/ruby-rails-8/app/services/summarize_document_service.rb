# frozen_string_literal: true

# DARE v3.0 — SummarizeDocumentService
# Convention (ADR-01): single responsibility — load document, call LLM, persist summary, emit event
#
# Usage:
#   result = SummarizeDocumentService.new(
#     document_repository: DocumentRepository.new,
#     llm_provider:        LLM::Providers::LLMProvider.instance,
#     event_publisher:     RealtimeService.instance
#   ).execute(document_id: "abc-123", user_id: 42)
#
# Returns: SummarizeDocumentService::Result (summary, document_id)
# Raises:  SummarizeDocumentService::DocumentNotFoundError
#          SummarizeDocumentService::SummarizationError

class SummarizeDocumentService
  # ── Domain errors ──────────────────────────────────────────────────────
  DocumentNotFoundError = Class.new(StandardError)
  SummarizationError    = Class.new(StandardError)

  # ── Result ─────────────────────────────────────────────────────────────
  Result = Struct.new(:summary, :document_id, keyword_init: true)

  # ── Constructor ────────────────────────────────────────────────────────
  def initialize(document_repository:, llm_provider:, event_publisher:)
    @document_repository = document_repository
    @llm_provider        = llm_provider
    @event_publisher     = event_publisher
  end

  # ── Main operation ─────────────────────────────────────────────────────
  def execute(document_id:, user_id:)
    doc = @document_repository.find(document_id)
    raise DocumentNotFoundError, "Document #{document_id} not found" if doc.nil?

    prompt  = LLM::Prompts::PromptLoader.load("summarize_v1", content: doc.content, language: "pt-BR")
    summary = @llm_provider.complete(model: "gpt-4o-mini", prompt: prompt)

    raise SummarizationError, "LLM returned empty summary" if summary.blank?

    @document_repository.update_summary!(doc, summary)

    @event_publisher.publish(
      type: "document.summarized",
      data: {
        document_id:    document_id,
        user_id:        user_id,
        summary_length: summary.length
      }
    )

    Result.new(summary: summary, document_id: document_id)
  end
end
