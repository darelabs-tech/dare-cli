# DESIGN — E2E Execution & Recovery

## Goal
Validate production CLI behavior across independent processes: DAG validation, selection, timeout, durable state, recovery, verified completion and diagnostics.

## Constraints
Use existing CLI and fixtures, `DARE_RALPH_MOCK` exclusively for deterministic verification, no external agents or networking; no merge without explicit human authorization.

## Acceptance
A single integration scenario covers failure and recovery, supported CI quality gates pass, and evidence references exact commit and workflow run. Do not claim production recovery beyond tested scenarios.
