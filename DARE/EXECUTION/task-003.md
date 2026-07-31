# TASK 003: Containerização governance (Docker + Compose)

> **Complexidade:** MED  
> **Depends on:** task-001, task-002  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, `docker compose -f docker-compose.governance.yml up --build -d` sobe o serviço `governance-check` com healthcheck verde baseado em `verify-structure.mjs`.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 1 (sempre primeira containerização)
- **Decisões:** healthcheck = CMD exec (sem HTTP); Node 20 slim

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `Dockerfile.governance` | multi-stage opcional; slim suficiente |
| CRIAR | `docker-compose.governance.yml` | serviço governance-check |
| CRIAR | `.env.governance.example` | NODE_VERSION=20; GOVERNANCE_TARBALL_PATH comentado |

---

## 4. IMPLEMENTAÇÃO

### Dockerfile.governance

```dockerfile
FROM node:20-bookworm-slim
WORKDIR /repo
COPY docs ./docs
COPY scripts/governance ./scripts/governance
WORKDIR /repo
CMD ["node", "scripts/governance/verify-all.mjs"]
```

### docker-compose.governance.yml

```yaml
services:
  governance-check:
    build:
      context: .
      dockerfile: Dockerfile.governance
    healthcheck:
      test: ["CMD", "node", "scripts/governance/verify-structure.mjs"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s
```

### Testes esperados

- [ ] `docker compose ... config` valida YAML
- [ ] `docker compose ... up --build -d` → container healthy
- [ ] Edge: Docker ausente → falha documentada + YAML ainda válido (não marcar DONE sem tentativa real se Docker existir)

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Imagem oficial Node; sem secrets no Dockerfile
- [ ] `.env.governance.example` só nomes de vars
- [ ] Não montar credenciais do host

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
docker compose -f docker-compose.governance.yml config
docker compose -f docker-compose.governance.yml up --build -d
docker compose -f docker-compose.governance.yml ps
# health = healthy
docker compose -f docker-compose.governance.yml down
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Healthcheck não pode ser `CMD true` / `exit 0` vazio
- [ ] Sem `TODO` no Dockerfile

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Serviço healthy observado
- [ ] `DARE/TASKS.md`: task-003 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

Aguarda rank: `task-013` (após 012)
