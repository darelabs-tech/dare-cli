# frozen_string_literal: true

# DARE v3.0 — UserUpdatesChannel
# Convention (ADR-03): per-user channel with authorization check
# Frontend subscribes with: consumer.subscriptions.create({ channel: "UserUpdatesChannel", user_id: 123 })

class UserUpdatesChannel < ApplicationCable::Channel
  def subscribed
    user_id = params[:user_id].to_i

    # Authorization: only subscribe to your own updates (or admin)
    unless current_user.can_view?(user_id)
      reject_subscription
      return
    end

    stream_for user_id
  end

  def unsubscribed
    stop_all_streams
  end
end
