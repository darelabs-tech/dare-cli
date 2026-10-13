# frozen_string_literal: true

# DARE v3.0 — RealtimeService
# Convention (ADR-03): centralized broadcaster — services call this, not channels directly
# Singleton via .instance; injectable in tests via double
#
# Usage in a Service:
#   RealtimeService.instance.publish(
#     type: "user.created",
#     data: { user_id: 42, email: "user@example.com" }
#   )
#
# Frontend receives:
#   { type: "user.created", data: { ... }, timestamp: 1234567890 }

class RealtimeService
  include Singleton

  # ── Public API ────────────────────────────────────────────────────────────

  # Broadcast to all subscribers of a channel stream.
  # channel_class: e.g. UserUpdatesChannel
  # target:        e.g. user.id  (matched against stream_for in the channel)
  # payload:       Hash with :type and :data
  def broadcast_to(channel_class, target, type:, data: {})
    event = build_event(type: type, data: data)
    channel_class.broadcast_to(target, event)
  end

  # Generic publish: broadcasts to `dare_updates:<user_id>` for all active users.
  # Use for system-wide events.
  def publish(type:, data: {}, user_id: nil)
    event   = build_event(type: type, data: data)
    stream  = user_id ? "dare_updates:#{user_id}" : "dare_updates:global"
    ActionCable.server.broadcast(stream, event)
  end

  # Publish to a named stream directly (lower-level).
  def broadcast_to_stream(stream_name, type:, data: {})
    event = build_event(type: type, data: data)
    ActionCable.server.broadcast(stream_name, event)
  end

  private

  def build_event(type:, data: {})
    {
      type:      type,
      data:      data,
      timestamp: Time.now.to_i
    }
  end
end
