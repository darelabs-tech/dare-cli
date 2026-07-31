# TASKS: Review (Microplano 032)

> **Blueprint:** `DARE/BLUEPRINT-032-review.md`  
> **DAG:** `DARE/dare-dag-032.yaml`  
> **Specs:** `DARE/EXECUTION-032/`  
> **Status:** READY FOR EXECUTE (autorizado)

| ID | Título | Depends | Complexity | Status |
|----|--------|---------|------------|--------|
| mp032-001 | Crate dare-review: rules + scan | — | HIGH | DONE |
| mp032-002 | run_review + formatters + agent merge | mp032-001 | HIGH | DONE |
| mp032-003 | CLI `dare review` + smokes | mp032-002 | MED | DONE |
| mp032-004 | Capability + docs + DEC-034 + matriz | mp032-003 | MED | DONE |
| mp032-005 | Ralph Loop workspace | mp032-004 | LOW | DONE |
