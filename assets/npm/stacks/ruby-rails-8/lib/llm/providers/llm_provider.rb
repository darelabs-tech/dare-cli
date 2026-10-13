# frozen_string_literal: true

# DARE v3.0 — LLMProvider interface
# Convention (ADR-04): all LLM calls go through this interface
# Implementations: OpenaiProvider, DummyProvider, LocalLlamaProvider
#
# Usage:
#   LLMProvider.instance.complete(model: "gpt-4o", prompt: "Summarize...", max_tokens: 150)

module LLM
  module Providers
    class LLMProvider
      include Singleton

      # ── Factory: configure at boot from dare.yml ─────────────────────────

      # Override the singleton instance (useful in tests to inject DummyProvider)
      def self.configure(provider_instance)
        @configured_instance = provider_instance
      end

      def self.instance
        @configured_instance ||= build_from_config
      end

      def self.build_from_config
        provider_name = Rails.configuration.dare.llm_provider

        case provider_name
        when "openai"
          LLM::Providers::OpenaiProvider.new
        when "dummy"
          LLM::Providers::DummyProvider.new
        else
          raise ArgumentError, "Unknown LLM provider: #{provider_name}. Valid: openai, dummy"
        end
      end

      # ── Interface — subclasses must implement ─────────────────────────────

      # Complete a prompt and return the response text.
      #
      # @param model      [String]  e.g. "gpt-4o"
      # @param prompt     [String]  the full prompt text
      # @param max_tokens [Integer] maximum tokens in response
      # @param temperature [Float]  0.0 to 1.0
      # @return [String] the completion text
      def complete(model:, prompt:, max_tokens: 1024, temperature: 0.7, **_opts)
        raise NotImplementedError, "#{self.class} must implement #complete"
      end

      # Chat-style completion with messages array
      #
      # @param model    [String]
      # @param messages [Array<Hash>] [{role: "user", content: "..."}]
      # @return [String] assistant message content
      def chat(model:, messages:, max_tokens: 1024, temperature: 0.7, **_opts)
        raise NotImplementedError, "#{self.class} must implement #chat"
      end

      # Stream a completion — yields chunks as they arrive.
      def stream(model:, prompt:, max_tokens: 1024, &block)
        raise NotImplementedError, "#{self.class} must implement #stream"
      end
    end
  end
end
