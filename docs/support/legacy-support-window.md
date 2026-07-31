# Legacy support window — `@dewtech/dare-cli`

> **Microplano:** 056  
> **Package:** `@dewtech/dare-cli` (TypeScript / npm legacy line)

## Machine fields

| Field | Value |
|-------|-------|
| `window_start` | `2026-07-31` |
| `window_end` | `2026-10-29` |
| `scope` | `security-only` |
| `contact` | Release/Ops (maintainers of `darelabs-tech/dare-cli`) |
| `escalation` | [`incident-response.md`](incident-response.md) |

## Window

| Boundary | ISO-8601 date | Notes |
|----------|---------------|-------|
| Start | **`2026-07-31`** | Stable cutover seed date (microplano 056) |
| End | **`2026-10-29`** | Seed = start **+ 90 days** (adjustable by maintainers) |

After `window_end`, no further security patches are promised on the npm legacy line. Migrate to Rust **`v4.0.0`**: [`../migration/install-rust.md`](../migration/install-rust.md).

## Scope

**`security-only`** on the TypeScript / npm line:

| Allowed | Rejected |
|---------|----------|
| CVE HIGH/CRITICAL fixes, Class D / RS-07 must_fix | Features, refactors, non-security bugfixes, capability bumps |

Canonical policy fields: [`../migration/npm-legacy-policy.md`](../migration/npm-legacy-policy.md) (`security_fixes_until` ≥ this window end).

## Contact

| Role | Contact |
|------|---------|
| Owner | **Release/Ops** |
| Channel | GitHub Issues / Discussions on `darelabs-tech/dare-cli` (no private tokens or PII in tickets) |

Do **not** put secrets, credentials, or personal data in public issue bodies.

## Escalation

Security or production incidents involving the legacy line escalate via [`incident-response.md`](incident-response.md) (P0–P3, ack SLAs). Pilot-era log: [`../pilot/incidents.md`](../pilot/incidents.md).

## Related

- [`../migration/RELEASE-NOTES-stable.md`](../migration/RELEASE-NOTES-stable.md)
- [`../release-candidate/typescript-freeze.md`](../release-candidate/typescript-freeze.md) (superseded by 056 legacy policy)
