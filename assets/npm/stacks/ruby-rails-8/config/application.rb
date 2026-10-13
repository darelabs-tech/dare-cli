# frozen_string_literal: true

require_relative "boot"

require "rails"
# API-only — pick only the frameworks needed (no Action View / asset pipeline).
require "active_model/railtie"
require "active_job/railtie"
require "active_record/railtie"
require "action_controller/railtie"
require "action_mailer/railtie"
require "action_cable/engine"
require "rails/test_unit/railtie"

# Require the gems listed in Gemfile, including any gems limited to :test,
# :development, or :production.
Bundler.require(*Rails.groups)

module <%= app_module %>
  class Application < Rails::Application
    config.load_defaults 8.0

    # API-only: no cookies / sessions / views / asset pipeline by default.
    config.api_only = true

    # ── DARE Layered Design (ADR-05) — autoload the extra app/ layers ────────
    config.autoload_paths << root.join("app/handlers")
    config.autoload_paths << root.join("app/services")
    config.autoload_paths << root.join("app/repositories")
    config.autoload_paths << root.join("app/presenters")
    config.autoload_lib(ignore: %w[tasks])

    # Don't generate system test files.
    config.generators.system_tests = nil
  end
end
