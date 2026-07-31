# Incident response (post-stable)

> **Language:** en-US (public process)  
> **Microplano:** 056  
> **Owner:** Release/Ops  
> **Ack timezone:** America/Sao_Paulo (BRT, UTC−3) — business days Mon–Fri excluding local public holidays observed by the project maintainers

## Severity (P0–P3)

Same lexicon as microplano 055 / pilot shadow findings:

| Severity | Meaning | Close gate impact |
|----------|---------|-------------------|
| **P0** | Data loss, security bypass, CLI unusable | Must not remain `open` for cutover / release gates |
| **P1** | MUST flow fails without an accepted workaround | Must not remain `open` for cutover / release gates |
| **P2** | Documented workaround available | May close / mitigate without blocking |
| **P3** | Cosmetic / docs | May close without blocking |

## Acknowledgement SLA (`ack_sla`)

| Severity | Ack SLA |
|----------|---------|
| **P0** | ≤ **4 hours** |
| **P1** | ≤ **1 business day** |
| **P2** | Best effort within 3 business days |
| **P3** | Best effort / backlog |

Ack = a maintainer has triaged severity, owner, and next step in the incident log (status may still be `open`).

## Owner

| Role | Value |
|------|-------|
| `owner` | **Release/Ops** |
| Escalation path | This document → maintainers of `darelabs-tech/dare-cli` via GitHub Issues |

Do not embed secrets, tokens, passwords, or PII in public issue bodies or logs.

## Incident log

| Era | Log |
|-----|-----|
| Pilot / RC (055) | [`../pilot/incidents.md`](../pilot/incidents.md) |
| Post-cutover (056+) | Continue appending to [`../pilot/incidents.md`](../pilot/incidents.md) **or** open a dated section there until a dedicated post-stable log is split — keep the same severity columns (`id`, `sev`, `status`, `summary`, `opened`, `closed`) |

## Process (after cutover)

1. **Detect** — operator, CI, or user report (redact secrets).
2. **Classify** — assign P0–P3 using the table above.
3. **Ack** — within `ack_sla`; record owner and next step.
4. **Mitigate / fix** — prefer native Rust `v4.0.0` path; legacy npm only if in [`legacy-support-window.md`](legacy-support-window.md) and security-scoped.
5. **Close** — set `status` to `closed` / `mitigated` with date; link PR or advisory id (no secrets).

## Related

- [`legacy-support-window.md`](legacy-support-window.md) — npm security-only window
- [`../migration/install-rust.md`](../migration/install-rust.md) — recommended install
- [`../migration/RELEASE-NOTES-stable.md`](../migration/RELEASE-NOTES-stable.md) — stable notes
- [`../pilot/incidents.md`](../pilot/incidents.md) — incident log
