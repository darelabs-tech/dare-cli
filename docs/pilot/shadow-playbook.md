# Shadow playbook (055)

> Isolated pilot runs: **copy-only** into a disposable temp tree. Never mutate the original pilot source. Scripts: `scripts/pilot-shadow.ps1` (Windows) and `scripts/pilot-shadow.sh` (Unix).

## Anti-write policy (MUST)

1. All CLI work happens under `SHADOW_ROOT` (`$TMP/dare-pilot-<pilot_id>-<uuid>`).
2. The original `--source` tree is **read-only** for the operator: no edits, installs, or DARE writes into it during shadow.
3. After the cycle, scripts re-hash a fingerprint sample on the **source**. Any mismatch exits **4** with:

   `shadow must not write to the original pilot tree`

   (`MSG_WRITE_ORIGINAL` — Blueprint-055).
4. Do not `cd` into the source and run mutating commands. Prefer the scripts; if running manually, copy first, then operate only in the copy.

## Fingerprint (integrity spot-check)

1. Before copy/run, collect **≥3 regular files** under `--source` (recursive, stable sort).
2. Store `(rel_path, sha256)` for each.
3. After allowlist commands finish, recompute hashes on the **same source paths**.
4. Mismatch → policy failure (exit 4 + `MSG_WRITE_ORIGINAL`).
5. Sources with fewer than three files cannot satisfy the spot-check — use a denser fixture (e.g. `tests/fixtures/monorepo`) or materialise stubs before shadowing.

## Allowlist (argv only)

Commands must be spawned as argv arrays against `--dare-bin` (no `Invoke-Expression`, no `bash -c "…"`, no string-concatenated shells).

Permitted after the binary (Blueprint-055 `ALLOWLIST_CMDS`):

| Argv (after dare) | Notes |
|-------------------|--------|
| `--version` | top-level |
| `--help` | top-level |
| `welcome` | |
| `info` | |
| `discover` | |
| `discover --check` | |
| `validate` | |
| `update --dry-run` | |
| `self --help` | |
| `mcp --help` | |
| `capabilities` | |
| `harness … --help` | any `harness` argv that includes `--help` |

Anything else → exit **4** (policy). Working directory for each command is `SHADOW_ROOT`.

## Three-cycle expectation

Close gate for 055 requires **≥3** documented cycles per pilot (`MIN_SHADOW_CYCLES`):

```text
docs/pilot/results/<pilot_id>/cycle-1.md
docs/pilot/results/<pilot_id>/cycle-2.md
docs/pilot/results/<pilot_id>/cycle-3.md
```

Each report is redacted (no secrets, tokens, or absolute home paths when avoidable). Schema fields: `pilot_id`, `cycle`, `commands`, `source_integrity`, `verdict`.

Update `shadow_cycles_done` in `docs/pilot/pilots.md` after successful cycles (mp055-003).

## How to run

### Windows (PowerShell 7+)

```powershell
pwsh -File scripts/pilot-shadow.ps1 `
  --pilot-id pilot-linux-empty `
  --source tests/fixtures/monorepo `
  --dare-bin path\to\dare.exe
```

Optional: `--cycle 1` (default: next free `cycle-N.md`), `--skip-commands` (copy + fingerprint only).

### Unix (bash)

```bash
bash scripts/pilot-shadow.sh \
  --pilot-id pilot-linux-empty \
  --source tests/fixtures/monorepo \
  --dare-bin /path/to/dare
```

Same optional flags as Windows.

### Stub binary (smoke without a real `dare`)

Point `--dare-bin` at a no-op executable that exits 0, or pass `--skip-commands` so only copy + fingerprint run. Fingerprint and copy **must** still execute for real.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Usage / bad args |
| 3 | Path missing / not a directory / too few files for fingerprint |
| 4 | Policy (write to original / command outside allowlist) |
| 5 | IO (copy, report write, hash read) |
| 6 | Compare / verify fail (command failure classified as compare) |

## Manual checklist (if scripts unavailable)

- [ ] Create temp dir `dare-pilot-<id>-<uuid>` under OS temp
- [ ] Copy entire source tree into it (copy-only)
- [ ] Record ≥3 source file SHA-256 digests
- [ ] Run only allowlisted `dare` argv with cwd = shadow root
- [ ] Re-check source fingerprints; abort on drift
- [ ] Write redacted `docs/pilot/results/<id>/cycle-<n>.md`
- [ ] Remove or leave shadow root under temp (never commit `$TMP` trees)

## Security notes

- Redact `$HOME` / `%USERPROFILE%` / tokens in cycle reports.
- Do not commit secrets, real pilot PII, or `target*` build artefacts from shadow runs.
- Consent for real pilots is recorded in `docs/pilot/pilots.md` (`consent: true`).
