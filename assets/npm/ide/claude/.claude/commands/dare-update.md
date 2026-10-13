# /dare-update

Sincroniza os artefatos do projeto (comandos de IDE, skills, templates) com a versão instalada do DARE CLI, preservando customizações.

> Este comando expõe o CLI `dare update` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Depois de atualizar o `@dewtech/dare-cli` para uma versão nova.
- Quando `dare info` apontar artefatos desatualizados.

## Como rodar

```bash
dare update --dry-run     # mostra o que mudaria, sem escrever
dare update -y            # aplica tudo, mantém customizações
dare update --target 3.2.0
```

## O que fazer

1. Rode `dare update --dry-run` e revise o diff proposto.
2. Se estiver ok, rode `dare update -y`.
3. Evite `--force` salvo se o usuário aceitar sobrescrever arquivos customizados.

## Comandos relacionados

`/dare-info` · `/dare-welcome`
