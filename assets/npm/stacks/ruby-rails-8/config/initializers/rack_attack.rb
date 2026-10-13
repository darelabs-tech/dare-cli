# frozen_string_literal: true

# DARE v3.0 — Rack::Attack configuration
# Satisfies metric M-04: 100% of public endpoints have rate limit configured
#
# D-006 (RFC 7807): throttle responses use Problem Details format

class Rack::Attack
  # ── Safelist ───────────────────────────────────────────────────────────────
  # Always allow localhost in development
  safelist("allow-localhost") do |req|
    req.ip == "127.0.0.1" || req.ip == "::1" if Rails.env.development?
  end

  # ── Public API throttle ───────────────────────────────────────────────────
  # M-04: All public endpoints limited to 100 req/min per IP
  throttle("public-api/ip", limit: 100, period: 1.minute) do |req|
    req.ip if req.path.start_with?("/api/")
  end

  # ── Auth endpoints throttle ───────────────────────────────────────────────
  # 10 req/min per IP for sensitive auth endpoints
  throttle("auth/ip", limit: 10, period: 1.minute) do |req|
    req.ip if req.path =~ %r{\A/api/(auth|sessions|passwords)}
  end

  # ── LLM endpoints throttle ───────────────────────────────────────────────
  # 20 req/min per authenticated user for LLM-heavy endpoints
  throttle("llm/user", limit: 20, period: 1.minute) do |req|
    req.env["HTTP_X_USER_ID"] if req.path.start_with?("/api/") && req.env["HTTP_X_USER_ID"]
  end

  # ── Response — RFC 7807 Problem Details ───────────────────────────────────
  self.throttled_responder = lambda do |env|
    match_data = env["rack.attack.match_data"]
    now        = match_data[:epoch_time]
    retry_at   = now + (match_data[:period] - now % match_data[:period])

    headers = {
      "Content-Type"  => "application/problem+json",
      "Retry-After"   => (retry_at - now).to_s,
      "X-RateLimit-Limit"     => match_data[:limit].to_s,
      "X-RateLimit-Remaining" => "0",
      "X-RateLimit-Reset"     => retry_at.to_s,
    }

    body = {
      type:     "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429",
      title:    "Too Many Requests",
      status:   429,
      detail:   "Rate limit exceeded. Please retry after #{retry_at - now} seconds.",
      instance: env["PATH_INFO"],
    }.to_json

    [429, headers, [body]]
  end
end

# Use Redis for distributed rate limiting (multi-server setups)
if defined?(Redis) && ENV["REDIS_URL"]
  Rack::Attack.cache.store = ActiveSupport::Cache::RedisCacheStore.new(
    url: ENV["REDIS_URL"]
  )
end
