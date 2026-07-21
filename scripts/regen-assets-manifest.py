#!/usr/bin/env python3
"""Regenerate sha256 fields in assets/manifest.yml from files on disk.

Usage (from repo root):
  python scripts/regen-assets-manifest.py

Preserves id, path, kind, and entry order. Skips kind == external.
Hex digests are lowercase. Run after editing files under assets/.
"""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML required: pip install pyyaml", file=sys.stderr)
    sys.exit(1)

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
MANIFEST = ASSETS / "manifest.yml"


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    raw = MANIFEST.read_text(encoding="utf-8")
    data = yaml.safe_load(raw)
    if not isinstance(data, dict) or data.get("version") != 1:
        print("unsupported or missing version:1", file=sys.stderr)
        return 1
    assets = data.get("assets") or []
    for entry in assets:
        kind = entry.get("kind", "")
        if kind == "external":
            continue
        rel = entry["path"]
        path = ASSETS / rel
        if not path.is_file():
            print(f"missing file: {path}", file=sys.stderr)
            return 1
        entry["sha256"] = sha256_hex(path.read_bytes())
    # Prefer block style; keep simple dump
    out = yaml.safe_dump(data, sort_keys=False, allow_unicode=True)
    MANIFEST.write_text(out, encoding="utf-8")
    print(f"updated {MANIFEST.relative_to(ROOT)} ({len(assets)} entries)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
