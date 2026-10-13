# frozen_string_literal: true

require_relative "boot"

# Full-stack MVC — load all frameworks (Action View, asset pipeline, etc.).
require "rails/all"

# Require the gems listed in Gemfile, including any gems limited to :test,
# :development, or :production.
Bundler.require(*Rails.groups)

module <%= app_module %>
  class Application < Rails::Application
    config.load_defaults 8.0

    # Full-stack MVC — server-rendered views, Propshaft asset pipeline and
    # Hotwire (Turbo + Stimulus) are enabled. NOT api_only.

    # ── DARE Layered Design (ADR-05) — autoload the extra app/ layers ────────
    config.autoload_paths << root.join("app/handlers")
    config.autoload_paths << root.join("app/services")
    config.autoload_paths << root.join("app/repositories")
    config.autoload_paths << root.join("app/presenters")
    config.autoload_lib(ignore: %w[tasks])
  end
end
