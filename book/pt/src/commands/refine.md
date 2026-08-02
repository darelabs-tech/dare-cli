# `dare refine`

Decompõe e refina uma task específica de alta complexidade (classificada como `HIGH` ou `CRITICAL`) em sub-tasks atômicas menores, reorganizando e validando dinamicamente o grafo de dependências no `dare-dag.yaml`.

## Uso

```bash
dare refine <TASK_ID> [OPTIONS]
```

## Flags

| Flag | Tipo | Descrição |
|---|---|---|
| `--apply` | bool | Efetiva a decomposição da task no arquivo `dare-dag.yaml` e no estado do DAG (sobrescreve e reescreve as conexões) |
| `--strict` | bool | Modo estrito: falha com exit code 2 caso a complexidade da task seja inferior a `HIGH` (não recomenda divisão) |
| `--json` | bool | Saída formatada em JSON estruturado |

---

## Como funciona a análise de complexidade?

O comando `dare refine` calcula uma pontuação de complexidade para a task alvo através da soma de pesos (heurística determinística):

```
score = 0
score += min(arquivos_afetados * 2, 10)       # Mapeado a partir da seção 3 da spec do EXECUTION
score += min(caracteres_do_prompt / 400, 6)  # Tamanho do prompt da task
score += min(dependencias.len(), 4)          # Quantidade de dependências
score += 3 por cada keyword pesada (máximo 9) # Ex.: "auth", "refactor", "migration", "security"
score += peso_base: LOW=0, MED=2, HIGH=4
```

### Classificação de Complexidade

| Pontuação | Nível | Recomendação de Divisão |
|---|---|---|
| 0 – 5 | `LOW` | Não recomendado |
| 6 – 11 | `MED` | Não recomendado |
| 12 – 17 | `HIGH` | **Recomendado** |
| ≥ 18 | `CRITICAL` | **Recomendado** |

---

## Efeito do Splice no DAG (`--apply`)

Quando `--apply` é executado, o DARE realiza uma reestruturação atômica do grafo:

1. **Splice de Subtasks:** A task pai é dividida em subtasks nomeadas sequencialmente no padrão kebab-case (ex: se o pai for `task-003`, as filhas serão `task-003-a`, `task-003-b`, `task-003-c`).
2. **Encadeamento Interno:** As novas subtasks são organizadas em cadeia de dependência sequencial (`task-003-a` -> `task-003-b` -> `task-003-c`).
3. **Re-fiação de Dependentes (Rewiring):** Qualquer task no DAG que dependia da task pai (`task-003`) passa a depender explicitamente da **última** subtask gerada (`task-003-c`), garantindo que o fluxo downstream só execute após o término de todo o escopo dividido.
4. **Atualização de Estado:** A task pai tem seu status marcado como `SPLIT` no arquivo `.dare/state.json` (preservando o histórico de tentativas), sendo removida da lista ativa de execução. As subtasks filhas são adicionadas com status `PENDING` ou `READY`.

> **Profundidade Máxima:** Para evitar explosão de subtasks, o DARE CLI define a constante `MAX_SUBDAG_DEPTH = 2`. Subtasks não podem ser refinadas novamente caso excedam esse limite.

---

## Exemplos de Uso

```bash
# Analisa e propõe o plano de refinamento para a task-003
dare refine task-003

# Aplica as modificações e divide a task no dare-dag.yaml
dare refine task-003 --apply

# Falha imediatamente se a task-003 for considerada simples (LOW ou MED)
dare refine task-003 --strict
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Sucesso — Análise exibida ou aplicação do splice realizada com sucesso |
| `1` | Falha na validação do DAG pós-splice (erro interno no rearranjo de dependências) |
| `2` | Uso da flag `--strict` e a task não atingiu a complexidade necessária (HIGH/CRITICAL) |
| `3` | A task especificada, o arquivo do DAG ou o projeto não foram encontrados |
| `4` | Entrada inválida, profundidade máxima atingida (`MAX_SUBDAG_DEPTH`) ou tentativa de refinar DAG legado |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
