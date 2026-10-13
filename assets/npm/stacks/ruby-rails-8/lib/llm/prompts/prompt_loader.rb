# frozen_string_literal: true

# DARE v3.0 — PromptLoader
# Loads .txt or .jinja2 prompt templates from lib/llm/prompts/
# Performs simple variable interpolation
#
# Usage:
#   prompt = LLM::Prompts::PromptLoader.load("summarize", "v1", text: "Long document text...")
#   response = LLMProvider.instance.complete(model: "gpt-4o", prompt: prompt)

module LLM
  module Prompts
    class PromptLoader
      PROMPTS_DIR = Rails.root.join("lib", "llm", "prompts")

      # Load a prompt template and interpolate variables.
      #
      # @param name     [String] template name (e.g. "summarize")
      # @param version  [String] version suffix (e.g. "v1")
      # @param vars     [Hash]   variables to interpolate into the template
      # @return [String] rendered prompt
      def self.load(name, version = "v1", **vars)
        template = find_template!(name, version)
        interpolate(template, vars)
      end

      private_class_method def self.find_template!(name, version)
        # Search order: .jinja2 first, then .txt, then .erb
        candidates = [
          PROMPTS_DIR.join("#{name}_#{version}.jinja2"),
          PROMPTS_DIR.join("#{name}_#{version}.txt"),
          PROMPTS_DIR.join("#{name}.jinja2"),
          PROMPTS_DIR.join("#{name}.txt"),
        ]

        path = candidates.find(&:exist?)
        raise ArgumentError, "Prompt template not found: #{name}/#{version}. Searched:\n#{candidates.map(&:to_s).join("\n")}" unless path

        path.read
      end

      # Simple variable interpolation: {{ variable_name }} syntax (Jinja2-compatible)
      # Also supports <%= variable_name %> (ERB) and {{variable_name}} without spaces
      private_class_method def self.interpolate(template, vars)
        result = template.dup
        vars.each do |key, value|
          result.gsub!(/\{\{\s*#{Regexp.escape(key.to_s)}\s*\}\}/, value.to_s)
          result.gsub!(/<%=\s*#{Regexp.escape(key.to_s)}\s*%>/, value.to_s)
        end
        result
      end
    end
  end
end
