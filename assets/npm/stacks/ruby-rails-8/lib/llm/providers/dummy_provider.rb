# frozen_string_literal: true

# DARE v3.0 — DummyProvider for testing and development
# Returns deterministic canned responses — no network calls

module LLM
  module Providers
    class DummyProvider < LLMProvider
      DEFAULT_RESPONSE = "This is a dummy LLM response. Configure a real provider in config/dare.yml."

      def initialize(responses: {})
        # Allow injecting specific responses by prompt keyword for tests:
        # DummyProvider.new(responses: { "summarize" => "Summary here" })
        @responses = responses
      end

      def complete(model:, prompt:, max_tokens: 1024, temperature: 0.7, **_opts)
        matched_key = @responses.keys.find { |k| prompt.to_s.downcase.include?(k.to_s.downcase) }
        @responses.fetch(matched_key, DEFAULT_RESPONSE)
      end

      def chat(model:, messages:, max_tokens: 1024, temperature: 0.7, **_opts)
        last_content = messages.last&.dig(:content) || messages.last&.dig("content") || ""
        complete(model: model, prompt: last_content, max_tokens: max_tokens)
      end

      def stream(model:, prompt:, max_tokens: 1024, &block)
        response = complete(model: model, prompt: prompt, max_tokens: max_tokens)
        response.chars.each_slice(10).map(&:join).each do |chunk|
          block.call(chunk)
        end
      end
    end
  end
end
