# Comando: /dare-steering

Inspeciona steering files resolvidos (`dare steering list|show`) — determinístico, sem LLM.

> Este comando expõe o CLI `dare steering` na IDE.

## Como rodar

```bash
dare steering list
dare steering list --json
dare steering show src/auth/login.ts
dare steering show src/auth/login.ts --json
```

## O que fazer

1. `list` — arquivos descobertos (incl. PROJECT-DNA) na ordem de precedência.
2. `show <file>` — blocos aplicáveis ao arquivo, base → project → glob.
