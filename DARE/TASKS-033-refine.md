# TASKS: Refine e sub-DAG (Microplano 033)

> **Blueprint:** `DARE/BLUEPRINT-033-refine.md`  
> **DAG:** `DARE/dare-dag-033.yaml`  
> **Specs:** `DARE/EXECUTION-033/`  
> **Status:** DONE

| ID | Título | Depends | Complexity | Status |
|----|--------|---------|------------|--------|
| mp033-001 | subdag: score + propose_split | — | HIGH | DONE |
| mp033-002 | spliceSubDag + depth/cycle/state | mp033-001 | HIGH | DONE |
| mp033-003 | CLI `dare refine` + smokes | mp033-002 | MED | DONE |
| mp033-004 | Capability + docs + DEC-040 + matriz | mp033-003 | MED | DONE |
| mp033-005 | Ralph Loop dare-dag + dare-cli | mp033-004 | LOW | DONE |
