# Tasks: Engine AST nativo (035)

> **Fonte:** `DARE/BLUEPRINT-035-engine-ast-nativo.md`  
> **Design:** `DARE/DESIGN-035-engine-ast-nativo.md`  
> **DAG:** `DARE/dare-dag-035.yaml`  
> **Specs:** `DARE/EXECUTION-035/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp035-*`; crate `dare-ast`; **fora** reverse/dna/patterns/CLI

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 8 (sequencial)
- Tempo estimado: ~12–16 h

## Tabela de Status

| ID        | Título                                           | Status  | Depends On | Complexity |
|-----------|--------------------------------------------------|---------|------------|------------|
| mp035-001 | Scaffold crate + model/language                  | ✅ DONE | —          | MED        |
| mp035-002 | Parse + feature-gated grammars                   | ✅ DONE | mp035-001  | HIGH       |
| mp035-003 | Extract AST endpoints/entities                   | ✅ DONE | mp035-002  | HIGH       |
| mp035-004 | Regex fallback + merge/dedupe                    | ✅ DONE | mp035-003  | HIGH       |
| mp035-005 | analyze API + corpus fixtures                    | ✅ DONE | mp035-004  | MED        |
| mp035-006 | Docs DEC-032 + ast-engine.md                     | ✅ DONE | mp035-005  | LOW        |
| mp035-007 | Ralph Loop (fmt/clippy/test/audit)               | ✅ DONE | mp035-006  | MED        |
| mp035-008 | Fechamento TASKS/matriz/Blueprint                | ✅ DONE | mp035-007  | LOW        |

## Progresso

```
████████████████████ 100%
```

## Entrega

- Crate `dare-ast` (parse/extract/regex/merge/analyze)
- Fixtures por linguagem (+ tsx)
- Docs: `docs/compatibility/ast-engine.md` + **DEC-032**
