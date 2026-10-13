#!/usr/bin/env python3
"""Sync the npm `dare init` templates into assets/npm/ (cycle rust-init-npm-parity-011).

The Rust `dare init` renders exactly the same templates as the published npm
package `@dewtech/dare-cli`. This script copies the subtrees `dare init` reads
from the installed package (bytes as-is, LF as published) and records
provenance in assets/npm/PROVENANCE.json.

Usage (from repo root):
  python scripts/sync-npm-templates.py [--source <path to @dewtech/dare-cli>]

Default source: `npm root -g`/@dewtech/dare-cli.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "assets" / "npm"

# (source subtree relative to <pkg>/templates, destination relative to assets/npm)
SUBTREES = [
    ("stacks", "stacks"),
    ("ide/claude/CLAUDE.md", "ide/claude/CLAUDE.md"),
    ("ide/claude/.claude/commands", "ide/claude/.claude/commands"),
    ("ide/cursor/.cursor/commands", "ide/cursor/.cursor/commands"),
    ("ide/cursor/.cursor/rules", "ide/cursor/.cursor/rules"),
    ("ide/cursor/templates", "ide/cursor/templates"),
    ("ide/antigravity/.agents/skills", "ide/antigravity/.agents/skills"),
]


def default_source() -> Path:
    npm = shutil.which("npm") or shutil.which("npm.cmd")
    if not npm:
        raise SystemExit("npm not found; pass --source")
    out = subprocess.run([npm, "root", "-g"], capture_output=True, text=True, check=True)
    return Path(out.stdout.strip()) / "@dewtech" / "dare-cli"


def iter_files(base: Path):
    if base.is_file():
        yield base
        return
    for p in sorted(base.rglob("*")):
        if p.is_file():
            yield p


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", type=Path, default=None)
    args = ap.parse_args()
    pkg = args.source or default_source()
    pkg_json = pkg / "package.json"
    if not pkg_json.is_file():
        print(f"not an npm package: {pkg}", file=sys.stderr)
        return 1
    meta = json.loads(pkg_json.read_text(encoding="utf-8"))
    templates = pkg / "templates"

    if DEST.exists():
        shutil.rmtree(DEST)
    agg = hashlib.sha256()
    count = 0
    for src_rel, dst_rel in SUBTREES:
        src = templates / src_rel
        if not src.exists():
            print(f"missing in package: templates/{src_rel}", file=sys.stderr)
            return 1
        for f in iter_files(src):
            rel = Path(dst_rel) if src.is_file() else Path(dst_rel) / f.relative_to(src)
            out = DEST / rel
            out.parent.mkdir(parents=True, exist_ok=True)
            data = f.read_bytes()
            out.write_bytes(data)
            agg.update(rel.as_posix().encode() + b"\0" + hashlib.sha256(data).digest())
            count += 1

    provenance = {
        "package": meta.get("name"),
        "version": meta.get("version"),
        "files": count,
        "treeSha256": agg.hexdigest(),
        "subtrees": [s for s, _ in SUBTREES],
    }
    (DEST / "PROVENANCE.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")
    print(f"synced {count} files from {meta.get('name')}@{meta.get('version')} -> {DEST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
