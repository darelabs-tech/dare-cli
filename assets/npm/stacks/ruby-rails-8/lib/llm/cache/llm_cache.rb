# frozen_string_literal: true

# DARE v3.0 — LLM Cache (Redis wrapper)
# Caches LLM responses to reduce API costs and latency
# Uses SHA256 of (model + prompt) as cache key
#
# Usage:
#   result = LLM::Cache::LlmCache.instance.fetch(model: "gpt-4o", prompt: prompt) do
#     LLMProvider.instance.complete(model: "gpt-4o", prompt: prompt)
#   end

module LLM
  module Cache
    class LlmCache
      include Singleton

      DEFAULT_TTL = 24.hours

      def fetch(model:, prompt:, ttl: DEFAULT_TTL, &block)
        key = cache_key(model, prompt)

        Rails.cache.fetch(key, expires_in: ttl, &block)
      end

      def invalidate(model:, prompt:)
        key = cache_key(model, prompt)
        Rails.cache.delete(key)
      end

      def clear_all!
        # WARNING: clears entire Rails cache — use only in test/dev
        raise "clear_all! not allowed in production!" if Rails.env.production?
        Rails.cache.clear
      end

      private

      def cache_key(model, prompt)
        hash = Digest::SHA256.hexdigest("#{model}:#{prompt}")
        "llm_cache:#{hash}"
      end
    end
  end
end
