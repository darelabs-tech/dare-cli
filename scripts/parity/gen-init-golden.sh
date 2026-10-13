#!/usr/bin/env sh
# Regenerates crates/dare-cli/tests/fixtures/golden/init-npm/ from the published
# npm @dewtech/dare-cli (3.18.1) inside a pinned Linux container. No network.
#
#   scripts/parity/gen-init-golden.sh [--only slug1,slug2] [--allow-version]
#
# Env overrides:
#   DARE_NPM_PKG       path to the installed @dewtech/dare-cli package
#                      (default: "$(npm root -g)/@dewtech/dare-cli")
#   DARE_PARITY_IMAGE  container image (default: pinned node:22-bookworm digest)
#   DARE_GOLDEN_OUT    output dir (default: crates/dare-cli/tests/fixtures/golden/init-npm)
set -eu

export MSYS_NO_PATHCONV=1 # Git Bash on Windows: do not mangle /container paths.

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
# Git Bash: hand Docker a Windows-style path.
if pwd -W >/dev/null 2>&1; then
  repo=$(cd "$repo" && pwd -W)
fi

pkg=${DARE_NPM_PKG:-"$(npm root -g)/@dewtech/dare-cli"}
image=${DARE_PARITY_IMAGE:-node:22-bookworm@sha256:0e5f906573693feaa1e21057ebdcfdb5bd5021f050b2dc7c9deceb629c7da2a8}
out=${DARE_GOLDEN_OUT:-"$repo/crates/dare-cli/tests/fixtures/golden/init-npm"}

if [ ! -f "$pkg/package.json" ]; then
  echo "error: @dewtech/dare-cli not found at $pkg (set DARE_NPM_PKG)" >&2
  exit 2
fi
mkdir -p "$out"

user_flags=""
if [ "$(uname -s)" = "Linux" ]; then
  user_flags="--user $(id -u):$(id -g)"
fi

# shellcheck disable=SC2086
docker run --rm --network none $user_flags \
  -e HOME=/tmp/home -e DARE_PARITY_IMAGE="$image" \
  -v "$pkg:/opt/dare/node_modules/@dewtech/dare-cli:ro" \
  -v "$repo/scripts/parity:/parity:ro" \
  -v "$out:/out" \
  "$image" \
  node /parity/gen-init-golden.mjs \
  --pkg /opt/dare/node_modules/@dewtech/dare-cli \
  --out /out \
  --work /tmp/dare-init-golden \
  "$@"
