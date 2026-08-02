# DAG Runner

O **DAG Runner** é a engine de execução de tasks paralelas do DARE CLI, implementada em Rust com o **algoritmo de Kahn** (topological sort).

## O que é um DAG?

Um DAG (Directed Acyclic Graph) é um grafo onde as tasks são nós e as dependências são arestas direcionadas, sem ciclos. Isso permite executar tasks em paralelo quando não há dependência entre elas.

```
task-001 (setup)
    │
    ├──► task-002 (models)      ├──► task-005 (auth)
    │                           │
    └──► task-003 (database) ───┤──► task-006 (api)
                                │
                                └──► task-007 (tests)
```

Tasks `task-002`, `task-003` podem rodar em paralelo após `task-001`.

## Arquivo `dare-dag.yaml`

```yaml
version: "1"
tasks:
  - id: task-001
    title: "Setup do workspace Cargo"
    depends_on: []
    status: done

  - id: task-002
    title: "Modelos de dados"
    depends_on: [task-001]
    status: ready

  - id: task-003
    title: "Migrations do banco"
    depends_on: [task-001]
    status: ready

  - id: task-005
    title: "Autenticação JWT"
    depends_on: [task-002, task-003]
    status: pending
```

## Algoritmo de Kahn

O algoritmo de Kahn funciona assim:

1. Calcular o **grau de entrada** (in-degree) de cada nó
2. Colocar todos os nós com grau zero na fila de prontos (`ready`)
3. Processar um nó por vez: removê-lo do grafo e decrementar o grau dos seus dependentes
4. Quando um dependente chega a grau zero, entra na fila
5. Se o grafo não se esgotar (ciclo detectado), o `dare validate` reporta erro

## Comandos

```bash
dare dag next              # próxima task ready (para agentes)
dare dag status            # tabela de status
dare dag visualize         # exibe grafo no terminal
dare dag visualize --mmd   # exporta formato Mermaid
dare dag complete <id>     # marca task como done
dare dag fail <id>         # marca task como failed
dare dag reset <id>        # volta task para ready
dare dag watch             # modo watch (atualiza a cada 2s)
```

## `dare dag status` — exemplo de saída

```
Task ID      Title                      Status    Depends on
──────────── ─────────────────────────  ────────  ──────────
task-001     Setup workspace            DONE      —
task-002     Modelos de dados           DONE      task-001
task-003     Migrations                 RUNNING   task-001
task-004     Autenticação JWT           PENDING   task-002, task-003
task-005     Testes de integração       PENDING   task-004
```

## `--json` para agentes

```bash
dare dag next --json
```

```json
{
  "schemaVersion": 1,
  "next": {
    "id": "task-003",
    "title": "Migrations do banco",
    "spec_path": "DARE/EXECUTION/task-003.md"
  }
}
```

## Performance

O algoritmo de Kahn em Rust processa grafos de até 1.000 tasks em < 1ms. A redução de tempo de execução total com paralelismo chega a **75%** em projetos típicos.

## Validação do grafo

Antes de executar, o grafo é sempre validado:

```bash
dare validate   # detecta ciclos, referências quebradas, campos obrigatórios
```

Pode ser usado como pre-commit hook ou em CI.
