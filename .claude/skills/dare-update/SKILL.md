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
dare update --dry-run                    # plano sem escrever
dare update --dry-run --target codex     # só assets Codex / AGENTS.md
dare update --dry-run -d . --json        # envelope UpdatePlan schema 1
dare update -y                           # apply: cria/atualiza managed; keep customized
dare update --force -y                   # apply: sobrescreve customized (com backup)
dare update -y -d . --json               # envelope UpdateApplyReport schema 1
```

`--target` aceita harness ids (`claude-code`, `cursor`, `codex`, `antigravity`, `hybrid`, `claude-hybrid`) — **não** semver.

**`-y` ≠ `--force`:** `-y` aplica sem prompts e **mantém** ficheiros `customized`; só `--force` os sobrescreve (session backup em `.dare/backup-<ver>/`). Apply real está implementado no microplano **022** (DEC-023).

## O que fazer

1. Rode `dare update --dry-run` e revise contagens (`identical`, `missing`, `apply`, `customized`).
2. Se houver `customized`, confirme com o utilizador antes de `--force`.
3. Para apply seguro sem tocar customizações: `dare update -y`.
4. Não use semver em `--target`; filtre por harness quando quiser escopo IDE específico.

## Comandos relacionados

`/dare-info` · `/dare-welcome`
