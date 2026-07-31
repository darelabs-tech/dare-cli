# TASKS: Skills registry — modelo e resolução (Microplano 044)

> **Blueprint:** `DARE/BLUEPRINT-044-skills-registry-modelo-e-resolucao.md`  
> **DAG:** `DARE/dare-dag-044.yaml`  
> **Specs:** `DARE/EXECUTION-044/`

| ID | Título | Status | Depends |
|----|--------|--------|---------|
| mp044-001 | Scaffold crate `dare-skills` + workspace | DONE | — |
| mp044-002 | Model + mock registry + classify | DONE | mp044-001 |
| mp044-003 | Local/remote/composite + topo resolve | DONE | mp044-002 |
| mp044-004 | CLI `dare skill list\|info` + smokes | DONE | mp044-003 |
| mp044-005 | Docs + DEC-033 + matriz 044 Concluído | DONE | mp044-004 |

## Ralph Loop (por task)

`cargo fmt` → `cargo clippy --workspace --all-targets -- -D warnings` → `cargo test --workspace`
