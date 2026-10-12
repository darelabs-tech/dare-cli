#!/usr/bin/env bash
set -euo pipefail
BIN="${1:?usage: smoke-dare.sh /path/to/dare}"
test -x "$BIN"
OUT_V="$("$BIN" --version)"
echo "$OUT_V" | grep -Eq '^dare [0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?(-[a-zA-Z0-9._-]+)?[[:space:]]*$'
OUT_H="$("$BIN" --help)"
echo "$OUT_H" | grep -Eq 'Usage:|--version'
