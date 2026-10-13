# Comando: /dare-validate

Valida `DARE/dare-dag.yaml` (ciclos, referências quebradas, campos obrigatórios). Adequado para pre-commit hooks e CI.

> Este comando expõe o CLI `dare validate` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Antes de commitar mudanças no DAG.
- Em CI, como gate de integridade do plano de execução.

## Como rodar

```bash
dare validate
dare validate --strict          # trata warnings como erros (CI-friendly)
dare validate --dag DARE/dare-dag.yaml
```

## O que fazer

1. Rode `dare validate` (use `--strict` em CI).
2. Se houver erros, corrija o `dare-dag.yaml` apontado e rode de novo até passar.
3. Saída limpa = DAG íntegro e pronto para `/dare-execute`.

## Comandos relacionados

`/dare-dag` · `/dare-execute`
