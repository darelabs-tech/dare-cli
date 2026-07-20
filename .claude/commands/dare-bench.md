# /dare-bench

Roda o harness determinístico de fixtures de verificação (`dare bench`) — mede Fix·Rate e solve-rate contra patches versionados.

> Este comando expõe o CLI `dare bench` na IDE.

## Como rodar

```bash
dare bench --suite fixtures/bench --json
dare bench --suite fixtures/bench --baseline bench-baseline.json --fail-on-regression 3
```

## O que fazer

1. Rode `dare bench` com `--json` para relatório estruturado.
2. Compare com `--baseline` em CI; exit 1 = regressão > limiar pp.
3. Exit 2 = suite inválida/ausente.
