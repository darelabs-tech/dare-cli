# `dare bench`

Executa o harness determinístico de avaliação e benchmarks de qualidade e taxa de resolução (solve-rate) de correções do DARE sobre suites de verificação (fixtures).

## Uso

```bash
dare bench [OPTIONS]
```

## Flags

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--suite <PATH>` | path | `fixtures/bench` | Diretório contendo a suite de fixtures de teste |
| `--fail-on-regression <N>` | u32 | `0` | Limiar tolerado de regressão em pontos percentuais (0-100) no índice de `solveRate` comparado com o baseline anterior. Caso excedido, falha com exit code 1 |
| `--json` | bool | `false` | Saída formatada em JSON estruturado |

---

## Como funciona a avaliação de métricas?

O comando compara o estado do projeto (após aplicação de patches de correções) contra duas especificações dentro de cada fixture:

1. **`fail_to_pass.txt`:** Lista de IDs de testes que falhavam no legado e que **devem passar** na nova versão.
2. **`pass_to_pass.txt`:** Lista de IDs de testes que já passavam e **devem continuar passando** (garantia anti-regressão).

### Algoritmo de Cálculo de Taxa de Correção (Fix·Rate)

Para cada fixture avaliada:
- Se **qualquer** teste listado em `pass_to_pass.txt` falhar na execução atual:
  $$Fix\cdot Rate = 0.0$$
  O status da fixture é considerado `failed` (`fixtureOk = false`).
- Se todos os testes passantes se mantiverem estáveis:
  Seja $A$ o total de testes listados em `fail_to_pass.txt` e $B$ a quantidade de testes que agora passam com sucesso:
  - Se $A == 0$ e `fixtureOk` for verdadeiro: $$Fix\cdot Rate = 1.0$$
  - Caso contrário: $$Fix\cdot Rate = \frac{B}{A}$$

### Solve-Rate Geral

O `solveRate` geral do benchmark representa o percentual de fixtures avaliadas que obtiveram `fixtureOk = true` (ou seja, passaram em 100% de regressão e corrigiram as falhas previstas).

---

## Aspectos Avançados de Validação (`AdvancedAspect`)

Ao rodar o pipeline de execução e verificação, o DARE pode avaliar aspectos de qualidade adicionais (se ativados por configuração):

- **`fail-to-pass` / `anti-tamper`:** Validação contra adulteração de testes de suite.
- **`mutation` (Mutação):** Valida a cobertura de testes de mutação. Requer utilitário na PATH. Se a ferramenta estiver instalada, exige score de mutação mínimo de **0.70** (`MUTATION_THRESHOLD = 0.70`). Caso a ferramenta não exista na máquina e a flag `--full-mutation` esteja ativa, a verificação falha; caso contrário, é marcada como `skipped`.
- **`formal` (Formal):** Validação formal de código. Se ativado por opt-in (`verify.formal.enabled: true`), a ausência do backend formal na PATH falha imediatamente com o erro `FORMAL_TOOL_MISSING`.

---

## Exemplos de Uso

```bash
# Executa benchmark básico utilizando a suite padrão
dare bench

# Executa e falha caso haja regressão de mais de 5% em relação ao baseline
dare bench --fail-on-regression 5

# Executa e exibe o relatório de performance formatado em JSON
dare bench --json
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "solveRate": 85.0,
  "fixtures": [
    {
      "name": "auth-jwt-fixture",
      "ok": true,
      "fixRate": 1.0,
      "durationMs": 4520,
      "aspects": [
        {
          "aspect": "fail-to-pass",
          "status": "pass",
          "duration_ms": 1200
        },
        {
          "aspect": "mutation",
          "status": "skipped",
          "reason": "mutation_tool_missing",
          "duration_ms": 0
        }
      ]
    }
  ]
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Benchmark executado com sucesso e sem regressões fora do limiar |
| `1` | Queda de performance detectada acima da tolerância configurada em `--fail-on-regression` |
| `2` | Uso de argumentos inválidos |
| `3` | O diretório especificado em `--suite` não existe |
| `4` | Formato inválido nos arquivos de baseline ou nas especificações das fixtures |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
