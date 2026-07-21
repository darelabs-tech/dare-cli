---
name: dare-update
description: Sincroniza os artefatos do projeto (comandos de IDE, skills, templates) com a versão instalada do DARE CLI, preservando customizações. Mapeia o CLI `dare update`.
---

# Atualizar o projeto para a versão atual do CLI

Sincroniza os artefatos do projeto (comandos de IDE, skills, templates) com a versão instalada do DARE CLI, preservando customizações.

> Este comando expõe o CLI `dare update` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Depois de atualizar o binário `dare` para uma versão nova.
- Quando `dare info` apontar artefatos desatualizados.

## Como rodar

```bash
dare update --dry-run                    # plano sem escrever (obrigatório até mp022)
dare update --dry-run --target codex     # só assets Codex / AGENTS.md
dare update --dry-run -d . --json        # envelope UpdatePlan schema 1
```

`--target` aceita harness ids (`claude-code`, `cursor`, `codex`, `antigravity`, `hybrid`, `claude-hybrid`) — **não** semver.

Apply real (`dare update` sem `--dry-run`) ainda não implementado; use `--dry-run` e aguarde microplano 022.

## O que fazer

1. Rode `dare update --dry-run` e revise contagens (`identical`, `missing`, `apply`, `customized`).
2. Se houver `customized`, confirme com o utilizador antes de qualquer apply futuro.
3. Não use semver em `--target`; filtre por harness quando quiser escopo IDE específico.

## Comandos relacionados

`/dare-info` · `/dare-welcome`
