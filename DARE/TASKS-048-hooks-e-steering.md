# Tasks: Hooks e steering (048)

> **Fonte:** `DARE/BLUEPRINT-048-hooks-e-steering.md` (APPROVED)  
> **Design:** `DARE/DESIGN-048-hooks-e-steering.md`  
> **DAG:** `DARE/dare-dag-048.yaml`  
> **Specs:** `DARE/EXECUTION-048/`  
> **DEC:** DEC-049  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 002) → 1 (003 ∥ 006) → 2 (004) → 3 (005) → 4 (007)

## Tabela de Status

| ID        | Título                                           | Status  | Depends On           | Complexity |
|-----------|--------------------------------------------------|---------|----------------------|------------|
| mp048-001 | dare-hooks: events + actions + defs/embed        | ✅ DONE | —                    | HIGH       |
| mp048-002 | dare-steering: frontmatter + resolve + .env deny | ✅ DONE | —                    | HIGH       |
| mp048-003 | hooks trust + validate + idempotency             | ✅ DONE | mp048-001            | HIGH       |
| mp048-004 | hooks run_hooks + SafeCommand spawn              | ✅ DONE | mp048-003            | HIGH       |
| mp048-005 | CLI dare hooks list/run/validate                 | ✅ DONE | mp048-004            | MED        |
| mp048-006 | CLI dare steering list/show                      | ✅ DONE | mp048-002            | MED        |
| mp048-007 | Docs DEC-049 + capabilities + Ralph              | ✅ DONE | mp048-005, mp048-006 | MED        |

## Progresso

```
████████████████████ 100%
```

## Entregáveis

- Crates `dare-hooks` + `dare-steering`
- CLI `dare hooks` / `dare steering`
- Docs `cli-hooks-steering.md` + **DEC-049**
- Capabilities `hooks` / `steering`
