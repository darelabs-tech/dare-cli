# Parity diff log (MP-054)

Index of classified TypeScript ↔ Rust parity differences for the golden harness.
Used by `DiffLogIndex` / case validation (`class` C rows require `adr_ref` or an entry here).

See also: [`parity-hardening.md`](parity-hardening.md) (how to run suites, N-01..N-08, perf gate).

Classes: **A** preserve · **B** fix without ADR · **C** intentional / ADR · **D** must-fix (security).

| diff_id | surface | class | action | adr_ref | notes |
|---------|---------|-------|--------|---------|-------|
| PD-001 | design LLM variance | C | accept native SoT | ADR-pending / DEC-024–025 | Rust SoT is native design pipeline; TS LLM wording variance is Class C — do not over-normalize prose |
