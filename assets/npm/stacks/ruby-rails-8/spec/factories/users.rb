# frozen_string_literal: true

# DARE v3.0 — FactoryBot factories
# Convention: no fixtures YAML; FactoryBot only

FactoryBot.define do
  factory :user do
    sequence(:email) { |n| "user#{n}@example.com" }
    name             { Faker::Name.name }
    active           { true }

    # Trait: admin user
    trait :admin do
      admin { true }
    end

    # Trait: inactive user
    trait :inactive do
      active { false }
    end

    # Trait: with all optional fields
    trait :complete do
      last_active_at { 1.day.ago }
    end
  end
end
