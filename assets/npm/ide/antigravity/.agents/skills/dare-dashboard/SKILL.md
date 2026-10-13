---
name: dare-dashboard
description: Start the local read-only telemetry dashboard (dare dashboard).
---

# /dare-dashboard

Sobe o dashboard local de telemetria (`dare dashboard`) — read-only, loopback + token.

```bash
dare dashboard [--port <n>] [--no-open]
```

- Default: `127.0.0.1:4100`
- API: `GET /api/telemetry` (JSON read-only)
- UI: `GET /dashboard` (HTML/CSS/JS vanilla)

Reusa hardening do MCP (auth + bind loopback). Não muta grafo/estado.
