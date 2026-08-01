---
name: dare-discover
description: Detecta a stack de um projeto já existente (brownfield) e instala os arquivos da metodologia DARE — incluindo os comandos/skills de IDE — sem tocar no código. Mapeia o CLI `dare discover`.
---

# Adotar o DARE em um projeto existente

Detecta a stack de um projeto já existente (brownfield) e instala os arquivos da metodologia DARE — incluindo os comandos/skills de IDE — sem tocar no código.

> Este comando expõe o CLI `dare discover` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- O usuário quer adotar o DARE em um repositório que **já existe**.
- Para entender/documentar o legado em profundidade depois, encadeie com `/dare-reverse` e `/dare-dna`.

## Como rodar

```bash
dare discover
dare discover --dir ./caminho/do/projeto
dare discover --check    # só mostra a detecção, sem instalar nada
```

## O que fazer

1. Rode `dare discover --check` primeiro para revisar a stack detectada.
2. Se a detecção estiver correta, rode `dare discover` para instalar os artefatos DARE + comandos de IDE.
3. Próximo passo: `/dare-reverse` (Fase 0 — coleta) e `/dare-dna` (convenções).

## Comandos relacionados

`/dare-reverse` · `/dare-dna` · `/dare-migrate`
