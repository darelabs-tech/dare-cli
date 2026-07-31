# Pilot inventory (055)

> Seed inventory for microplano **055** (pilotos + shadow + RC). Synthetic fixtures from `tests/fixtures/` stand in until real brownfield owners are available. No secrets or PII.

## Selection criteria

A project (or fixture mirror) is eligible as a pilot when **all** of the following hold:

1. **Coverage role** — Together with the rest of the set, the inventory covers **Linux**, **macOS**, and **Windows** (O-01 / O-12).
2. **Source class** — Either a real brownfield tree with owner consent, **or** a materialised fixture from `docs/compatibility/fixtures-inventory.md` (054 inventory) referenced as `fixture:<id>`.
3. **Stack signal** — Detectable stack markers (`package.json`, `Cargo.toml`, etc.) **or** an intentional empty/greenfield case for `discover` / init paths.
4. **Consent** — `consent: true` recorded; owners listed without emails or personal data in-repo (RF-26).
5. **Allowlist flows** — At least one MUST flow whose `command[]` is a subset of the 055 allowlist (`welcome`, `info`, `discover`, `discover --check`, `validate`, `update --dry-run`, `self --help`, `mcp --help`, `capabilities`, `harness … --help`, `--version`, `--help`).
6. **Shadow-ready** — Source can be copied to a disposable shadow root; no requirement to mutate the original tree.
7. **Harness (SHOULD)** — Prefer ≥1 IDE harness when a real project is available; synthetic seeds may omit harness until replaced.

Synthetic seeds below satisfy criteria 1–6 and are **replaceable** by real pilots without changing the schema.

## Allowlist (reference)

Commands permitted in pilot MUST flows (Blueprint-055 `ALLOWLIST_CMDS`):

- `dare welcome`
- `dare info`
- `dare discover`
- `dare discover --check`
- `dare validate`
- `dare update --dry-run`
- `dare self --help`
- `dare mcp --help`
- `dare capabilities`
- `dare harness --help` (and other `harness … --help`)
- `dare --version`
- `dare --help`

---

## Pilots

### pilot-linux-empty

```yaml
pilot_id: pilot-linux-empty
synthetic: true
stack: empty
os: linux
owner: DARE Labs
source: fixture:empty-project
consent: true
shadow_cycles_done: 3
flows:
  - id: smoke-help
    must: true
    command: ["dare", "--help"]
  - id: version
    must: true
    command: ["dare", "--version"]
  - id: welcome
    must: true
    command: ["dare", "welcome"]
  - id: info
    must: true
    command: ["dare", "info"]
  - id: discover-empty
    must: true
    command: ["dare", "discover"]
```

Fixture path: `tests/fixtures/empty-project/` (greenfield stub).  
Shadow source (mp055-003): `tests/fixtures/monorepo/` (≥3 files for fingerprint; see `incidents.md` INC-001).

---

### pilot-macos-node

```yaml
pilot_id: pilot-macos-node
synthetic: true
stack: node
os: macos
owner: DARE Labs
source: fixture:existing-node-project
consent: true
shadow_cycles_done: 3
flows:
  - id: smoke-help
    must: true
    command: ["dare", "--help"]
  - id: version
    must: true
    command: ["dare", "--version"]
  - id: info
    must: true
    command: ["dare", "info"]
  - id: discover-node
    must: true
    command: ["dare", "discover"]
  - id: discover-check
    must: true
    command: ["dare", "discover", "--check"]
  - id: capabilities
    must: true
    command: ["dare", "capabilities"]
```

Fixture path: `tests/fixtures/existing-node-project/` (`package.json` private stub).  
Shadow source (mp055-003): `tests/fixtures/monorepo/` (≥3 files for fingerprint; see `incidents.md` INC-001).

---

### pilot-windows-rust

```yaml
pilot_id: pilot-windows-rust
synthetic: true
stack: rust
os: windows
owner: DARE Labs
source: fixture:existing-rust-project
consent: true
shadow_cycles_done: 3
flows:
  - id: smoke-help
    must: true
    command: ["dare", "--help"]
  - id: version
    must: true
    command: ["dare", "--version"]
  - id: info
    must: true
    command: ["dare", "info"]
  - id: discover-rust
    must: true
    command: ["dare", "discover"]
  - id: validate
    must: true
    command: ["dare", "validate"]
  - id: update-dry-run
    must: true
    command: ["dare", "update", "--dry-run"]
  - id: self-help
    must: true
    command: ["dare", "self", "--help"]
  - id: mcp-help
    must: true
    command: ["dare", "mcp", "--help"]
  - id: harness-help
    must: true
    command: ["dare", "harness", "--help"]
```

Fixture path: `tests/fixtures/existing-rust-project/` (minimal `Cargo.toml` package).  
Shadow source (mp055-003): `tests/fixtures/monorepo/` (≥3 files for fingerprint; see `incidents.md` INC-001).

---

## Fixture materialisation

| Fixture id | Path | Contents |
|------------|------|----------|
| `empty-project` | `tests/fixtures/empty-project/` | `.gitkeep` (+ optional README) |
| `existing-node-project` | `tests/fixtures/existing-node-project/` | `package.json` (`name`, `version`, `private: true`) |
| `existing-rust-project` | `tests/fixtures/existing-rust-project/` | minimal `Cargo.toml` package |

---

## Notes

- `shadow_cycles_done` is `3` for each seed pilot after mp055-003 (≥ `MIN_SHADOW_CYCLES`).
- Do not commit secrets, tokens, or personally identifying paths from real pilots.
- Shadow evidence: `docs/pilot/results/<pilot_id>/cycle-{1,2,3}.md`; incidents: `docs/pilot/incidents.md`.