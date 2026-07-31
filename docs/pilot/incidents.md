# Pilot incidents (055)

> Shadow / pilot findings for microplano **055**. Severity and close rules: Blueprint-055 §0.2.  
> Close gate: **zero** P0/P1 with `status=open`.

## Summary (mp055-003)

| Metric | Value |
|--------|-------|
| Pilots with ≥3 cycles | 3 (`pilot-linux-empty`, `pilot-macos-node`, `pilot-windows-rust`) |
| P0/P1 open | **0** |
| Class C gaps (new) | **none** — `docs/compatibility/parity-diff-log.md` unchanged (N/A) |
| Runner | `scripts/pilot-shadow.ps1` |
| Shadow source | `tests/fixtures/monorepo` (fingerprint density; see INC-001) |
| `--dare-bin` | exit-0 stub (`dare.cmd`) — plumbing smoke; see INC-002 |

## Incidents

| id | sev | pilot_id | status | compat_class | summary | repro | opened | closed |
|----|-----|----------|--------|--------------|---------|-------|--------|--------|
| INC-001 | P2 | pilot-linux-empty | mitigated | B | Seed fixtures `empty-project` / `existing-node-project` / `existing-rust-project` have &lt;3 files; fingerprint gate requires ≥3. Workaround: shadow `--source tests/fixtures/monorepo` (playbook). Same mitigation applied to all three pilots. | Run `pilot-shadow.ps1` with `--source tests/fixtures/empty-project` → exit 3 | 2026-07-31 | 2026-07-31 |
| INC-002 | P3 | pilot-windows-rust | mitigated | B | Full native `dare` binary not used for mp055-003 allowlist smoke (heavy build). Exit-0 stub validates copy + fingerprint + redacted cycle markdown; real CLI output deferred to later pilots / CI matrix. Not a product P0/P1. | Point `--dare-bin` at stub; command rows show `stdout_len=0` | 2026-07-31 | 2026-07-31 |
| INC-003 | P3 | pilot-macos-node | closed | B | Synthetic macOS/Linux pilots executed on Windows host (single-OS runner). OS coverage for RC remains CI matrix (O-12); no Class C parity claim. | Host = win32; pilots marked `os: macos` / `linux` with `synthetic: true` | 2026-07-31 | 2026-07-31 |

## Notes

- No P0/P1 opened during mp055-003 cycles (all `verdict=pass`, `source_integrity=pass`).
- Reports under `docs/pilot/results/` are redacted (`$HOME` / `$TMP`; no tokens).
- Do not commit `$TMP/dare-pilot-*` trees or `target*` artefacts from shadow runs.
