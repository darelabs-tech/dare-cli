# Comando: /dare-hooks

Gerencia e executa agent hooks (`dare hooks list|run|validate`) — determinístico, com trust gate.

## Como rodar

```bash
dare hooks list [--json]
dare hooks run on-save --file src/x.ts --trust
dare hooks validate [--json]
```

## Exit codes

- `0` — sucesso
- `1` — ação falhou ou config inválida (`validate`)
- `2` — evento desconhecido ou hooks não confiáveis (sem `--trust`)
