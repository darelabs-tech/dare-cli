---
name: dare-guard
description: Run the DARE security guard gate on artifacts (dare guard).
---

# /dare-guard

Executa o gate de segurança sobre artefatos (`dare guard <path>|--staged|--all`).

```bash
dare guard DARE/EXECUTION/task-601.md
dare guard --staged [--strict] [--format json]
dare guard --all [--unicode strip|block]
```

Exit codes: `0` PASS/WARN; `6` FAIL (ou WARN com `--strict`).
