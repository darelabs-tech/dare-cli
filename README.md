 DARE CLI

Native **Rust** CLI for the DARE methodology (Design → Architecture → Review → Execute).

**Official product:** stable **`v4.0.0`**. Install guide: [`docs/migration/install-rust.md`](docs/migration/install-rust.md).

> **Legacy npm banner:** The TypeScript package **`@dewtech/dare-cli`** is **legacy** (last supported **`3.18.1`**). Do **not** use npm for new installs. Prefer the native binary — see [`docs/migration/install-rust.md`](docs/migration/install-rust.md). Policy: [`docs/migration/npm-legacy-policy.md`](docs/migration/npm-legacy-policy.md). Registry `npm deprecate` is currently **`blocked:credentials`** (operator must re-run when authenticated); see [`docs/migration/legacy-archive-checklist.md`](docs/migration/legacy-archive-checklist.md).

## Quick start

1. Download Release **`v4.0.0`**: https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0
2. Verify `SHA256SUMS` / follow installers under `installers/`
3. Or: `dare self update --channel stable --yes` (after a native binary is present)

Ou instale de forma rápida via terminal:

**macOS, Linux, and FreeBSD:**
```bash
curl -fsSL https://raw.githubusercontent.com/dewtech/dare-cli/main/installers/install | sh
```

**Windows PowerShell:**
```powershell
irm https://raw.githubusercontent.com/dewtech/dare-cli/main/installers/install.ps1 | iex
```

Node.js / npm is **not required** for the recommended path.

## Docs

| Doc | Purpose |
|-----|---------|
| [`docs/migration/install-rust.md`](docs/migration/install-rust.md) | Recommended install |
| [`docs/migration/RELEASE-NOTES-stable.md`](docs/migration/RELEASE-NOTES-stable.md) | Stable notes |
| [`docs/migration/npm-legacy-policy.md`](docs/migration/npm-legacy-policy.md) | npm legacy policy |
| [`docs/migration/final-compatibility-report.md`](docs/migration/final-compatibility-report.md) | Final compatibility report |
| [`docs/compatibility/README.md`](docs/compatibility/README.md) | Parity / baseline package |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribute to the Rust workspace |

## License

Apache-2.0 — see [`LICENSE`](LICENSE).
