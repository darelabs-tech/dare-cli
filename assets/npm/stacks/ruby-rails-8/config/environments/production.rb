# frozen_string_literal: true

Rails.application.configure do
  # Settings here take precedence over config/application.rb.

  config.enable_reloading = false
  config.eager_load = true
  config.consider_all_requests_local = false
  config.action_controller.perform_caching = true

  # Log to STDOUT (12-factor) with request tagging.
  config.log_level = ENV.fetch("RAILS_LOG_LEVEL", "info")
  config.log_tags = [:request_id]
  config.logger = ActiveSupport::TaggedLogging.logger($stdout)

  config.i18n.fallbacks = true
  config.active_support.report_deprecations = false
  config.active_record.dump_schema_after_migration = false

  # TLS is terminated upstream (load balancer / proxy). Adjust if not.
  config.force_ssl = true
  config.assume_ssl = true

  # Solid Cache/Queue/Cable ship in the Gemfile but are not wired by default —
  # run their install generators when you opt in. Until then, in-process
  # defaults keep the app bootable with a single database.
  config.cache_store = :memory_store
  config.active_job.queue_adapter = :async
end
