# Changelog

All notable changes to the DARE CLI are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/).

## 4.0.0

### Added

- Stable native Rust CLI cutover tagged **`v4.0.0`** (major 4 after npm baseline `@dewtech/dare-cli@3.18.1`).
- Migration and support docs:
  - [`docs/migration/install-rust.md`](docs/migration/install-rust.md) — Rust-first install (Download / Homebrew / WinGet / `dare self update --channel stable`)
  - [`docs/migration/RELEASE-NOTES-stable.md`](docs/migration/RELEASE-NOTES-stable.md) — stable release notes (not RC)
  - [`docs/migration/npm-legacy-policy.md`](docs/migration/npm-legacy-policy.md) — npm line status `legacy`
  - [`docs/support/legacy-support-window.md`](docs/support/legacy-support-window.md) — security-only support window
  - [`docs/support/incident-response.md`](docs/support/incident-response.md) — P0–P3 + ack SLA

### Changed

- Recommended install path is the native Rust binary. **Node/npm is not required.**
- Default `dare self` channel remains **`beta`** (unchanged); stable is opt-in via `--channel stable`.

### Deprecated

- TypeScript / npm package **`@dewtech/dare-cli`** — status **`legacy`**. Last supported npm version **`3.18.1`**. Features rejected; security fixes only until the window in [`docs/support/legacy-support-window.md`](docs/support/legacy-support-window.md) / [`docs/migration/npm-legacy-policy.md`](docs/migration/npm-legacy-policy.md).

### Migrate

1. Install Rust CLI **`v4.0.0`**: [`docs/migration/install-rust.md`](docs/migration/install-rust.md)
2. Read stable notes: [`docs/migration/RELEASE-NOTES-stable.md`](docs/migration/RELEASE-NOTES-stable.md)
3. Legacy pointer: [`docs/migration/npm-legacy-policy.md`](docs/migration/npm-legacy-policy.md)
