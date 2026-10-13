# Comando: /dare-dashboard

Sobe o dashboard local de telemetria (`dare dashboard`) — read-only, loopback + token.

## Como rodar

```bash
dare dashboard [--port <n>] [--no-open]
```

- Default: `127.0.0.1:4100`
- Reusa auth/bind do MCP (token via `DARE_MCP_TOKEN` ou gerado na subida)
- Front vanilla em `/dashboard`; API JSON em `/api/telemetry`

## Notas

- **Read-only** — nenhuma rota muta grafo ou estado.
- Abre o navegador automaticamente (use `--no-open` para desabilitar).
