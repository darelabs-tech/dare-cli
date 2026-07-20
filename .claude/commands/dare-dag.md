# /dare-dag

Mostra o DAG estático de tasks (`DARE/dare-dag.yaml`): ranks, dependências e caminho crítico. Use `dare dag viz` para exportar o diagrama.

> Este comando expõe o CLI `dare dag` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Você quer ver a ordem de execução, os ranks e o caminho crítico das tasks.
- Antes de começar a execução, para conferir a topologia do plano.

## Como rodar

```bash
dare dag viz
dare dag viz --dag DARE/dare-dag.yaml
```

## O que fazer

1. Rode `dare dag viz` para renderizar o grafo de tasks.
2. Confira ranks e dependências; tasks de mesmo rank podem rodar em paralelo.
3. Para validar a integridade do arquivo, use `/dare-validate`. Para executar, `/dare-execute`.

## Comandos relacionados

`/dare-validate` · `/dare-execute` · `/dare-graph`
