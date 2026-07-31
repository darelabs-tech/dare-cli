# Tasks: MCP real como transporte separado (052)

> **Fonte:** `DARE/BLUEPRINT-052-mcp-real-como-transporte-separado.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-052-mcp-real-como-transporte-separado.md`  
> **DAG:** `DARE/dare-dag-052.yaml`  
> **Specs:** `DARE/EXECUTION-052/`  
> **DEC:** DEC-053  
> **Progresso:** 0/7 (0%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 002 ∥ 006) → 1 (003) → 2 (004 ∥ 005) → 3 (007)
- Tempo estimado: ~14–18 h
- Escopo: services compartilhados · `rmcp` · tools MCP · stdio + streamable-http · CLI `dare mcp serve` · alias `dare-mcp-server` · DEC-053

## Tabela de Status

| ID        | Título                                              | Status     | Depends On     | Complexity |
|-----------|-----------------------------------------------------|------------|----------------|------------|
| mp052-001 | Services layer + REST delegation                    | ⏳ PENDING | —              | HIGH       |
| mp052-002 | Workspace rmcp pins + feature mcp scaffold          | ⏳ PENDING | —              | MED        |
| mp052-003 | MCP handler + 10 tools + error_map                  | ⏳ PENDING | mp052-001, 002 | HIGH       |
| mp052-004 | stdio + CLI dare mcp serve                          | ⏳ PENDING | mp052-003      | HIGH       |
| mp052-005 | streamable-http serve (port 3100)                   | ⏳ PENDING | mp052-003      | HIGH       |
| mp052-006 | Alias bin dare-mcp-server (REST + deprecation)      | ⏳ PENDING | —              | MED        |
| mp052-007 | Docs DEC-053 + capability + Ralph                   | ⏳ PENDING | mp052-004, 005, 006 | MED   |

## Progresso

```
░░░░░░░░░░░░░░░░░░░░ 0%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp052-001 — services + REST delegation
- mp052-002 — rmcp feature scaffold
- mp052-006 — alias `dare-mcp-server`

### Rank 1
- mp052-003 — MCP tools/handler (← 001, 002)

### Rank 2 (paralelo)
- mp052-004 — stdio + CLI (← 003)
- mp052-005 — streamable-http (← 003)

### Rank 3
- mp052-007 — docs + DEC-053 + Ralph (← 004, 005, 006)

## Caminho crítico

`001 → 003 → 004 → 007` (002 ∥ 001; 005 ∥ 004; 006 ∥ rank0)

## Ready agora

🟢 **mp052-001**, **mp052-002**, **mp052-006**

```text
dare execute --parallel --dag DARE/dare-dag-052.yaml
```

## Próximas Etapas

1. Revisar `DARE/dag-graph-052.mmd`
2. Executar rank 0 em paralelo
3. Após 007 DONE → microplano **053**
