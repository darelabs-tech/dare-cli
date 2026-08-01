---
name: dare-bench
description: Roda o harness determinístico de fixtures de verificação (Fix·Rate, solve-rate, baseline). Mapeia o CLI `dare bench`.
---

# DARE Bench — harness de verificação

Roda fixtures versionadas com patches golden/errados; mede Fix·Rate e solve-rate. Determinístico — sem LLM.

## Como rodar

```bash
dare bench --suite fixtures/bench --json
dare bench --suite fixtures/bench --baseline bench-baseline.json --fail-on-regression 3
```

## Exit codes

- `0` — ok (sem regressão vs baseline)
- `1` — regressão de solve-rate > limiar
- `2` — suite inválida
