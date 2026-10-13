# frozen_string_literal: true

# DARE v3.0 — CreateUserService spec
# Convention (ADR-06): unit tests with injected doubles — no database, no network
require "rails_helper"

RSpec.describe CreateUserService do
  subject(:service) do
    described_class.new(
      user_repository: user_repository,
      event_publisher: event_publisher
    )
  end

  let(:user_repository) { instance_double(UserRepository) }
  let(:event_publisher) { instance_double(RealtimeService) }
  let(:created_user)    { instance_double(User, id: 1, email: "alice@example.com", name: "Alice") }

  before do
    allow(event_publisher).to receive(:publish)
  end

  describe "#execute" do
    context "with valid params and no existing user" do
      before do
        allow(user_repository).to receive(:find_by_email).with("alice@example.com").and_return(nil)
        allow(user_repository).to receive(:build).and_return(created_user)
        allow(created_user).to receive(:password=)
        allow(user_repository).to receive(:save!).with(created_user).and_return(created_user)
      end

      it "returns the created user" do
        result = service.execute(email: "alice@example.com", name: "Alice")
        expect(result).to eq created_user
      end

      it "publishes a user.created event" do
        service.execute(email: "alice@example.com", name: "Alice")
        expect(event_publisher).to have_received(:publish).with(
          type: "user.created",
          data: hash_including(user_id: 1, email: "alice@example.com")
        )
      end

      it "normalizes email to lowercase" do
        allow(user_repository).to receive(:find_by_email).with("alice@example.com").and_return(nil)
        allow(user_repository).to receive(:build).with(hash_including(email: "alice@example.com")).and_return(created_user)
        allow(user_repository).to receive(:save!).and_return(created_user)

        service.execute(email: "ALICE@EXAMPLE.COM", name: "Alice")

        expect(user_repository).to have_received(:build).with(hash_including(email: "alice@example.com"))
      end
    end

    context "when email is already taken" do
      before do
        allow(user_repository).to receive(:find_by_email).with("alice@example.com").and_return(created_user)
      end

      it "raises EmailTakenError" do
        expect {
          service.execute(email: "alice@example.com", name: "Alice")
        }.to raise_error(CreateUserService::EmailTakenError, /already registered/)
      end

      it "does not publish any event" do
        service.execute(email: "alice@example.com", name: "Alice") rescue nil
        expect(event_publisher).not_to have_received(:publish)
      end
    end

    context "when repository save fails" do
      before do
        allow(user_repository).to receive(:find_by_email).and_return(nil)
        allow(user_repository).to receive(:build).and_return(created_user)
        allow(created_user).to receive(:password=)
        allow(user_repository).to receive(:save!).and_raise(ActiveRecord::RecordInvalid)
      end

      it "propagates the error" do
        expect {
          service.execute(email: "alice@example.com", name: "Alice")
        }.to raise_error(ActiveRecord::RecordInvalid)
      end
    end
  end
end
