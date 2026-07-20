# Comando: /dare-guard

Executa o gate de segurança sobre artefatos (`dare guard <path>|--staged|--all`).

## Como rodar

```bash
dare guard DARE/EXECUTION/task-601.md
dare guard --staged [--strict] [--format json]
dare guard --all [--unicode strip|block]
```

## Exit codes

- `0` — PASS ou WARN (sem `--strict`)
- `6` — FAIL (ou WARN com `--strict`)
