# frozen_string_literal: true

# Homebrew Formula for the DARE CLI (native binary).
# Microplano 056 / ADR-008 — filled from GitHub Release v4.0.0 SHA256SUMS.
#
# Asset: dare-v4.0.0-aarch64-apple-darwin.tar.gz (Apple Silicon).
# GAP: x86_64-apple-darwin was not published on Release v4.0.0 (Intel Mac).
#      Owner: Tech Lead — track in docs/migration/stable-smoke/README.md.
#
# How to refresh url / sha256 from a future GitHub Release:
#   1. Pick TAG = git tag (e.g. v4.0.1).
#   2. Prefer TARGET aarch64-apple-darwin; x86_64-apple-darwin when available.
#   3. Asset name: dare-${TAG}-${TARGET}.tar.gz
#   4. url / sha256 from Release download URL + SHA256SUMS line for that asset.

class Dare < Formula
  desc "DARE CLI — Design, Architect, Review, Execute methodology toolkit"
  homepage "https://github.com/darelabs-tech/dare-cli"

  url "https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-aarch64-apple-darwin.tar.gz"
  sha256 "831878c53819e9de7ef31358940207eb75e41e8feaef7dc8b04f2a0083d3578c"

  license "MIT"

  def install
    bin.install "dare"
  end

  test do
    assert_match(/dare/i, shell_output("#{bin}/dare --version"))
  end
end
