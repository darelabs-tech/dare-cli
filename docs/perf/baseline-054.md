---
schemaVersion: 1
targetTriple: ""
startupMedianMs: 0
rssPeakKiB: 0
binarySizeBytes: 0
binarySha256: ""
measuredAt: ""
gitSha: ""
---

# Perf baseline — MP-054

Committed baseline for CI regression gate
(`measured <= baseline * (1 + PERF_REGRESSION_MAX)` with **PERF_REGRESSION_MAX=0.15**).

Values above start as placeholders. After the first successful local/CI measure for a
host triple, humans commit the filled front-matter; subsequent CI jobs **compare**
against it and must not rewrite this file.

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
