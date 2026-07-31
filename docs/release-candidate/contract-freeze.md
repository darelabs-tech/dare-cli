# Contract freeze (RC)

> **Microplano 055** · Fase D · RC tag: **`v4.0.0-rc1`**

During the release-candidate window, **Classe A** contracts stay frozen unless the full breaking-change path is completed. This document does not replace [`../compatibility/breaking-change-process.md`](../compatibility/breaking-change-process.md); it tightens the checklist for RC PRs.

## Scope (Classe A)

Classe A items in [`../compatibility/classification-matrix.md`](../compatibility/classification-matrix.md) (preserve), including but not limited to:

- Public exit codes
- Public command and flag names
- Persisted schemas (`dare.config.json`, state, DAG)
- Canonical IDs
- Public JSON field shapes / schemas touched by the closed breaking list

Touching exit codes, flags, JSON schemas, or canonical IDs in a PR is presumed Classe A / breaking until proven otherwise.

## MUST checklist (before merge)

Any Classe A change during RC **MUST** satisfy all of the following:

- [ ] **ADR Accepted** — link to an ADR with `status: Accepted` (cite `ADR-NNN` in the PR)
- [ ] **`classification-matrix.md` updated** — class / action / `adr_ref` reflect the change when applicable
- [ ] **`DECISION-LOG` entry** — required when a waiver, deferred scope, or exception applies
- [ ] **PR description cites `ADR-`** — when the PR touches exit codes, flags, public JSON schema, or canonical IDs

Merge without ADR Accepted for a Classe A / breaking change is an RS-03 violation — reject in review.

## Process link

Full state machine, closed breaking-type list, and PR preconditions:

→ [`../compatibility/breaking-change-process.md`](../compatibility/breaking-change-process.md)

## CI (SHOULD)

Automated jobs that fail when contract paths change without an `ADR-` reference in the PR body remain **SHOULD** (same posture as the breaking-change process COULD for cycle 001). Human checklist above is **MUST**.

## Related

- [`typescript-freeze.md`](typescript-freeze.md) — TS legacy security-fixes-only policy
- [`../compatibility/classification-matrix.md`](../compatibility/classification-matrix.md)
- [`../DECISION-LOG.md`](../DECISION-LOG.md)
- [`../compatibility/README.md`](../compatibility/README.md)
