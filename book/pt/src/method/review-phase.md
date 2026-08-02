# Fase 3 — Review

A fase de Review é um **checkpoint humano obrigatório** entre a arquitetura proposta e a execução. Nenhuma task é implementada sem aprovação explícita.

## Objetivo

Revisar `DARE/BLUEPRINT.md` e `DARE/TASKS.md`, ajustar se necessário, e aprovar formalmente o plano de execução.

## O que revisar?

### No BLUEPRINT.md

- ✅ Os trade-offs arquiteturais fazem sentido?
- ✅ A stack técnica está alinhada com os requisitos?
- ✅ O modelo de dados está correto?
- ✅ Os contratos de API/CLI estão completos?

### No TASKS.md / dare-dag.yaml

- ✅ As tasks são atômicas e implementáveis?
- ✅ As dependências entre tasks estão corretas?
- ✅ Alguma task está grande demais? (use `dare refine`)

## Comando de auditoria

```bash
dare review
```

O `dare review` analisa o que já foi implementado e cruza com as specs:

- Detecta stubs, mocks fora de testes, funções vazias
- Identifica TODOs deixados pela IA
- Valida critério a critério se a implementação satisfaz a spec
- Emite um veredicto: PASS / FAIL com lista de gaps

## Refinando tasks grandes

Se uma task ficou complexa demais, use:

```bash
dare refine task-023
```

O `dare refine` quebra tasks de alta complexidade em sub-tasks menores e regenera o DAG.

## Aprovação e continuidade

Após a revisão e aprovação, prossiga para a [Fase 4 — Execute](execute-phase.md):

```bash
dare execute task-001
```
