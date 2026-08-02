# Dare CLI Book

> **Design. Architect. Review. Execute.**
> A structured methodology for AI-assisted software development, with mandatory human checkpoints.

---

## What is DARE CLI?

**DARE CLI** is a command line tool written in Rust that implements the **DARE Method** — a methodology that balances AI speed with structure, context, and human reviews at each phase transition.

AI software development operates today in two extremes:

| Vibe Coding | Traditional |
|---|---|
| "Give me a code that does X" + hope | Detailed specification done only by humans |
| Fast for prototypes, **chaos to evolve** | Slow, **makes little use of AI** |
| No auditability of reasoning | No real productivity gains |

### The DARE Solution

DARE fills this gap:

1. **Persistent Context**: each phase generates markdown artifacts (`DESIGN.md`, `BLUEPRINT.md`) that feed the next phase — the AI never starts from scratch.
2. **Human Checkpoints**: design and architecture require explicit approval — no "surprises" after hours of execution.
3. **Deterministic Execution**: the DAG of tasks ensures safe parallelism and trackable state.
4. **Ralph Loop**: the AI automatically iterates until verification gates (tests, linter, formatter) pass.

---

## The 4 Phases of the Method

```
1. DESIGN      →   2. ARCHITECT   →   3. REVIEW      →   4. EXECUTE
Define requirements    Propose architecture   Human approval     Ralph Loop code
(DESIGN.md)            (BLUEPRINT.md)         (Checkpoints)      (Tests & Gates)
```

- **Fase 1 — Design**: Focuses on **what** to build and **why**.
- **Fase 2 — Architect**: Decides **how** to build it, generating a blueprint and a task dependency graph.
- **Fase 3 — Review**: Verification step before the AI writes code.
- **Fase 4 — Execute**: Automated execution of tasks with verification loops.
