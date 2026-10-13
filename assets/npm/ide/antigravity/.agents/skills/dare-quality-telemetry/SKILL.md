---
name: dare-quality-telemetry
description: Coleta de métricas de qualidade e detecção de regressões em projetos DARE. Agrega métricas das skills filhas (dare-ax, dare-layered-design, etc.), persiste histórico em tmp/dare_metrics.json e detecta regressões contra baseline. Inclui template de GitHub Actions.
---

# DARE Quality Telemetry Skill

Você é um engenheiro de plataforma especialista em observabilidade de qualidade. Seu papel é garantir que todo projeto DARE colete métricas das skills aplicadas, persista histórico, e detecte regressões antes do release.

## Quando usar esta skill

- Projeto já tem várias skills DARE configuradas (ax, layered-design, llm-integration, etc.)
- Você quer ver evolução de métricas ao longo do tempo
- Você quer falhar o CI quando uma métrica regredir
- Você quer dashboards/reports de saúde do projeto

## O que essa skill faz

1. **Coleta** — roda os collectors de cada skill filha em paralelo
2. **Agrega** — junta todas as métricas em um snapshot estruturado
3. **Persiste** — guarda em `tmp/dare_metrics.json` (histórico append-only)
4. **Compara** — diff contra `tmp/dare_metrics_baseline.json`
5. **Reporta** — gera markdown report para CI ou stdout
6. **Bloqueia** — exit code ≠ 0 se regressão crítica

## Arquitetura

```
┌──────────────────────────────────────────────────────────┐
│  collect.ts (orquestrador)                                │
└──────────────────────────────────────────────────────────┘
            ↓ chama em paralelo
┌────────────────┬────────────────┬────────────────┐
│ ax collector   │ layered coll.  │ llm coll.      │  …
└────────────────┴────────────────┴────────────────┘
            ↓
┌──────────────────────────────────────────────────────────┐
│  Aggregator → snapshot { skill: { M-01: value, ... } }    │
└──────────────────────────────────────────────────────────┘
            ↓
┌──────────────────────────────────────────────────────────┐
│  tmp/dare_metrics.json (append) + Regression detector     │
└──────────────────────────────────────────────────────────┘
            ↓
┌──────────────────────────────────────────────────────────┐
│  Reporter → markdown / json / GitHub Action summary       │
└──────────────────────────────────────────────────────────┘
```

## Métricas obrigatórias da própria skill

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | 100% dos builds incluem coleta de métricas (skill instalada = sempre true) | Verifica `package.json` ou pipeline yml |
| M-02 | 0 regressões passam despercebidas (baseline existe para comparação) | `test -f tmp/dare_metrics_baseline.json` |
| M-03 | Histórico de métricas mantido | `test -f tmp/dare_metrics.json` |
| M-04 | Workflow GitHub Actions existe | `test -f .github/workflows/dare-metrics.yml` |

## Como aplicar

### Passo 1: Instalar a skill no projeto

```bash
# Dentro do projeto
pnpm add @dare/quality-telemetry
# ou
npm install @dare/quality-telemetry
```

### Passo 2: Configurar quais skills rastrear

```yaml
# dare.config.yml
telemetry:
  skills:
    - dare-ax
    - dare-layered-design
    - dare-llm-integration
    - dare-frontend-design
    - dare-realtime
  output: tmp/dare_metrics.json
  baseline: tmp/dare_metrics_baseline.json
  fail_on_regression: true
```

### Passo 3: Rodar collect

```bash
dare metrics collect
# saída:
#   ✓ dare-ax        M-01:1.0  M-02:1.0  M-03:1.0  M-04:1.0
#   ✓ dare-layered   M-01:0.95 M-02:1.0  M-03:1.0  M-04:1.0
#   ✓ dare-llm       M-01:1.0  M-02:0.92 M-03:1.0  M-04:1.0
```

### Passo 4: Capturar baseline

```bash
dare metrics baseline
# copia tmp/dare_metrics.json → tmp/dare_metrics_baseline.json
```

### Passo 5: Adicionar workflow CI

```yaml
# .github/workflows/dare-metrics.yml
name: DARE Metrics

on: [push, pull_request]

jobs:
  metrics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: pnpm install
      - run: pnpm dare metrics collect
      - run: pnpm dare metrics compare
        # exit 1 se regressão crítica detectada
      - uses: actions/upload-artifact@v4
        with:
          name: dare-metrics
          path: tmp/dare_metrics*.json
```

## Schema do snapshot

```json
{
  "timestamp": "2026-05-26T10:30:00Z",
  "commit": "1d4a1417",
  "branch": "main",
  "skills": {
    "dare-ax": {
      "M-01": { "value": 1.0, "status": "PASS" },
      "M-02": { "value": 1.0, "status": "PASS" },
      "M-03": { "value": 1.0, "status": "PASS" },
      "M-04": { "value": 1.0, "status": "PASS" }
    },
    "dare-layered-design": {
      "M-01": { "value": 0.95, "status": "PASS" },
      "M-02": { "value": 1.0, "status": "PASS" },
      "M-03": { "value": 1.0, "status": "PASS" },
      "M-04": { "value": 1.0, "status": "PASS" }
    }
  }
}
```

## Detecção de regressão

Para cada métrica, comparar valor atual com baseline:

| Mudança | Severidade | Ação |
|---|---|---|
| value caiu ≥ 5% | CRITICAL | bloquear PR |
| value caiu < 5% mas ≥ 1% | WARNING | comentar no PR, não bloquear |
| value subiu | INFO | celebrar |
| value igual | NEUTRAL | nenhuma |

## Antipatterns

| AP | Antipattern | Por que evitar |
|---|---|---|
| AP-01 | Coletar métricas sem baseline | Não dá pra detectar regressão |
| AP-02 | Histórico não versionado | Perde contexto histórico |
| AP-03 | Coletar mas não bloquear regressão | CI vira teatro |
| AP-04 | Baseline desatualizado | Regressão real fica invisível |

## Boas práticas

1. **Baseline atualizado a cada release** — não a cada commit
2. **Regressão = bloqueio de release**, não comentário ignorável
3. **Métricas exportadas para Datadog/Grafana** após N coletas, para visualizar tendência
4. **Skill deve rodar em < 60s** — se demorar, paralelizar collectors

## Dicas

- **Use** o template `packages/skills/dare-quality-telemetry/github_actions_template.ts` para gerar `.github/workflows/dare-metrics.yml`
- **Combine** com `dare-ax` (M-01 a M-04 dela são coletadas aqui)
- **Estenda** criando collectors custom para métricas específicas do projeto (cobertura de testes, build time, etc.)

---

Esta skill é parte do DARE Method e está sob licença MIT.
