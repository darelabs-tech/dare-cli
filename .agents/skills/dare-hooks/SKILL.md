# dare-hooks Skill

Expõe `dare hooks list|run|validate` — execução determinística de hooks com trust gate (RS-05).

## Como usar

```bash
dare hooks list [--json]
dare hooks run <event> [--file --task --trust --json]
dare hooks validate [--json]
```

Repo clonado: `hooks.trusted` default `false` — requer `--trust` ou `trusted: true` para `run`.
