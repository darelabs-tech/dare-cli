# Ralph Loop

The **Ralph Loop** is the automated self-correction loop inside the Execute phase.

```
1. AI implements task code
        │
        ▼
2. Validation gates run automatically (tests, clippy, format)
        │
        ├── All passed? ──► Task DONE ✓
        │
        └── Failure? ────► Reads error ──► Fixes code ──► (loop back to 2)
```
