# frozen_string_literal: true

class User
  attr_reader :id
end

Rails.application.routes.draw do
  get '/users', to: 'users#index'
  post '/users', to: 'users#create'
end
