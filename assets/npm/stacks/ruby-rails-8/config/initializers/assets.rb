# frozen_string_literal: true

# Version of your assets, change this if you want to expire all your assets.
Rails.application.config.assets.version = "1.0"

# Propshaft serves everything under app/assets by default. Add extra load
# paths here if you keep assets elsewhere.
Rails.application.config.assets.paths << Rails.root.join("app/javascript")
