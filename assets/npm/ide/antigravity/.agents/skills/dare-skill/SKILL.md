---
name: dare-skill
description: Adiciona, remove, lista, inspeciona, atualiza ou publica skills DARE neste projeto. Mapeia o CLI `dare skill`.
---

# Gerenciar skills DARE do projeto

Adiciona, remove, lista, inspeciona, atualiza ou publica skills DARE neste projeto.

> Este comando expõe o CLI `dare skill` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Você quer instalar uma skill extra (ex.: uma skill de stack) no projeto.
- Quer listar/inspecionar as skills disponíveis ou publicar uma própria.

## Como rodar

```bash
dare skill list
dare skill info <nome>
dare skill add <nome>
dare skill remove <nome>
dare skill update
```

## O que fazer

1. Use `dare skill list` para ver o que está instalado/disponível.
2. Rode o subcomando desejado (`add`/`remove`/`info`/`update`/`publish`).
3. Confirme o resultado e, se mudou comandos de IDE, recarregue a IDE.

## Comandos relacionados

`/dare-update` · `/dare-info`
