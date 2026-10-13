# frozen_string_literal: true

# DARE v3.0 — ApplicationCable::Channel base
# All channels inherit from this; subscription authorization happens here

module ApplicationCable
  class Channel < ActionCable::Channel::Base
    # Convenience: access current_user from connection in all channels
    delegate :current_user, to: :connection
  end
end
