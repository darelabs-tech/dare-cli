---
name: dare-dag-runner
description: Constrói e executa o grafo DAG do método DARE com paralelismo lógico via Kahn's algorithm. Antigravity é o executor — usa o plano nativo da IDE; sem API key. O CLI dare é orquestrador (--next/--complete/--fail).
---

# DARE DAG Runner Skill

Você é o executor da fase E (Execute) do método DARE no Antigravity. O CLI
`dare` é o **orquestrador**: ele indica quais tasks executar agora e registra
o que você terminou. **Você** é quem efetivamente roda cada task usando o
runtime nativo do Antigravity — não há API key nem custo extra de tokens.

## Quando usar esta skill

- BLUEPRINT.md está aprovado e é hora de gerar tasks
- Existe `DARE/dare-dag.yaml` e você precisa entender, executar ou modificar
- Aparece o canvas `DARE/.canvas.md` durante uma execução
- Usuário pede "executa o DAG" ou "começa o execute"

## Modelo de execução

> **Antigravity é o executor. O CLI `dare` é orquestrador.**

- A IDE já está autenticada
- Você lê `dare-dag.yaml` e as specs em `DARE/EXECUTION/task-*.md`
- Você executa cada task — escreve código, roda testes, faz lint
- Após cada task, registra o resultado no CLI:
  - `dare execute --complete <task-id> --output "<resumo>"`
  - `dare execute --fail <task-id> --reason "<mensagem>"`
- O CLI atualiza `DARE/.canvas.md` e popula o `dare-graph` automaticamente

## O que é o DAG do DARE

`DARE/dare-dag.yaml` é o **plano de execução** da fase E. Grafo direcionado
acíclico:

- **Nó** = uma task atômica
- **Aresta** = `depends_on` (filha precisa do output da pai)

O CLI ordena topologicamente (Kahn's algorithm). Tasks no mesmo rank podem
rodar em paralelo (logicamente — você decide se literalmente fan-out ou roda
uma após a outra).

```
rank 0  ─→  task-001  task-002       (paralelas)
rank 1  ─→  task-003 (deps: 001,002)
rank 2  ─→  task-004 (deps: 003)
```

## Schema canônico

```yaml
title: "<Nome do projeto> - Development Tasks"
version: "1.0.0"

limits:
  parent_context_chars: 2000   # snippet de output de pai injetado no filho
  task_output_chars: 4000      # cap do output capturado por task
  timeout_seconds: 600         # apenas referência

models:
  cursor:      { HIGH: gpt-5.3-codex,     MED: composer-2,       LOW: auto-low }
  claude:      { HIGH: claude-sonnet-4-5, MED: claude-haiku-4,   LOW: claude-haiku-4 }
  antigravity: { HIGH: gemini-2.5-pro,    MED: gemini-2.5-flash, LOW: gemini-2.5-flash }

tasks:
  - id: task-001
    title: "Setup project structure"
    depends_on: []
    complexity: LOW
    spec_file: EXECUTION/task-001.md
    subtask_prompt: |
      <prompt completamente self-contained>
```

## Loop de execução

```
1. dare execute --next
   ↓ imprime prompts das tasks ready (rank atual)
2. Para cada prompt:
     - leia spec_file se houver
     - implemente
3. dare execute --complete <id> --output "<resumo + arquivos tocados>"
     ↓ o CLI roda o RALPH LOOP automático (build → test → lint)
     ↓ se passar: task vira DONE
     ↓ se falhar: task vira FAILED com stderr capturado; corrija e retente
4. Volte ao passo 1 até não haver mais tasks ready
```

> **Ralph Loop é AUTOMÁTICO e OBRIGATÓRIO.** Você NÃO roda build/test/lint
> manualmente — o `dare execute --complete` faz isso. Se algum gate falhar,
> a task NÃO vai para DONE; vai para FAILED. Corrija e retente.
>
> **Não existe flag para pular o Ralph Loop.** Toda task passa pelos 3 gates
> da stack do projeto.

Comandos úteis:

```bash
dare execute --next                                # próximas tasks ready
dare execute --complete task-001 --output "..."    # marca DONE
dare execute --fail task-002 --reason "..."        # marca FAILED + cascade
dare execute --reset task-002                      # volta para PENDING (retry)
dare execute --status                              # snapshot do canvas
```

## Regras inegociáveis ao construir o DAG

### 1. `id` em kebab-case e único
`task-001`, `auth-jwt`, `db-migrations`.

### 2. `depends_on` mínimo
Só adicione dependência quando a filha **literalmente** precisa do output.

| Cenário | Dep? |
|---------|------|
| B precisa do arquivo de A | sim |
| B precisa de decisão de A | sim |
| B é independente de A | não |
| Testes/Docs do módulo X | sim — depende da implementação |
| Pesquisa sem efeito colateral | não |

Cadeia linear é antipattern. Reanalise.

### 3. `complexity` é sinal de cuidado, não custo
| Nível | Uso |
|-------|-----|
| `LOW`  | Setup, scaffolding, docs simples |
| `MED`  | Implementação direta, refactors, testes unitários |
| `HIGH` | Lógica crítica, segurança, integrações |

### 4. `subtask_prompt` self-contained
Receberá `subtask_prompt` + snippets de até 2000 chars dos outputs dos pais.
Não vale "como combinamos". Tudo no prompt ou via pais.

### 5. Output cap 4000 chars
Se gerar muito, escreva em arquivo e faça o `--output` ser resumo + caminhos.

### 6. Spec por task em `EXECUTION/task-<id>.md`
Spec detalhada com objetivo, arquivos, validation gates, testes, segurança.

## Os 3 artefatos sempre juntos

| Arquivo | Para quê |
|---------|----------|
| `DARE/TASKS.md` | Visão humana com tabela e progresso |
| `DARE/dare-dag.yaml` | Grafo executável |
| `DARE/EXECUTION/task-<id>.md` | Spec por task |

Os três precisam ser consistentes: mesmos `id`s, `depends_on`, `complexity`.

## Canvas ao vivo (`DARE/.canvas.md`)

O CLI reescreve a cada `--complete`/`--fail`:

| Status | Significado |
|--------|-------------|
| `PENDING` ⏳ | aguardando rank |
| `RUNNING` 🔄 | você está executando |
| `DONE` ✅ | concluído |
| `FAILED` ❌ | erro durante execução |
| `SKIPPED` ⏭️ | dependência falhou — automático, não toque |

## Erros comuns

| Erro | Correção |
|------|----------|
| Ciclo | Retire a aresta cíclica |
| `id` duplicado | Renomeie |
| `depends_on` inexistente | Corrija ou adicione a task |
| Tudo em rank 0 | Adicione deps reais quando há contenção |
| Cadeia linear | Reveja se as deps são necessárias |

## Antipatterns que você NÃO deve criar

- ❌ Task **"Ralph Loop final"** / **"Hardening"** / **"QA"** — gate é por task
- ❌ Tests com `assertTrue(true)` — o gate `test` roda de verdade
- ❌ "Setup project structure" antes de containerizar o app

## Ordem recomendada das primeiras tasks

1. **Containerize app** (Dockerfile + docker-compose + healthcheck)
2. **Database schema** (migrations + factories)
3. **Core endpoints / componentes**
4. **Auth / autorização**
5. **Test suite real** (assertions de verdade)

A task de container/compose pode estar em outra ordem para projetos sem
DB ou em monorepo já containerizado. Mas quase sempre é uma das primeiras.

## Checklist antes de aprovar

- [ ] Pelo menos 2 tasks no rank 0
- [ ] `subtask_prompt` executável sem contexto adicional
- [ ] Tasks de teste/doc dependem da implementação correspondente
- [ ] `complexity` reflete o esforço real
- [ ] `id` em kebab-case e único
- [ ] Sem ciclos
- [ ] **Sem task de "Ralph Loop final"** — gate é por task
- [ ] **Tests com assertions reais** — placeholder quebra o gate
- [ ] Container/runtime resolvido cedo
- [ ] Os 3 artefatos consistentes
