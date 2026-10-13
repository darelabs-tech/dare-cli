# frozen_string_literal: true

# DARE v3.0 — Token Bucket rate limiter for LLM API calls
# Prevents exceeding OpenAI/provider rate limits (tokens per minute)
# Uses Redis for distributed state across multiple app instances
#
# Usage:
#   bucket = LLM::RateLimit::TokenBucket.new(user_id: current_user.id)
#   unless bucket.consume!(tokens: estimated_tokens)
#     raise RateLimitExceeded, "LLM token limit reached. Retry after #{bucket.retry_after}s"
#   end

module LLM
  module RateLimit
    class TokenBucket
      class RateLimitExceeded < StandardError
        attr_reader :retry_after

        def initialize(retry_after: 60)
          @retry_after = retry_after
          super("LLM token rate limit exceeded. Retry after #{retry_after} seconds.")
        end
      end

      # Default: 100K tokens per minute per user (well below OpenAI limits)
      DEFAULT_CAPACITY   = 100_000
      DEFAULT_REFILL_RPM = 100_000  # tokens refilled per minute
      WINDOW_SECONDS     = 60

      def initialize(user_id:, capacity: DEFAULT_CAPACITY)
        @user_id  = user_id
        @capacity = capacity
        @redis    = Redis.new(url: Rails.configuration.dare.redis_url)
      end

      # Attempt to consume tokens. Returns true if allowed, false if rate-limited.
      def consume!(tokens: 1)
        key       = redis_key
        now       = Time.now.to_f
        window_start = now - WINDOW_SECONDS

        # Redis pipeline: count tokens used in current window + add new entry
        used = @redis.multi do |pipeline|
          pipeline.zremrangebyscore(key, "-inf", window_start)
          pipeline.zadd(key, now, "#{now}:#{tokens}")
          pipeline.zrange(key, 0, -1)
          pipeline.expire(key, WINDOW_SECONDS * 2)
        end

        # Tally tokens from sorted set entries
        entries = used[2] || []
        total_used = entries.sum do |entry|
          entry.split(":").last.to_i
        end

        if total_used > @capacity
          # Roll back the zadd we just did
          @redis.zrem(key, "#{now}:#{tokens}")
          return false
        end

        true
      end

      def retry_after
        # Time until oldest entry expires
        oldest = @redis.zrange(redis_key, 0, 0, with_scores: true).first
        return 0 unless oldest

        oldest_time = oldest[1]
        remaining = (oldest_time + WINDOW_SECONDS) - Time.now.to_f
        [remaining.ceil, 0].max
      end

      private

      def redis_key
        "llm_token_bucket:#{@user_id}"
      end
    end
  end
end
