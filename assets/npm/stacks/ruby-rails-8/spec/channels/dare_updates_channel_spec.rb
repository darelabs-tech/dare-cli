# frozen_string_literal: true

# DARE v3.0 — DareUpdatesChannel spec
# Convention (ADR-06): tests auth and subscription behavior
# Verifies that document.summarized events arrive on the correct channel
require "rails_helper"

RSpec.describe DareUpdatesChannel, type: :channel do
  let(:user) { create(:user) }

  describe "#subscribed" do
    context "when user is authenticated" do
      before do
        stub_connection current_user: user
      end

      it "subscribes successfully" do
        subscribe

        expect(subscription).to be_confirmed
      end

      it "streams from the user-scoped dare_updates channel" do
        subscribe

        expect(streams).to include("dare_updates:#{user.id}")
      end
    end

    context "when user is not authenticated" do
      before do
        stub_connection current_user: nil
      end

      it "rejects the subscription" do
        subscribe

        expect(subscription).to be_rejected
      end
    end
  end

  describe "document.summarized event delivery" do
    before do
      stub_connection current_user: user
    end

    it "delivers document.summarized payload to subscribed user" do
      subscribe

      expect {
        ActionCable.server.broadcast(
          "dare_updates:#{user.id}",
          { type: "document.summarized", data: { document_id: "doc-1", summary_length: 120 } }
        )
      }.to have_broadcasted_to("dare_updates:#{user.id}").with(
        hash_including(type: "document.summarized")
      )
    end
  end
end
