# frozen_string_literal: true

# DARE v3.0 — DareUpdatesChannel
# Convention (ADR-03): subscriptions authorize before streaming
# This is a generic channel for DARE system events (metrics, deploys, etc.)

class DareUpdatesChannel < ApplicationCable::Channel
  def subscribed
    # Reject if not authenticated (connection already checks, but belt-and-suspenders)
    reject_subscription unless current_user

    stream_from "dare_updates:#{current_user.id}"
  end

  def unsubscribed
    stop_all_streams
  end
end
