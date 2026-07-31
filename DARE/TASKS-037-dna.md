# Tasks: DNA (037)

> **Fonte:** `DARE/BLUEPRINT-037-dna.md`  
> **Design:** `DARE/DESIGN-037-dna.md`  
> **DAG:** `DARE/dare-dag-037.yaml`  
> **Specs:** `DARE/EXECUTION-037/`  
> **Progresso:** 6/6 (100%)  
> **Nota:** IDs `mp037-*`; **fora** reverse/patterns/migrate; DEC-039

## Visão Geral

- Total de Tasks: 6
- Ranks: collect → render/git-ast → CLI → docs → Ralph
- Tempo estimado: ~8–12 h

## Tabela de Status

| ID        | Título                                              | Status  | Depends On              | Complexity |
|-----------|-----------------------------------------------------|---------|-------------------------|------------|
| mp037-001 | Domínio dna collect + report types                  | ✅ DONE | —                       | MED        |
| mp037-002 | Render PROJECT-DNA + dna-facts + write/check        | ✅ DONE | mp037-001               | MED        |
| mp037-003 | Git log + AST opt-in + graph soft                   | ✅ DONE | mp037-001               | MED        |
| mp037-004 | CLI dare dna + main.rs                              | ✅ DONE | mp037-002, mp037-003    | MED        |
| mp037-005 | Capability + docs DEC-039 + matriz                  | ✅ DONE | mp037-004               | LOW        |
| mp037-006 | Smokes + Ralph close                                | ✅ DONE | mp037-005               | MED        |

## Progresso

```
████████████████████ 100%
```

## Entrega

- `crates/dare-project/src/dna.rs`
- `crates/dare-cli/src/commands/dna.rs` + wire `main.rs`
- `assets/capabilities/dare-dna`
- `docs/compatibility/cli-dna.md` + **DEC-039**
- Smokes: success, `--check`, no-git
