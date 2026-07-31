# Parity hardening (MP-054)

> **DEC-055** · Microplano 054 · Crate: `crates/dare-parity` · Suites: `tests/{golden,security,cross-platform}` · Perf: `docs/perf/baseline-054.md`

## Purpose

Hardening harness for **contract observability** (golden), **security regression**, and **engineering baselines** (startup / RSS / binary size). There is **no** new product CLI command and **no** capability-matrix bump (stays at **51**).

| Surface | Role |
|---------|------|
| `cargo test -p dare-parity` | Golden + security + cross-platform + unit/proptest |
| `scripts/measure-perf.*` | Release startup / RSS / size → `docs/perf/baseline-054.md` |
| [`parity-diff-log.md`](parity-diff-log.md) | Classified TS↔Rust diffs (`PD-*`) for Class C cases |

## How to run suites

From the repo root:

```bash
# Full parity crate (golden, security, xplat, normalize, proptest)
cargo test -p dare-parity

# Golden / security are integration tests under the crate + fixtures in tests/
# Optional: last golden JSON report (schemaVersion 1) when written by the runner
#   tests/golden/last-report.json
```

Security fixtures live under `tests/security/**` and **orchestrate** path/process/redact/archive/sig helpers from core crates — they do **not** reimplement extract logic.

Cross-platform smoke fixtures: `tests/cross-platform/**`.

## Normalizer allowlist (N-01..N-08)

Closed allowlist in `dare_parity::normalize`. Only these transforms are permitted:

| Id | Rule | Placeholder / effect |
|----|------|----------------------|
| N-01 | ISO-8601 timestamps | `1970-01-01T00:00:00Z` |
| N-02 | UUID v4 hex | `00000000-0000-4000-8000-000000000000` |
| N-03 | Temp prefixes (`TMPDIR` / `TEMP` / `CARGO_TARGET_DIR` / tempfile) | `$TMP/` |
| N-04 | Strip ANSI CSI (`\x1b\[…m`) | removed |
| N-05 | `\` → `/` in reported paths | separators unified |
| N-06 | Drive letter `C:` / `c:` | `$DRIVE:` |
| N-07 | Binary semver in banners | `$VERSION` |
| N-08 | Tokens matched by `dare_core::redact` | `$REDACTED` |

**Must not normalize:** exit codes, flag/command names, JSON contract keys, canonical capability IDs, ADR-002-stable array ordering. Over-normalize fails the suite (`normalize_anti_cheat`).

## Perf gate

| Const | Value |
|-------|-------|
| `PERF_REGRESSION_MAX` | `0.15` (15%) |
| Startup command | `dare --version` (release) |
| Samples | 5 runs; discard 1st cold; median of remaining 4 |

Gate per metric present in the committed baseline:

```text
measured <= baseline * (1 + PERF_REGRESSION_MAX)
```

Metrics: `startupMedianMs`, `rssPeakKiB`, `binarySizeBytes`. See [`../perf/baseline-054.md`](../perf/baseline-054.md).

Regenerate (writes front-matter; CI compare jobs must **not** rewrite):

```powershell
.\scripts\measure-perf.ps1
```

```bash
bash scripts/measure-perf.sh
```

## Diff log

Classified differences: [`parity-diff-log.md`](parity-diff-log.md) (`PD-001` …). Class **C** golden cases require `adr_ref` or a `PD-*` row.

## Distinction vs 055 / 056

| Microplano | Scope |
|------------|--------|
| **054** (this) | Harness + docs + perf gate — **no** RC, **no** npm cutover, **no** new capability |
| **055** | Pilotos, shadow tests, release candidate |
| **056** | Cutover, stable, encerramento do legado npm |

## Out of scope

- Docker / container CI for the harness
- Live npm baseline in PR CI (snapshots under `tests/golden/cases/**/expected.*` are SoT)
- Capability `dare-parity` or bump 51→52
- CLI binary `dare golden` (v1 uses `cargo test -p dare-parity` only)
