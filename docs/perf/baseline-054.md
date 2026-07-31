---
schemaVersion: 1
targetTriple: "x86_64-pc-windows-gnu"
startupMedianMs: 13
rssPeakKiB: 7820
binarySizeBytes: 40310226
binarySha256: "5bf9169b9756cf32db47ae8482bd5c8846201368faa5a1e01e0ce929f3b88ef4"
measuredAt: "2026-07-31T07:14:50Z"
gitSha: "f6bd07004f258800b08a2a7bc111e0438d264cee"
---

# Perf baseline — MP-054

Committed baseline for CI regression gate
(`measured <= baseline * (1 + PERF_REGRESSION_MAX)` with **PERF_REGRESSION_MAX=0.15**).

Values above were filled from a local **release** build (`cargo build -p dare-cli --release`)
on `x86_64-pc-windows-gnu` via manual measure equivalent to `scripts/measure-perf.ps1`
(RSS sampling fixed for fast-exit processes). Humans commit the first baseline for each
`targetTriple`; CI compare jobs must **not** rewrite this file.

Gate rule: `measured <= baseline * (1 + PERF_REGRESSION_MAX)` per present metric
(`startupMedianMs`, `rssPeakKiB`, `binarySizeBytes`).

## How to regenerate

From the repo root (Windows):

```powershell
.\scripts\measure-perf.ps1
```

On Unix/macOS (CI):

```bash
bash scripts/measure-perf.sh
```

Both scripts:

1. `cargo build -p dare-cli --release`
2. Run `dare --version` five times; discard the first cold sample; take the median ms of the remaining four
3. Capture RSS (Unix: `ps`; Windows: `WorkingSet64`)
4. Record release binary size and SHA-256
5. Rewrite the YAML front-matter in this file
