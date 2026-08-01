---
name: dare-dag-run
description: Executa o grafo de tasks definido em DARE/dare-dag.yaml usando Antigravity como executor e o CLI dare como orquestrador. Sem API keys — usa o plano nativo da IDE. O canvas ao vivo fica em DARE/.canvas.md.
---

# DARE DAG Run Skill

Você é o executor da fase EXECUTE do método DARE no Antigravity. Esta skill **executa APENAS** o grafo que já foi construído — não regenera o `dare-dag.yaml`, não modifica o BLUEPRINT, não cria specs novas. Apenas roda as tasks na ordem topológica correta.

> Se você precisa **(re)construir** o grafo, use `dare-dag-build`.
> Se você precisa **visualizar** o grafo, use `dare-dag-viz`.
> Se você precisa **fazer tudo** (build + run num passo), use `dare-dag-runner`.

> **Sem API keys.** Você (Antigravity) usa o plano nativo da IDE. O CLI `dare` apenas coordena estado, monta prompts e atualiza o canvas.

## Quando usar esta skill

- `DARE/dare-dag.yaml` está aprovado e pronto
- As specs em `DARE/EXECUTION/task-<id>.md` estão geradas
- É hora de implementar as tasks na ordem do grafo
- O usuário aprovou os ANTI-STUB contracts

## Pré-requisitos

- `DARE/dare-dag.yaml` existe e foi aprovado pelo usuário
- Specs em `DARE/EXECUTION/task-<id>.md` geradas (se não, use `dare-dag-build`)
- `dare` disponível no PATH (`npm i -g @dewtech/dare-cli`)

## Como usar

```
Invoque a skill com:
- (sem args) — executa todas as ready do próximo rank
- --task task-003 — executa só uma task específica
```

### Passo 1 — Validar pré-condições

- Confirme que `DARE/dare-dag.yaml` existe. Se não, oriente o usuário a invocar `dare-blueprint` ou `dare-dag-build`
- Leia o YAML e verifique: sem ciclos, ids únicos, ranks paralelizáveis
- Liste rapidamente para o usuário: total de tasks, ranks, próximas ready

### Passo 2 — Pegar próximas tasks

```bash
dare execute --next
```

O CLI imprime as tasks **ready** do rank atual com o prompt completo (snippets de até 2000 chars dos outputs dos pais já costurados). Use esses prompts diretamente — não reescreva.

### Passo 3 — Sugerir abrir o canvas

Antes de começar, peça ao usuário abrir `DARE/.canvas.md` em outra aba do Antigravity. Esse é o feedback visual ao vivo da execução.

### Passo 4 — Executar cada task

Para cada task ready:

1. Leia `spec_file` se houver (`DARE/EXECUTION/task-<id>.md`)
2. Implemente conforme o `subtask_prompt` — **não invente, não use mock fora de tests, sem TODOs**
3. Rode o Ralph Loop completo: build → test → lint
4. Registre o resultado no CLI:

```bash
# Sucesso
dare execute --complete task-001 --output "Resumo curto + arquivos criados/modificados (paths)"

# Falha
dare execute --fail task-002 --reason "Mensagem clara da falha (compilação? teste? lint?)"
```

### Passo 5 — Avançar de rank

```bash
dare execute --next
```

- Se aparecer `✅ All tasks resolved`, todas terminaram. Reporte o resumo.
- Caso contrário, continue executando o próximo rank.

### Passo 6 — Pós-execução

```bash
dare execute --status
```

Mostra snapshot do canvas + sumário (X DONE, Y FAILED, Z SKIPPED).

Para retentar tasks FAILED:

```bash
dare execute --reset task-002
dare execute --next
```

## Comandos do orquestrador `dare`

| Comando | Função |
|---------|--------|
| `dare execute --next` | Próximas tasks ready com prompts compostos |
| `dare execute --complete <id> --output "..."` | Marca DONE |
| `dare execute --fail <id> --reason "..."` | Marca FAILED + cascade-skip |
| `dare execute --reset <id>` | Volta para PENDING |
| `dare execute --status` | Snapshot do canvas + sumário |

## Erros comuns

| Sintoma | Causa | Correção |
|---------|-------|----------|
| `dare-dag.yaml not found` | Não foi gerado | Invoque `dare-blueprint` ou `dare-dag-build` |
| Cascata de SKIPPED | Pai falhou | Corrija pai → `dare execute --reset` → `--next` |
| Output truncado | Cap de 4000 chars | Escreva em arquivo, retorne resumo |
| Spec inconsistente | TASKS.md ≠ dare-dag.yaml | Re-gere com `dare-dag-build` |

## Referências

- Schema: `DARE/dare-dag.yaml`
- Canvas: `DARE/.canvas.md`
- Specs: `DARE/EXECUTION/task-*.md`
- Status: `DARE/TASKS.md`

## Quando NÃO usar esta skill

- Se `dare-dag.yaml` não existe — use `dare-blueprint` ou `dare-dag-build`
- Se você quer só visualizar o grafo — use `dare-dag-viz`
- Se você quer build + run num único passo — use `dare-dag-runner`

## Licença

Esta skill é parte do DARE Method e está sob licença MIT (D-001).
