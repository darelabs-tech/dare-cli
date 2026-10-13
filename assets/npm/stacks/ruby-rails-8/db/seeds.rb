# frozen_string_literal: true

# Seed data — run with: bin/rails db:seed (also runs on db:prepare for a fresh DB).
#
# Prefer Services for anything beyond trivial fixtures (DARE ADR-05):
#   Services::CreateUserService.new(...).execute(...)
#
# Example:
# User.find_or_create_by!(email: "admin@example.com") do |user|
#   user.name  = "Admin"
#   user.admin = true
# end
