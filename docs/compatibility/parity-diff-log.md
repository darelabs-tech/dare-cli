# Parity diff log (MP-054)

Index of classified TypeScript ↔ Rust parity differences for the golden harness.
Used by `DiffLogIndex` / case validation (`class` C rows require `adr_ref` or an entry here).

Classes: **A** preserve · **B** fix without ADR · **C** intentional / ADR · **D** must-fix (security).

| diff_id | surface | class | action | adr_ref | notes |
|---------|---------|-------|--------|---------|-------|
| PD-001 | design LLM variance | C | accept native SoT | ADR-pending / DEC-054 | Rust SoT is native design pipeline; TS LLM wording variance is Class C — do not over-normalize prose |
