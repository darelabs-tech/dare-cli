# frozen_string_literal: true

require "ostruct" # Ruby 3.4+ no longer autoloads OpenStruct.

# DARE v3.0 — Central configuration initializer
# All DARE runtime settings are loaded from config/dare.yml
# Access via: Rails.configuration.dare

dare_config_path = Rails.root.join("config", "dare.yml")

if dare_config_path.exist?
  # aliases: true — dare.yml uses YAML anchors/aliases (disabled by default on Psych 4 / Ruby 3).
  raw = YAML.safe_load_file(dare_config_path, symbolize_names: false, aliases: true)
  env_config = raw.dig(Rails.env) || raw["default"] || {}

  Rails.configuration.dare = OpenStruct.new(
    llm_provider:   env_config.dig("llm", "provider") || "dummy",
    llm_model:      env_config.dig("llm", "model")    || "gpt-4o",
    llm_api_key:    ENV.fetch("OPENAI_API_KEY", nil),
    redis_url:      ENV.fetch("REDIS_URL", "redis://localhost:6379/0"),
    rate_limit_rpm: env_config.dig("rate_limit", "rpm") || 100,
    metrics_path:   Rails.root.join("tmp", "dare_metrics.json")
  )
else
  Rails.logger.warn "[DARE] config/dare.yml not found — using defaults"
  Rails.configuration.dare = OpenStruct.new(
    llm_provider:   "dummy",
    llm_model:      "gpt-4o",
    llm_api_key:    ENV.fetch("OPENAI_API_KEY", nil),
    redis_url:      ENV.fetch("REDIS_URL", "redis://localhost:6379/0"),
    rate_limit_rpm: 100,
    metrics_path:   Rails.root.join("tmp", "dare_metrics.json")
  )
end
