# /dare-bootstrap

Executa o scaffolder oficial da stack registrada em `dare.config.json` para materializar o esqueleto do framework no projeto atual.

> Este comando expõe o CLI `dare bootstrap` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Logo após `dare init`, para gerar o esqueleto real do framework.
- Quando `dare.config.json` existe mas os artefatos do framework ainda não foram gerados.

## Como rodar

```bash
dare bootstrap
dare bootstrap --force   # roda mesmo se já houver artefatos (pode sobrescrever)
```

## O que fazer

1. Confirme que existe `dare.config.json` com a stack definida.
2. Rode `dare bootstrap` (use `--force` apenas se o usuário aceitar sobrescrever arquivos existentes).
3. Verifique a saída: arquivos gerados e próximos passos sugeridos pelo CLI.

## Comandos relacionados

`/dare-init` · `/dare-design`
