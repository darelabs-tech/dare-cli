# DESIGN — Diagnostics: Locks & Safe Support Bundles

Status: implementation pending

## Goal
Finish the deferred operational diagnostics features from the previous cycle: accurate execution lock diagnostics and opt-in redacted local support bundles.

## Constraints
- Identify the canonical lock implementation before implementing detection; never delete or override an active lock.
- No automatic network upload, credential collection or unreviewed file inclusion.
- Support bundles must use a strict allowlist and redact secrets and personal data before writing.
- Preserve existing `dare info` and `--json` contracts.
- Merge requires separate approval.
