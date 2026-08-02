# Fase 2 — Architect

A fase Architect é onde **a IA propõe a arquitetura completa** com base no `DARE/DESIGN.md` aprovado.

## Objetivo

Produzir o `DARE/BLUEPRINT.md` — a arquitetura técnica detalhada com decomposição em tasks executáveis.

## Comando

```bash
dare blueprint
```

## O que o Blueprint captura?

| Seção | Conteúdo |
|---|---|
| Trade-offs | Decisões arquiteturais com justificativa |
| Visão geral da arquitetura | Diagrama Mermaid das camadas |
| Stack técnica | Crates/libs escolhidas com versões |
| Modelo de dados | Structs, schemas, contratos |
| Endpoints / Contratos | API, CLI flags, saída JSON |
| Tasks | Decomposição atômica em tasks com dependências |

## Geração do DAG

Após o Blueprint, o comando `dare tasks` gera os 3 artefatos de execução:

```bash
dare tasks   # gera TASKS.md + dare-dag.yaml + EXECUTION/task-*.md
```

O `dare-dag.yaml` define o grafo de dependências entre tasks, permitindo execução paralela pelo [DAG Runner](../engines/dag-runner.md).

## Visualizar o grafo

```bash
dare dag visualize          # exibe no terminal
dare dag visualize --mmd    # exporta Mermaid
```

## Próximo passo

Revise o `DARE/BLUEPRINT.md` e aprove para a [Fase 3 — Review](review-phase.md).
