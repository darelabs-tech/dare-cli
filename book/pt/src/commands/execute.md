# `dare execute`

Executa uma ação sobre o grafo DAG — status, próxima task, watch em tempo real, ou transições de estado (complete/fail/reset).

## Uso

```bash
dare execute [OPTIONS]
```

## Ações (mutuamente exclusivas)

| Flag | Ação padrão | Descrição |
|---|---|---|
| *(nenhuma)* | `--status` | Exibe snapshot do estado atual das tasks |
| `--status` | — | Snapshot explícito |
| `--next` | — | Mostra próxima(s) task(s) pronta(s) com prompt |
| `--watch` | — | Modo watch: atualiza a cada `--interval` segundos |
| `--complete <ID>` | — | Marca task como DONE |
| `--fail <ID>` | — | Marca task como FAILED |
| `--reset <ID>` | — | Volta task para READY |

## Flags comuns

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--dag <PATH>` | path | `DARE/dare-dag.yaml` | Path do DAG (relativo ao root) |
| `--json` | bool | false | Saída em JSON estruturado |
| `--interval <SECS>` | u64 | `2` | Intervalo do watch em segundos |
| `--max-ticks <N>` | u64 | ilimitado | Máximo de ciclos do watch (útil em CI) |

## `dare execute` (status padrão)

```bash
dare execute                 # snapshot com status de todas as tasks
dare execute --status        # equivalente
```

### Saída humana

```
DAG: DARE/dare-dag.yaml   Tasks: 12 total (4 done, 2 running, 3 ready, 3 pending)
Canvas: .dare/canvas.md

ID          TITLE                       STATUS    RANK
─────────── ────────────────────────── ──────── ──────
task-001    Setup workspace            DONE         0
task-002    Modelos de dados           DONE         1
task-003    Migrations                 RUNNING      1
task-004    Autenticação JWT           READY        2
```

### Outcomes possíveis

| Outcome | Mensagem | Exit |
|---|---|---|
| Normal | tabela de tasks | `0` |
| DAG vazio | `Empty DAG — no tasks.` | `0` |
| Todas resolvidas | `✅ All tasks resolved.` | `0` |
| Bloqueado | `Blocked — no executable tasks` | `0` |

## `dare execute --next`

Mostra a(s) próxima(s) task(s) prontas para execução com prompt completo para o agente:

```bash
dare execute --next          # task pronta de menor rank
dare execute --next --json   # saída estruturada para scripts
```

### Formato de saída (humano)

```
Next task: task-004
Title:      Autenticação JWT
Complexity: HIGH
Rank:       2
Spec:       DARE/EXECUTION/task-004.md

## Upstream context

### From parent: task-002 — Modelos de dados
[últimas 2000 chars do output de task-002]

### From parent: task-003 — Migrations
[últimas 2000 chars do output de task-003]
```

### Saída JSON (`--next --json`)

```json
{
  "schemaVersion": 1,
  "outcome": "next_ready",
  "tasks": [
    {
      "id": "task-004",
      "title": "Autenticação JWT",
      "complexity": "HIGH",
      "rank": 2,
      "spec_file": "DARE/EXECUTION/task-004.md",
      "prompt": "..."
    }
  ]
}
```

## `dare execute --watch`

Modo de monitoramento contínuo — útil em paralelo com a execução de tasks:

```bash
dare execute --watch
dare execute --watch --interval 5     # atualiza a cada 5s
dare execute --watch --max-ticks 10   # para após 10 ciclos (CI)
```

- Zero writes no estado — apenas leitura periódica
- Cancelar com `Ctrl+C` → exit `0`

## Transições de estado

```bash
dare execute --complete task-004    # DONE
dare execute --fail task-004        # FAILED
dare execute --reset task-004       # READY
```

> Essas transições escrevem no `STATE_REL` (`.dare/state.json`). Use com atenção.

## Exit codes

| Código | Quando |
|---|---|
| `0` | OK — incl. empty, resolved, blocked |
| `1` | Erro interno |
| `2` | Uso inválido (flags mutuamente exclusivas, interval inválido) |
| `3` | `dare-dag.yaml` não encontrado |
| `4` | Input inválido, config YAML inválido, ou ciclo detectado no DAG |
| `5` | Erro de I/O (lock, write state/canvas) |
