# Tasks: Patterns (038)

> **Fonte:** `DARE/BLUEPRINT-038-patterns.md`  
> **Design:** `DARE/DESIGN-038-patterns.md`  
> **DAG:** `DARE/dare-dag-038.yaml`  
> **Specs:** `DARE/EXECUTION-038/`  
> **Progresso:** 6/6 (100%)  
> **Nota:** IDs `mp038-*`; **fora** reverse/dna/migrate; DEC-041

## Visão Geral

- Total de Tasks: 6
- Ranks: mine → render/ast → CLI → docs → Ralph
- Tempo estimado: ~8–12 h

## Tabela de Status

| ID        | Título                                              | Status  | Depends On              | Complexity |
|-----------|-----------------------------------------------------|---------|-------------------------|------------|
| mp038-001 | Domínio patterns types + mine kinds/freq            | ✅ DONE | —                       | MED        |
| mp038-002 | Cooccur + render PATTERNS.md/facts + check/inject   | ✅ DONE | mp038-001               | MED        |
| mp038-003 | AST opt-in + graph soft + --modules                 | ✅ DONE | mp038-001               | MED        |
| mp038-004 | CLI dare patterns + main.rs                         | ✅ DONE | mp038-002, mp038-003    | MED        |
| mp038-005 | Capability + docs DEC-041 + matriz                  | ✅ DONE | mp038-004               | LOW        |
| mp038-006 | Smokes + Ralph close                                | ✅ DONE | mp038-005               | MED        |

## Progresso

```
████████████████████ 100%
```

## Entrega

- `crates/dare-project/src/patterns.rs`
- `crates/dare-cli/src/commands/patterns.rs` + wire `main.rs`
- `assets/capabilities/dare-patterns`
- `docs/compatibility/cli-patterns.md` + **DEC-041**
- Smokes: write, `--check` no-write, help
