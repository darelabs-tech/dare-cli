# frozen_string_literal: true

# DARE — Zeitwerk inflections so the lib/llm layer autoloads under the `LLM`
# namespace with its acronym (otherwise `llm` → `Llm`, `llm_provider` →
# `LlmProvider`). Other classes (LlmCache, OpenaiProvider) use the default
# camelization and are intentionally not listed here.
Rails.autoloaders.main.inflector.inflect(
  "llm" => "LLM",
  "llm_provider" => "LLMProvider"
)
