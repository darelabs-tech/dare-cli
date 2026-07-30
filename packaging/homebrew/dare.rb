# frozen_string_literal: true

# Homebrew Formula template for the DARE CLI (native binary).
# Microplano 053 / ADR-008 — placeholders must be filled per release before publishing a tap.
#
# How to fill url / sha256 from a GitHub Release (ADR-008):
#   1. Pick TAG = git tag that named the Release (e.g. v0.1.0-alpha.1).
#   2. Pick TARGET for this bottle host, typically:
#        aarch64-apple-darwin  (Apple Silicon)
#        x86_64-apple-darwin   (Intel Mac)
#   3. Asset name: dare-${TAG}-${TARGET}.tar.gz
#      Example: dare-v0.1.0-alpha.1-aarch64-apple-darwin.tar.gz
#   4. url = https://github.com/dewtech/dare-cli/releases/download/${TAG}/${ASSET}
#   5. sha256 = hex digest of that asset from Release SHA256SUMS
#      (sha256sum two-space format: "<digest>  <filename>")
#
# PLACEHOLDERS below: replace REPLACE_ME_* before shipping. Do not publish with placeholders.

class Dare < Formula
  desc "DARE CLI — Design, Architect, Review, Execute methodology toolkit"
  homepage "https://github.com/dewtech/dare-cli"

  # PLACEHOLDER url — ADR-008 asset dare-${TAG}-${TARGET}.tar.gz
  url "REPLACE_ME_URL"
  # PLACEHOLDER sha256 — copy from Release SHA256SUMS for the same asset
  sha256 "REPLACE_ME_SHA256"

  license "MIT"

  def install
    bin.install "dare"
  end

  test do
    assert_match(/dare/i, shell_output("#{bin}/dare --version"))
  end
end