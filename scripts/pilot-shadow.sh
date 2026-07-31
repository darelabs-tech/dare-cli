#!/usr/bin/env bash
# Shadow pilot runner (055): copy source to temp, fingerprint source, run allowlisted dare argv.
# Usage:
#   pilot-shadow.sh --pilot-id <id> --source <path> --dare-bin <path> [--cycle N] [--skip-commands]
# Exit codes: 0 ok · 2 usage · 3 path · 4 policy · 5 IO · 6 compare fail
set -euo pipefail

MSG_WRITE_ORIGINAL='shadow must not write to the original pilot tree'
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
  cat <<'EOF'
Usage:
  pilot-shadow.sh --pilot-id <id> --source <path> --dare-bin <path> [--cycle N] [--skip-commands]

Exit codes: 2 usage · 3 path · 4 policy · 5 IO · 6 compare fail
EOF
}

die() {
  local code="$1"
  shift
  echo "$*" >&2
  exit "$code"
}

is_allowlisted() {
  # "$@" = argv after dare-bin
  local joined="$*"
  case "$joined" in
    '--version'|'--help'|'welcome'|'info'|'discover'|'discover --check'|'validate'|'update --dry-run'|'self --help'|'mcp --help'|'capabilities')
      return 0
      ;;
  esac
  if [[ "${1:-}" == 'harness' ]]; then
    local a
    for a in "$@"; do
      if [[ "$a" == '--help' ]]; then
        return 0
      fi
    done
  fi
  return 1
}

sha256_file() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -- "$f" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -- "$f" | awk '{print $1}'
  else
    die 5 "IO: neither sha256sum nor shasum available"
  fi
}

list_rel_files() {
  local root="$1"
  # Portable: find relative paths, sorted
  (cd "$root" && find . -type f ! -path './.git/*' | sed 's|^\./||' | LC_ALL=C sort)
}

capture_fingerprints() {
  local root="$1"
  local min_count="${2:-3}"
  local -a files=()
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && files+=("$line")
  done < <(list_rel_files "$root")

  if ((${#files[@]} < min_count)); then
    die 3 "path: source must contain at least ${min_count} files for fingerprint (found ${#files[@]})"
  fi

  local take="$min_count"
  if ((${#files[@]} < 8)); then
    take=${#files[@]}
  else
    take=8
  fi
  if ((take < min_count)); then
    take=$min_count
  fi

  FP_RELS=()
  FP_HASHES=()
  local i
  for ((i = 0; i < take; i++)); do
    local rel="${files[$i]}"
    local abs="$root/$rel"
    local h
    h="$(sha256_file "$abs")" || die 5 "IO: failed to hash source file: $rel"
    FP_RELS+=("$rel")
    FP_HASHES+=("$h")
  done
}

assert_fingerprints() {
  local root="$1"
  local i
  for ((i = 0; i < ${#FP_RELS[@]}; i++)); do
    local rel="${FP_RELS[$i]}"
    local expect="${FP_HASHES[$i]}"
    local abs="$root/$rel"
    if [[ ! -f "$abs" ]]; then
      die 4 "$MSG_WRITE_ORIGINAL"
    fi
    local h
    h="$(sha256_file "$abs")" || die 5 "IO: failed to re-hash source file: $rel"
    if [[ "$h" != "$expect" ]]; then
      die 4 "$MSG_WRITE_ORIGINAL"
    fi
  done
}

redact() {
  local text="$1"
  local home="${HOME:-}"
  local tmp="${TMPDIR:-/tmp}"
  if [[ -n "$home" ]]; then
    text="${text//$home/\$HOME}"
  fi
  text="${text//$tmp/\$TMP}"
  # shellcheck disable=SC2001
  text="$(printf '%s' "$text" | sed -E 's/(api[_-]?key|token|secret|password)[[:space:]]*[:=][[:space:]]*[^[:space:]]+/\1=***/Ig')"
  printf '%s' "$text"
}

next_cycle() {
  local dir="$1"
  local n=1
  while [[ -f "$dir/cycle-$n.md" ]]; do
    n=$((n + 1))
  done
  printf '%s' "$n"
}

PILOT_ID=''
SOURCE=''
DARE_BIN=''
CYCLE_OPT=''
SKIP_COMMANDS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pilot-id)
      [[ $# -ge 2 ]] || die 2 'usage: --pilot-id requires a value'
      PILOT_ID="$2"; shift 2
      ;;
    --source)
      [[ $# -ge 2 ]] || die 2 'usage: --source requires a value'
      SOURCE="$2"; shift 2
      ;;
    --dare-bin)
      [[ $# -ge 2 ]] || die 2 'usage: --dare-bin requires a value'
      DARE_BIN="$2"; shift 2
      ;;
    --cycle)
      [[ $# -ge 2 ]] || die 2 'usage: --cycle requires a value'
      CYCLE_OPT="$2"; shift 2
      ;;
    --skip-commands)
      SKIP_COMMANDS=1; shift
      ;;
    --help|-h)
      usage; exit 0
      ;;
    *)
      usage
      die 2 "usage: unknown argument $1"
      ;;
  esac
done

[[ -n "$PILOT_ID" && -n "$SOURCE" && -n "$DARE_BIN" ]] || {
  usage
  die 2 'usage: --pilot-id, --source, and --dare-bin are required'
}

[[ "$PILOT_ID" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] || die 2 'usage: --pilot-id must match ^[a-z0-9]+(-[a-z0-9]+)*$'

SOURCE_FULL="$(cd "$(dirname -- "$SOURCE")" && pwd)/$(basename -- "$SOURCE")"
[[ -d "$SOURCE_FULL" ]] || die 3 "path: source is not a directory: $SOURCE"

if [[ "$DARE_BIN" = /* ]]; then
  DARE_BIN_FULL="$DARE_BIN"
else
  DARE_BIN_FULL="$(cd "$(dirname -- "$DARE_BIN")" && pwd)/$(basename -- "$DARE_BIN")"
fi
[[ -f "$DARE_BIN_FULL" && -x "$DARE_BIN_FULL" ]] || die 3 "path: dare-bin not found or not executable: $DARE_BIN"

if command -v uuidgen >/dev/null 2>&1; then
  UUID="$(uuidgen | tr '[:upper:]' '[:lower:]' | tr -d '-')"
elif [[ -r /proc/sys/kernel/random/uuid ]]; then
  UUID="$(tr -d '-' </proc/sys/kernel/random/uuid)"
else
  UUID="$(date +%s)-$$"
fi

TMP_BASE="${TMPDIR:-/tmp}"
SHADOW_ROOT="${TMP_BASE%/}/dare-pilot-${PILOT_ID}-${UUID}"

FP_RELS=()
FP_HASHES=()
capture_fingerprints "$SOURCE_FULL" 3

mkdir -p "$SHADOW_ROOT" || die 5 "IO: cannot create shadow root"
if command -v rsync >/dev/null 2>&1; then
  rsync -a -- "$SOURCE_FULL"/ "$SHADOW_ROOT"/ || die 5 'IO: rsync copy failed'
else
  cp -a -- "$SOURCE_FULL"/. "$SHADOW_ROOT"/ || die 5 'IO: cp copy failed'
fi

COMMAND_ROWS=()
VERDICT='pass'
COMPARE_FAILED=0

run_dare() {
  # argv-only: binary + "$@"; cwd = SHADOW_ROOT
  local -a cmd=("$DARE_BIN_FULL" "$@")
  local out_file err_file
  out_file="$(mktemp)"
  err_file="$(mktemp)"
  local ec=0
  (
    cd "$SHADOW_ROOT"
    "${cmd[@]}" >"$out_file" 2>"$err_file"
  ) || ec=$?
  local out_len err_len
  out_len="$(wc -c <"$out_file" | tr -d ' ')"
  err_len="$(wc -c <"$err_file" | tr -d ' ')"
  rm -f -- "$out_file" "$err_file"
  local argv_line="$*"
  local note
  note="$(redact "stdout_len=${out_len}; stderr_len=${err_len}")"
  COMMAND_ROWS+=("| \`dare ${argv_line}\` | ${ec} | ${note} |")
  if [[ "$ec" -ne 0 ]]; then
    COMPARE_FAILED=1
    VERDICT='fail'
  fi
}

if [[ "$SKIP_COMMANDS" -eq 0 ]]; then
  # Default allowlist smoke sets (each is a separate argv array)
  for spec in '--version' '--help' 'info'; do
    # split spec into words without eval
    # shellcheck disable=SC2206
    local_args=($spec)
    if ! is_allowlisted "${local_args[@]}"; then
      die 4 "policy: command not on allowlist: $spec"
    fi
    run_dare "${local_args[@]}"
  done
else
  COMMAND_ROWS+=('| _(skipped)_ | 0 | --skip-commands |')
fi

assert_fingerprints "$SOURCE_FULL"

RESULTS_DIR="$REPO_ROOT/docs/pilot/results/$PILOT_ID"
mkdir -p "$RESULTS_DIR" || die 5 "IO: cannot create results dir: $RESULTS_DIR"

if [[ -n "$CYCLE_OPT" ]]; then
  CYCLE="$CYCLE_OPT"
else
  CYCLE="$(next_cycle "$RESULTS_DIR")"
fi

REPORT_PATH="$RESULTS_DIR/cycle-${CYCLE}.md"
SHADOW_HINT="$(redact "$SHADOW_ROOT")"

FP_MD=''
for ((i = 0; i < ${#FP_RELS[@]}; i++)); do
  short="${FP_HASHES[$i]:0:12}"
  FP_MD+="- \`${FP_RELS[$i]}\`: \`${short}…\`"$'\n'
done

CMD_TABLE=''
for row in "${COMMAND_ROWS[@]}"; do
  CMD_TABLE+="${row}"$'\n'
done

cat >"$REPORT_PATH" <<EOF
# Shadow cycle ${CYCLE} — ${PILOT_ID}

| Field | Value |
|-------|-------|
| pilot_id | \`${PILOT_ID}\` |
| cycle | ${CYCLE} |
| shadow_root | \`${SHADOW_HINT}\` (redacted) |
| source_integrity | \`pass\` |
| verdict | \`${VERDICT}\` |

## Commands

| argv | exit | notes |
|------|------|-------|
${CMD_TABLE}
## Source fingerprint sample (≥3)

${FP_MD}
## Notes

- Copy-only shadow; original source verified unchanged (\`MSG_WRITE_ORIGINAL\` gate).
- Allowlist argv only; no shell string concatenation.
EOF

[[ -f "$REPORT_PATH" ]] || die 5 "IO: failed to write report: $REPORT_PATH"

echo "shadow OK: pilot=${PILOT_ID} cycle=${CYCLE} shadow=${SHADOW_ROOT} report=${REPORT_PATH} integrity=pass"

if [[ "$COMPARE_FAILED" -eq 1 ]]; then
  exit 6
fi
exit 0
