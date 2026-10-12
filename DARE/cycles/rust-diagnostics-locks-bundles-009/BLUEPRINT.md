# BLUEPRINT — Locks & Support Bundles

1. Inspect lock ownership, paths and stale-lock behavior in the existing execution/state code.
2. Implement read-only lock health diagnostics with explicit unknown/active/stale semantics; never infer staleness from existence alone.
3. Implement an opt-in bundle preview that lists every included file and excluded sensitive source.
4. Write only allowlisted, redacted diagnostics to a local bundle; no upload.
5. Add fixture tests for active/stale/unknown locks, redaction, path traversal and archive contents.
6. Run Rust formatting, tests, Clippy, build and applicable audit; record actual workflow evidence.
