# frozen_string_literal: true

# DARE v3.0 — UserUpdatesChannel spec
# Convention (ADR-06): tests auth and subscription behavior
require "rails_helper"

RSpec.describe UserUpdatesChannel, type: :channel do
  let(:user) { create(:user) }

  describe "#subscribed" do
    context "when user is authenticated and subscribing to their own updates" do
      before do
        stub_connection current_user: user
      end

      it "subscribes successfully" do
        subscribe user_id: user.id

        expect(subscription).to be_confirmed
      end

      it "streams for the user" do
        subscribe user_id: user.id

        expect(streams).to include("user_updates:#{user.id}")
      end
    end

    context "when user tries to subscribe to another user's updates without permission" do
      let(:other_user) { create(:user) }

      before do
        stub_connection current_user: user
        allow(user).to receive(:can_view?).with(other_user.id).and_return(false)
      end

      it "rejects the subscription" do
        subscribe user_id: other_user.id

        expect(subscription).to be_rejected
      end
    end

    context "when user is not authenticated" do
      before do
        stub_connection current_user: nil
      end

      it "rejects the subscription" do
        subscribe user_id: user.id

        expect(subscription).to be_rejected
      end
    end
  end
end
