# frozen_string_literal: true

Rails.application.routes.draw do
  # Health check endpoint (Rails 8 default)
  get "up" => "rails/health#show", as: :rails_health_check

  # OpenAPI / Swagger UI (rswag) — documents any JSON/API endpoints
  mount Rswag::Ui::Engine => "/api-docs"
  mount Rswag::Api::Engine => "/api-docs"

  # Root — server-rendered MVC home page
  root "home#index"

  # Add resourceful routes below, e.g.:
  #   resources :users
end
