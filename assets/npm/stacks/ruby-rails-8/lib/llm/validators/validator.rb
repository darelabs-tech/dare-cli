# frozen_string_literal: true

# DARE v3.0 — LLM Output Validator
# Validates LLM responses against JSON Schema definitions
# Schemas live in lib/llm/validators/*.json
#
# Usage:
#   LLM::Validators::Validator.validate!("summarize_output", response_hash)

module LLM
  module Validators
    class Validator
      SCHEMAS_DIR = Rails.root.join("lib", "llm", "validators")

      class ValidationError < StandardError
        attr_reader :errors

        def initialize(schema_name, errors)
          @errors = errors
          super("LLM output failed validation for '#{schema_name}': #{errors.map { |e| e[:error] }.join(", ")}")
        end
      end

      # Validate a hash against a named JSON Schema.
      # Raises ValidationError if invalid.
      def self.validate!(schema_name, data)
        schema = load_schema!(schema_name)
        validator = JSONSchemer.schema(schema)
        errors = validator.validate(data).to_a

        raise ValidationError.new(schema_name, errors) if errors.any?

        true
      end

      # Returns true/false without raising
      def self.valid?(schema_name, data)
        validate!(schema_name, data)
        true
      rescue ValidationError
        false
      end

      private_class_method def self.load_schema!(name)
        path = SCHEMAS_DIR.join("#{name}.json")
        raise ArgumentError, "Schema not found: #{name}.json in #{SCHEMAS_DIR}" unless path.exist?

        JSON.parse(path.read)
      end
    end
  end
end
