# Fase 4 — Execute

A fase Execute é onde **a IA implementa as tasks com auto-correção automática** via Ralph Loop.

## Objetivo

Implementar cada task do `DARE/TASKS.md` com código real, testes funcionais e gates de qualidade passando.

## Comando

```bash
# Executar uma task específica
dare execute task-001

# Ver próxima task disponível
dare dag next

# Ver status de todas as tasks
dare dag status
```

## Fluxo de execução

```
dare execute task-001
      │
      ▼
  Carrega spec  (DARE/EXECUTION/task-001.md)
      │
      ▼
  Implementa código
      │
      ▼
  Roda Gates ──── Fail ──► Lê erro ──► Corrige ──┐
      │                                           │
      OK                                          │
      │                                           └──► (volta a Roda Gates)
      ▼
  dare dag complete task-001
      │
      ▼
  Próxima task desbloqueada no DAG
```

## Gates de Validação

Os gates são executados automaticamente pelo Ralph Loop após cada implementação:

| Gate | Comando (Rust) | Critério |
|---|---|---|
| Testes | `cargo test` | Exit 0 |
| Linter | `cargo clippy -- -D warnings` | Sem warnings |
| Formatter | `cargo fmt --check` | Sem diff |
| Audit | `cargo audit` | Sem CVEs críticos |

## Estados de uma task

| Estado | Descrição |
|---|---|
| `pending` | Aguardando dependências |
| `ready` | Dependências satisfeitas, pronta para execução |
| `running` | Em execução pelo agente |
| `done` | Implementada e gates passando |
| `failed` | Falhou após esgotar tentativas |

## Comandos do DAG

```bash
dare dag next              # próxima task ready
dare dag status            # tabela de status de todas as tasks
dare dag complete task-001 # marcar task como done
dare dag fail task-001     # marcar task como failed
dare dag reset task-001    # voltar task para ready
dare dag watch             # modo watch (atualiza a cada 2s)
```

## Budget de tokens

Para controlar o custo da execução:

```bash
dare execute task-001 --budget 50000   # limite de 50k tokens
```

## Próximo passo

Após completar todas as tasks, rode o review final:

```bash
dare review
```
