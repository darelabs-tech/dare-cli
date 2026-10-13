# frozen_string_literal: true

# DARE v3.0 — SummarizeDocumentService spec
# Convention (ADR-06): unit tests with injected doubles — no database, no network
require "rails_helper"

RSpec.describe SummarizeDocumentService do
  subject(:service) do
    described_class.new(
      document_repository: document_repository,
      llm_provider:        llm_provider,
      event_publisher:     event_publisher
    )
  end

  let(:document_repository) { instance_double(DocumentRepository) }
  let(:llm_provider)        { instance_double(LLM::Providers::LLMProvider) }
  let(:event_publisher)     { instance_double(RealtimeService) }

  let(:document) do
    instance_double(Document,
      id:      "doc-123",
      content: "This is a very long document that needs summarizing."
    )
  end

  let(:summary_text) { "Short summary of the document." }

  before do
    allow(LLM::Prompts::PromptLoader).to receive(:load).and_return("rendered prompt")
    allow(event_publisher).to receive(:publish)
  end

  describe "#execute" do
    context "with valid document" do
      before do
        allow(document_repository).to receive(:find).with("doc-123").and_return(document)
        allow(llm_provider).to receive(:complete).and_return(summary_text)
        allow(document_repository).to receive(:update_summary!).with(document, summary_text).and_return(document)
      end

      it "returns a Result with summary and document_id" do
        result = service.execute(document_id: "doc-123", user_id: 42)

        expect(result).to be_a(SummarizeDocumentService::Result)
        expect(result.summary).to eq(summary_text)
        expect(result.document_id).to eq("doc-123")
      end

      it "persists the summary via repository" do
        service.execute(document_id: "doc-123", user_id: 42)

        expect(document_repository).to have_received(:update_summary!).with(document, summary_text)
      end

      it "publishes a document.summarized event" do
        service.execute(document_id: "doc-123", user_id: 42)

        expect(event_publisher).to have_received(:publish).with(
          type: "document.summarized",
          data: hash_including(
            document_id: "doc-123",
            user_id:     42,
            summary_length: summary_text.length
          )
        )
      end

      it "calls PromptLoader with document content and language" do
        service.execute(document_id: "doc-123", user_id: 42)

        expect(LLM::Prompts::PromptLoader).to have_received(:load).with(
          "summarize_v1",
          content:  document.content,
          language: "pt-BR"
        )
      end
    end

    context "when document is not found" do
      before do
        allow(document_repository).to receive(:find).with("missing-id").and_return(nil)
      end

      it "raises DocumentNotFoundError" do
        expect {
          service.execute(document_id: "missing-id", user_id: 42)
        }.to raise_error(SummarizeDocumentService::DocumentNotFoundError, /missing-id/)
      end

      it "does not call the LLM provider" do
        service.execute(document_id: "missing-id", user_id: 42) rescue nil

        expect(llm_provider).not_to have_received(:complete)
      end

      it "does not publish any event" do
        service.execute(document_id: "missing-id", user_id: 42) rescue nil

        expect(event_publisher).not_to have_received(:publish)
      end
    end

    context "when LLM returns blank response" do
      before do
        allow(document_repository).to receive(:find).with("doc-123").and_return(document)
        allow(llm_provider).to receive(:complete).and_return("")
      end

      it "raises SummarizationError" do
        expect {
          service.execute(document_id: "doc-123", user_id: 42)
        }.to raise_error(SummarizeDocumentService::SummarizationError, /empty summary/)
      end

      it "does not persist any summary" do
        service.execute(document_id: "doc-123", user_id: 42) rescue nil

        expect(document_repository).not_to have_received(:update_summary!)
      end

      it "does not publish any event" do
        service.execute(document_id: "doc-123", user_id: 42) rescue nil

        expect(event_publisher).not_to have_received(:publish)
      end
    end

    context "when LLM returns nil" do
      before do
        allow(document_repository).to receive(:find).with("doc-123").and_return(document)
        allow(llm_provider).to receive(:complete).and_return(nil)
      end

      it "raises SummarizationError" do
        expect {
          service.execute(document_id: "doc-123", user_id: 42)
        }.to raise_error(SummarizeDocumentService::SummarizationError)
      end
    end
  end
end
