# TASKS: Skills lifecycle e publish seguro (Microplano 045)

> **Blueprint:** `DARE/BLUEPRINT-045-skills-lifecycle-e-publish-seguro.md`  
> **Status:** IN PROGRESS → DONE ao fechar mp045-005

| ID | Título | Status | depends_on | Complexidade |
|----|--------|--------|------------|--------------|
| mp045-001 | Artefatos DARE (DESIGN/BLUEPRINT/TASKS/dag/EXECUTION) | ✅ DONE | [] | LOW |
| mp045-002 | install.rs — add/remove/update atômico + archive jail | ✅ DONE | [mp045-001] | HIGH |
| mp045-003 | publish.rs — tar.gz + sha256 + Ed25519 + MIT/dare_version | ✅ DONE | [mp045-001] | MED |
| mp045-004 | CLI SkillAction + smokes add/remove/publish/traversal | ✅ DONE | [mp045-002, mp045-003] | MED |
| mp045-005 | Docs cli-skill.md + DEC-043 + matriz 045 Concluído | ✅ DONE | [mp045-004] | LOW |

## Ralph (obrigatório)

```bash
cargo fmt --check
cargo clippy -p dare-skills -p dare-cli --all-features -- -D warnings
cargo test -p dare-skills -p dare-cli
```
