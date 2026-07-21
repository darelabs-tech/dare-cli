#!/usr/bin/env bash
# DARE native installer (alpha) — microplano 015
set -euo pipefail

REPO="${DARE_REPO:-dewtech/dare-cli}"
BASE_URL="${DARE_INSTALL_BASE:-https://github.com/${REPO}/releases/latest/download}"
PREFIX="${DARE_PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"
VERSION="${DARE_VERSION:-}"

detect_target() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$os" in
    linux) os=unknown-linux-gnu ;;
    darwin) os=apple-darwin ;;
    *) echo "unsupported OS: $os" >&2; exit 1 ;;
  esac
  case "$arch" in
    x86_64|amd64) arch=x86_64 ;;
    aarch64|arm64) arch=aarch64 ;;
    *) echo "unsupported arch: $arch" >&2; exit 1 ;;
  esac
  echo "${arch}-${os}"
}

main() {
  local target archive url tmp sums
  target="$(detect_target)"
  if [[ -n "$VERSION" ]]; then
    archive="dare-${VERSION}-${target}.tar.gz"
    url="${BASE_URL}/${archive}"
  else
    # latest release asset naming still needs version in filename — require VERSION or local file
    if [[ -n "${DARE_LOCAL_ARCHIVE:-}" ]]; then
      archive="$(basename "$DARE_LOCAL_ARCHIVE")"
      url="file://${DARE_LOCAL_ARCHIVE}"
    else
      echo "Set DARE_VERSION=vX.Y.Z-alpha.N or DARE_LOCAL_ARCHIVE=/path/to/archive.tar.gz" >&2
      exit 2
    fi
  fi

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT

  if [[ "$url" == file://* ]]; then
    cp "${url#file://}" "$tmp/$archive"
  else
    curl -fsSL "$url" -o "$tmp/$archive"
    sums_url="${BASE_URL}/SHA256SUMS"
    if curl -fsSL "$sums_url" -o "$tmp/SHA256SUMS"; then
      (cd "$tmp" && sha256sum -c SHA256SUMS --ignore-missing)
    fi
  fi

  mkdir -p "$BIN_DIR"
  tar -xzf "$tmp/$archive" -C "$tmp"
  local bin
  bin="$(find "$tmp" -type f -name dare | head -n1)"
  install -m 755 "$bin" "$BIN_DIR/dare"
  echo "Installed: $BIN_DIR/dare"
  "$BIN_DIR/dare" --version || "$BIN_DIR/dare" version || true
}

main "$@"
