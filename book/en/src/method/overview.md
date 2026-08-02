# The DARE Method

**Design. Architect. Review. Execute.**

The DARE Method is a methodology for AI-assisted software development that solves a fundamental problem: how to maintain AI speed without losing control and auditability?

---

## The 4 Phases

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│   1. DESIGN     →  2. ARCHITECT  →  3. REVIEW   →  4. EXECUTE           │
│   ─────────        ─────────────    ─────────      ──────────            │
│   Human            AI proposes      Human          AI implements         │
│   defines          architecture     validates      + Ralph Loop          │
│   requirements                      and approves                         │
│                                                                          │
│   ↓ DESIGN.md      ↓ BLUEPRINT.md   ↓ ✓ approval    ↓ Code + Tests ✓     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

| Phase | Description | Who does it | Output |
|---|---|---|---|
| **1. Design** | Defines **what** to build and **why** | Human (AI assists) | `DARE/DESIGN.md` |
| **2. Architect** | Decides **how** to build it | AI proposes, human validates | `DARE/BLUEPRINT.md` |
| **3. Review** | Approves or adjusts the plan before execution | Human | ✓ explicit approval |
| **4. Execute** | Implements tasks with Ralph Loop | AI | Code + green tests |
