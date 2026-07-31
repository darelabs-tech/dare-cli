# TASKS: Guard (Microplano 034)

> Fonte: `DARE/BLUEPRINT-034-guard.md` · DAG: `DARE/dare-dag-034.yaml`

| ID | Título | Status | depends_on |
|----|--------|--------|------------|
| mp034-001 | ErrorKind GuardFail + exit 6 | DONE | — |
| mp034-002 | Crate dare-guard + unicode strip/block | DONE | mp034-001 |
| mp034-003 | scan-rules.json + injection scan | DONE | mp034-002 |
| mp034-004 | Proveniência + trustedPaths | DONE | mp034-002 |
| mp034-005 | Assinatura Ed25519 sign/verify | DONE | mp034-002 |
| mp034-006 | Pipeline + GuardReport | DONE | mp034-003, mp034-004, mp034-005 |
| mp034-007 | CLI `dare guard` | DONE | mp034-006 |
| mp034-008 | Preflight agent (substituir stub) | DONE | mp034-006, mp034-007 |
| mp034-009 | Docs DEC-035 + matriz + manifest | DONE | mp034-007, mp034-008 |
| mp034-010 | Ralph Loop close | DONE | mp034-009 |

## Progresso

10/10 DONE
