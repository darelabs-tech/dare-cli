# frozen_string_literal: true

# DARE v3.0 — RSpec rails_helper

ENV["RAILS_ENV"] ||= "test"
require_relative "../config/environment"
require "rspec/rails"

# Require support files
Dir[Rails.root.join("spec/support/**/*.rb")].each { |f| require f }

# Prevent database truncation if the environment is production
abort("The Rails environment is running in production mode!") if Rails.env.production?

RSpec.configure do |config|
  config.fixture_paths = ["#{::Rails.root}/spec/fixtures"]
  config.use_transactional_fixtures = true
  config.infer_spec_type_from_file_location!
  config.filter_rails_from_backtrace!

  # FactoryBot
  config.include FactoryBot::Syntax::Methods

  # Shoulda Matchers
  Shoulda::Matchers.configure do |shoulda|
    shoulda.integrate do |with|
      with.test_framework :rspec
      with.library        :rails
    end
  end
end
