# frozen_string_literal: true

Rails.application.configure do
  # Settings here take precedence over config/application.rb.

  config.enable_reloading = false
  config.eager_load = ENV["CI"].present?

  config.public_file_server.enabled = true
  config.consider_all_requests_local = true
  config.action_controller.perform_caching = false
  config.cache_store = :null_store

  config.action_dispatch.show_exceptions = :rescuable
  config.action_controller.allow_forgery_protection = false

  config.active_support.deprecation = :stderr
  config.active_support.disallowed_deprecation = :raise
  config.active_support.disallowed_deprecation_warnings = []

  config.active_record.migration_error = :page_load
  config.i18n.raise_on_missing_translations = true
end
