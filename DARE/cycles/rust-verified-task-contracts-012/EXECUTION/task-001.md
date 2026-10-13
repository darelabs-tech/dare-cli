# Task 001: Golden baseline de `execute --next` / `--complete`

> **Complexity:** LOW · **depends_on:** — · **rank:** 0
> **Blueprint:** [§7 Fase 1](../BLUEPRINT.md) · **Aceite:** caso 8 e 11 · **Objetivo:** O-04

## Objetivo

Gravar o comportamento **atual** de `dare execute` como goldens commitados, **antes** de qualquer código de handoff existir. Sem este baseline, O-04 ("fluxos legados continuam operando") é uma afirmação sem referência de comparação.

## Por que é rank 0 e obrigatória

`task-011` compara a saída pós-implementação com estes snapshots. Se o baseline for gravado depois de qualquer mudança em `compose_task_prompt` ou em `execute.rs`, ele registra o comportamento já alterado e o teste passa trivialmente, provando nada.

## Arquivos a criar

```
crates/dare-cli/tests/fixtures/golden/handoff-baseline/
├── v21/dare-dag.yaml              DAG v2.1: task-a, task-b (rank 0), task-c ← [a, b]
├── legacy/dare-dag.yaml           DAG flat legacy
├── next-v21.stdout.txt            snapshot
├── next-v21.json.txt              snapshot
├── complete-v21.stdout.txt        snapshot
├── complete-v21.json.txt          snapshot
└── next-legacy.stdout.txt         snapshot

crates/dare-cli/tests/handoff_baseline.rs
```

## Especificação

O fixture v2.1 precisa exercitar o caminho de `## Upstream context`: `task-a` com `status: DONE` e `output` não vazio no `.dare/state.json` semeado, `task-c` dependendo dela. Sem isso o golden não cobre a composição, que é exatamente o que vai mudar.

Cenários gravados:

| Comando | O que ancora |
|---|---|
| `dare execute --next` (humano) | Prompt composto com cabeçalho de pai |
| `dare execute --next --json` | Envelope + campos de `ready` antes de ganharem `handoff` |
| `dare execute --complete task-a --json` | `complete_json` antes de ganhar `handoffPath`/`receipt` |
| `dare execute --next` no DAG legacy | `subtask_prompt` vazio, comportamento legacy |

Use `assert_cmd` + `predicates` (já dev-deps de `dare-cli`). Ralph via `DARE_RALPH_MOCK` ([`execute.rs:568`](../../../../crates/dare-cli/src/commands/execute.rs#L568)).

**Normalização mínima e justificada.** Normalize **apenas** `correlation_id` (UUID v4, muda a cada run) e paths absolutos do tempdir. Nada além disso — cada normalização extra é um lugar onde uma regressão real pode se esconder. Documente em comentário no teste por que cada uma existe.

## Validation Gates

- [ ] `cargo test -p dare-cli --test handoff_baseline` verde
- [ ] Snapshots são arquivos commitados, não strings inline no teste
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] **Nenhum arquivo em `src/` foi tocado** — `git diff --name-only` só mostra `tests/` e `fixtures/`

## Testes

```bash
cargo test -p dare-cli --test handoff_baseline
```

| Teste | Comportamento esperado |
|---|---|
| `baseline_next_v21_human` | stdout == `next-v21.stdout.txt`, exit 0 |
| `baseline_next_v21_json` | stdout == `next-v21.json.txt` após normalização, exit 0 |
| `baseline_complete_v21_json` | stdout == `complete-v21.json.txt`, exit 0 |
| `baseline_next_legacy_human` | stdout == `next-legacy.stdout.txt`, exit 0 |

## Segurança

Fixtures não contêm segredo, token ou path absoluto da máquina de quem gravou. O snapshot é lido e revisado em PR — qualquer valor sensível aqui vaza para o Git permanentemente.

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK` nos arquivos modificados
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`, `panic!("not implemented")`
- [ ] Nenhum `assert!(true)` nem teste sem assertion — cada teste compara contra um snapshot real
- [ ] Mocks apenas sob `tests/` (o `DARE_RALPH_MOCK` é harness oficial, documentado no Apêndice E)
- [ ] Snapshots gerados por execução real do binário, não escritos à mão
- [ ] `dare review task-001` sem findings

## Próxima Task

`task-011` consome estes snapshots. `task-002` e `task-004` rodam em paralelo com esta.
