#!/usr/bin/env bash
# Local smoke: package current host binary + SHA256SUMS + minimal SBOM + install dry path.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
VERSION="${DARE_SMOKE_VERSION:-v0.1.0-alpha.smoke}"
OUT="$ROOT/dist/smoke"
rm -rf "$OUT"
mkdir -p "$OUT"

cargo build -p dare-cli --release
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
HOST="$(rustc -vV | awk '/^host:/{print $2}')"
STAGE="dare-${VERSION}-${HOST}"
mkdir -p "$OUT/$STAGE"
BIN="$TARGET_DIR/release/dare"
EXT=""
if [[ "$HOST" == *"windows"* ]]; then
  BIN="${BIN}.exe"
  EXT=".exe"
fi
cp "$BIN" "$OUT/$STAGE/dare${EXT}"

if [[ "$HOST" == *"windows"* ]]; then
  ARTIFACT="${STAGE}.zip"
  (cd "$OUT" && zip -r "$ARTIFACT" "$STAGE")
else
  ARTIFACT="${STAGE}.tar.gz"
  (cd "$OUT" && tar -czf "$ARTIFACT" "$STAGE")
fi

(cd "$OUT" && sha256sum "$ARTIFACT" > SHA256SUMS)
CREATED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '%s\n' "{\"spdxVersion\":\"SPDX-2.3\",\"dataLicense\":\"CC0-1.0\",\"SPDXID\":\"SPDXRef-DOCUMENT\",\"name\":\"dare-cli-${VERSION}\",\"documentNamespace\":\"https://local/spdx/${VERSION}\",\"creationInfo\":{\"created\":\"${CREATED}\",\"creators\":[\"Tool: dare-smoke\"]},\"packages\":[{\"name\":\"dare\",\"SPDXID\":\"SPDXRef-Package-dare\",\"downloadLocation\":\"NOASSERTION\",\"filesAnalyzed\":false,\"versionInfo\":\"${VERSION}\"}]}" > "$OUT/SBOM.spdx.json"
echo "signing skipped — local smoke" > "$OUT/SHA256SUMS.sig"
cp "$ROOT/installers/install.sh" "$OUT/" 2>/dev/null || true
cp "$ROOT/installers/install.ps1" "$OUT/" 2>/dev/null || true

PREFIX="$OUT/prefix"
mkdir -p "$PREFIX/bin"
export DARE_LOCAL_ARCHIVE="$OUT/$ARTIFACT"
export DARE_PREFIX="$PREFIX"
if [[ "$HOST" == *"windows"* ]]; then
  powershell -NoProfile -File "$ROOT/installers/install.ps1"
  VER_OUT="$("$PREFIX/bin/dare.exe" --version)"
else
  bash "$ROOT/installers/install.sh"
  VER_OUT="$("$PREFIX/bin/dare" --version)"
fi
echo "$VER_OUT" | grep -Eq '^dare '

# Confirm five-target matrix in release.yml
for t in \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu \
  x86_64-apple-darwin \
  aarch64-apple-darwin \
  x86_64-pc-windows-msvc
do
  grep -q "$t" "$ROOT/.github/workflows/release.yml"
done
grep -q 'macos-13' "$ROOT/.github/workflows/release.yml"
grep -q 'macos-14' "$ROOT/.github/workflows/release.yml"

echo "smoke-install OK: $ARTIFACT"
test -f "$OUT/SHA256SUMS"
test -f "$OUT/SBOM.spdx.json"
echo "five-target matrix OK in release.yml"
