# Tasks: Reverse (036)

> **Fonte:** `DARE/BLUEPRINT-036-reverse.md`  
> **Design:** `DARE/DESIGN-036-reverse.md`  
> **DAG:** `DARE/dare-dag-036.yaml`  
> **Specs:** `DARE/EXECUTION-036/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp036-*`; **fora** dna/patterns/migrate

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 8 (sequencial)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                           | Status  | Depends On | Complexity |
|-----------|--------------------------------------------------|---------|------------|------------|
| mp036-001 | Domain types + module scan                       | ✅ DONE | —          | MED        |
| mp036-002 | Artefatos IDEIA/module/facts + --check           | ✅ DONE | mp036-001  | HIGH       |
| mp036-003 | --deep / excalidraw / --report                   | ✅ DONE | mp036-002  | MED        |
| mp036-004 | --ast merge estável                              | ✅ DONE | mp036-002  | MED        |
| mp036-005 | CLI reverse + enrichment soft-fail               | ✅ DONE | mp036-003, mp036-004 | HIGH |
| mp036-006 | Capability + docs + DEC-038                      | ✅ DONE | mp036-005  | LOW        |
| mp036-007 | Smokes + Ralph Loop                              | ✅ DONE | mp036-006  | MED        |
| mp036-008 | Fechamento TASKS/matriz/Blueprint                | ✅ DONE | mp036-007  | LOW        |

## Progresso

```
████████████████████ 100%
```

## Entrega

- `dare reverse` + `dare-project::reverse`
- Capability `dare-reverse`
- Docs `cli-reverse.md` + **DEC-038**
