# frozen_string_literal: true

# DARE v3.0 — OpenAI LLM provider
# Uses Faraday for HTTP — no heavy SDK dependency

module LLM
  module Providers
    class OpenaiProvider < LLMProvider
      BASE_URL = "https://api.openai.com/v1"

      def initialize(api_key: nil)
        @api_key = api_key || Rails.configuration.dare.llm_api_key
        raise ArgumentError, "OpenAI API key not configured. Set OPENAI_API_KEY." if @api_key.blank?
      end

      def complete(model:, prompt:, max_tokens: 1024, temperature: 0.7, **_opts)
        messages = [{ role: "user", content: prompt }]
        chat(model: model, messages: messages, max_tokens: max_tokens, temperature: temperature)
      end

      def chat(model:, messages:, max_tokens: 1024, temperature: 0.7, **_opts)
        response = connection.post("/v1/chat/completions") do |req|
          req.headers["Authorization"] = "Bearer #{@api_key}"
          req.headers["Content-Type"]  = "application/json"
          req.body = {
            model:       model,
            messages:    messages,
            max_tokens:  max_tokens,
            temperature: temperature
          }.to_json
        end

        handle_response(response)
          .dig("choices", 0, "message", "content")
          .to_s
          .strip
      end

      def stream(model:, prompt:, max_tokens: 1024, &block)
        # Streaming via SSE — basic implementation
        # For production use consider openai-ruby gem with native streaming
        raise NotImplementedError, "Streaming not yet implemented in OpenaiProvider. Use chat() instead."
      end

      private

      def connection
        @connection ||= Faraday.new(url: BASE_URL) do |f|
          f.request  :retry, max: 2, interval: 0.5, retry_statuses: [429, 503]
          f.response :raise_error
          f.adapter  Faraday.default_adapter
        end
      end

      def handle_response(response)
        JSON.parse(response.body)
      rescue JSON::ParserError => e
        raise "OpenAI returned non-JSON response: #{response.body.truncate(200)}"
      end
    end
  end
end
