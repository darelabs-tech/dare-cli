#!/usr/bin/env bash
# Measure dare-cli release startup / RSS / binary size for MP-054.
# CI gate (Fase F/G): measured <= baseline * (1 + PERF_REGRESSION_MAX)
# where PERF_REGRESSION_MAX=0.15 (fail if >15% above committed baseline).
# This script WRITES docs/perf/baseline-054.md; CI compare jobs must not rewrite.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BASELINE_PATH="$ROOT/docs/perf/baseline-054.md"
STARTUP_SAMPLES=5
# PERF_REGRESSION_MAX=0.15 — documented for consumers of this baseline

echo "Building dare-cli (release)..."
cargo build -p dare-cli --release

TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
HOST="$(rustc -vV | awk '/^host:/{print $2}')"
BIN="$TARGET_DIR/release/dare"
if [[ "$HOST" == *"windows"* ]]; then
  BIN="${BIN}.exe"
fi
if [[ ! -f "$BIN" ]]; then
  echo "binary missing: $BIN" >&2
  exit 1
fi

# Elapsed ms for one invocation (python for portable high-res timing)
run_once_ms() {
  python - "$BIN" <<'PY'
import subprocess, sys, time
bin_path = sys.argv[1]
t0 = time.perf_counter()
r = subprocess.run([bin_path, "--version"], check=False)
elapsed_ms = (time.perf_counter() - t0) * 1000.0
if r.returncode != 0:
    sys.exit(r.returncode or 1)
print(f"{elapsed_ms:.3f}")
PY
}

samples=()
i=0
while [[ $i -lt $STARTUP_SAMPLES ]]; do
  ms="$(run_once_ms)"
  samples+=("$ms")
  i=$((i + 1))
done

# Discard first (cold); median of remaining 4
warm=("${samples[@]:1}")
median_ms() {
  python - "$@" <<'PY'
import sys
vals = sorted(float(x) for x in sys.argv[1:])
n = len(vals)
if n == 0:
    raise SystemExit("no samples")
if n % 2 == 1:
    print(int(round(vals[n // 2])))
else:
    print(int(round((vals[n // 2 - 1] + vals[n // 2]) / 2.0)))
PY
}
STARTUP_MEDIAN_MS="$(median_ms "${warm[@]}")"

# RSS peak KiB after a startup sample (Unix: ps rss; Windows Git Bash: tasklist fallback via python)
measure_rss_kib() {
  python - "$BIN" <<'PY'
import os, subprocess, sys, time

bin_path = sys.argv[1]
proc = subprocess.Popen([bin_path, "--version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
rss_kib = 0
pid = proc.pid
try:
    while proc.poll() is None:
        try:
            if os.name == "nt":
                # WorkingSet in bytes via PowerShell
                out = subprocess.check_output(
                    [
                        "powershell",
                        "-NoProfile",
                        "-Command",
                        f"(Get-Process -Id {pid}).WorkingSet64",
                    ],
                    text=True,
                    stderr=subprocess.DEVNULL,
                ).strip()
                if out.isdigit():
                    rss_kib = max(rss_kib, int(out) // 1024)
            else:
                out = subprocess.check_output(["ps", "-o", "rss=", "-p", str(pid)], text=True).strip()
                if out:
                    rss_kib = max(rss_kib, int(out.split()[0]))
        except Exception:
            pass
        time.sleep(0.005)
    proc.wait()
    if rss_kib <= 0 and os.name != "nt":
        # Last-chance: ps on exited pid often fails; run once more and sample mid-flight
        proc2 = subprocess.Popen([bin_path, "--version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(0.01)
        try:
            out = subprocess.check_output(["ps", "-o", "rss=", "-p", str(proc2.pid)], text=True).strip()
            if out:
                rss_kib = int(out.split()[0])
        except Exception:
            pass
        proc2.wait()
finally:
    if proc.poll() is None:
        proc.kill()
print(rss_kib)
PY
}
RSS_PEAK_KIB="$(measure_rss_kib)"

BINARY_SIZE_BYTES="$(python -c 'import os,sys; print(os.path.getsize(sys.argv[1]))' "$BIN")"
if command -v sha256sum >/dev/null 2>&1; then
  BINARY_SHA256="$(sha256sum "$BIN" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  BINARY_SHA256="$(shasum -a 256 "$BIN" | awk '{print $1}')"
else
  BINARY_SHA256="$(python -c 'import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$BIN")"
fi

MEASURED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"

mkdir -p "$(dirname "$BASELINE_PATH")"
cat > "$BASELINE_PATH" <<EOF
---
schemaVersion: 1
targetTriple: "${HOST}"
startupMedianMs: ${STARTUP_MEDIAN_MS}
rssPeakKiB: ${RSS_PEAK_KIB}
binarySizeBytes: ${BINARY_SIZE_BYTES}
binarySha256: "${BINARY_SHA256}"
measuredAt: "${MEASURED_AT}"
gitSha: "${GIT_SHA}"
---

# Perf baseline — MP-054

Committed baseline for CI regression gate
(\`measured <= baseline * (1 + PERF_REGRESSION_MAX)\` with **PERF_REGRESSION_MAX=0.15**).

## How to regenerate

From the repo root:

\`\`\`powershell
.\scripts\measure-perf.ps1
\`\`\`

On Unix/macOS (CI):

\`\`\`bash
bash scripts/measure-perf.sh
\`\`\`

Both scripts build \`dare-cli\` release, run \`dare --version\` five times
(discard cold start; median of the remaining four), capture RSS, binary size,
and SHA-256, then rewrite the YAML front-matter above.

Humans commit the first baseline for each \`targetTriple\`; CI compare jobs
must **not** rewrite this file.
EOF

echo "Wrote $BASELINE_PATH"
echo "targetTriple=$HOST startupMedianMs=$STARTUP_MEDIAN_MS rssPeakKiB=$RSS_PEAK_KIB binarySizeBytes=$BINARY_SIZE_BYTES"
echo "binarySha256=$BINARY_SHA256"
echo "Note: PERF_REGRESSION_MAX=0.15 applies in CI gate vs this baseline."
