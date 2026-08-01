---
name: dare-dag-build
description: Regenera APENAS o DARE/dare-dag.yaml a partir do DARE/BLUEPRINT.md existente, sem refazer o blueprint nem as specs. Use quando o BLUEPRINT mudou pouco mas o grafo precisa refletir o novo estado, ou quando você quer experimentar uma decomposição diferente.
---

# DARE DAG Build Skill

Você é o construtor de grafos da fase EXECUTE do método DARE no Antigravity. Esta skill **regenera APENAS** o `DARE/dare-dag.yaml` — não refaz o BLUEPRINT, não refaz as specs, não roda nada. Apenas reconstrói o grafo de tasks a partir do estado atual do BLUEPRINT.

> Se você precisa **executar** o grafo, use `dare-dag-run`.
> Se você precisa **visualizar** o grafo, use `dare-dag-viz`.
> Se você precisa **fazer tudo de uma vez** (build + run), use `dare-dag-runner`.

## Quando usar esta skill

- O BLUEPRINT foi ajustado e o grafo precisa refletir as mudanças
- Você quer experimentar uma decomposição diferente sem refazer o blueprint
- O `dare-dag.yaml` ficou inconsistente com `EXECUTION/task-*.md`
- Precisa adicionar/remover/reordenar tasks no grafo
- A complexidade ou as dependências de tasks mudaram

## Pré-requisitos

- `DARE/BLUEPRINT.md` existe e está aprovado
- (Opcional) `DARE/EXECUTION/task-*.md` específicas — serão preservadas se não forem mencionadas

## Como usar

### Passo 1 — Ler `DARE/BLUEPRINT.md`

Obrigatório. Se faltar, peça `dare-blueprint` antes.

### Passo 2 — Ler `DARE/dare-dag.yaml` atual (se existir)

Para preservar `id`s já em uso e não confundir o usuário com renomeações desnecessárias.

### Passo 3 — Gerar o novo `dare-dag.yaml`

Schema canônico:

```yaml
title: "<Nome do Projeto> - Development Tasks"
version: "1.0.0"

limits:
  parent_context_chars: 2000
  task_output_chars: 4000
  timeout_seconds: 600

models:
  cursor:      { HIGH: gpt-5.3-codex,     MED: composer-2,       LOW: auto-low }
  claude:      { HIGH: claude-sonnet-4-5, MED: claude-haiku-4,   LOW: claude-haiku-4 }
  antigravity: { HIGH: gemini-2.5-pro,    MED: gemini-2.5-flash, LOW: gemini-2.5-flash }

tasks:
  - id: task-001
    title: "..."
    depends_on: []
    complexity: LOW
    spec_file: EXECUTION/task-001.md
    subtask_prompt: |
      <self-contained>
```

**Regras inegociáveis:**

- `id` em kebab-case e único
- `depends_on` mínimo — só quando filha *literalmente* precisa do output do pai
- `subtask_prompt` self-contained (o subagente recebe apenas isso + 2000 chars dos pais)
- Pelo menos **2 tasks no rank 0** (paralelismo desde o início)
- Cadeia linear é antipattern — quebre dependências sempre que possível
- `complexity` honesta — não inflar nem deflar

### Passo 4 — ANTI-STUB CONTRACT (inegociável)

Tasks geradas com `subtask_prompt` ou spec genéricos forçam o agente a inventar — e ele vai produzir mock, stub ou esqueleto. **O comando `dare-review` detecta isso e marca a task como FAILED.**

Cada `subtask_prompt` e `EXECUTION/task-<id>.md` deve atender:

**O `subtask_prompt` precisa ser auto-suficiente.** Inclua:
- Caminho exato dos arquivos a criar/modificar
- Assinaturas exatas das funções/endpoints (`fn name(params: T) -> R`)
- Schema de request/response com tipos
- Validações específicas (não "validar input" — `email: regex /^.../`, `senha: ≥ 8 chars + 1 maiúscula + 1 dígito`)
- Edge cases enumerados (input vazio, duplicado, expirado, sem permissão)
- Lista de testes esperados com nome + comportamento

**A `spec_file` (`EXECUTION/task-<id>.md`) precisa ter Definition of Done anti-stub:**

```markdown
## Definition of Done (ANTI-STUB)

- [ ] Nenhum TODO, FIXME, XXX ou HACK em arquivos modificados
- [ ] Nenhuma função vazia (fn x() {}, def x(): pass, function x() {})
- [ ] Nenhum throw new Error('not implemented'), unimplemented!(), todo!(), NotImplementedError
- [ ] Nenhum return null/undefined/{} como única statement de função pública
- [ ] Mocks SOMENTE em *.test.*, *.spec.*, __tests__/, tests/, spec/ — NUNCA em código de produção
- [ ] Todos os endpoints retornam dados reais (não hardcoded)
- [ ] Cada validação produz erro real com status code correto (testado)
- [ ] Cada edge case tem teste demonstrando comportamento
```

**Sinais de spec rasa (auto-validar antes de salvar):**

- ❌ "Implementar X" — sem assinatura, sem retorno, sem validações
- ❌ "Tratar erros adequadamente" — quais erros? como? que código?
- ❌ "Adicionar validações" — quais regras?
- ✅ "Implementar `POST /auth/login` retornando `{ token: string, refresh: string }` com 200 se credenciais válidas, 401 se inválidas, 429 se rate limit"

### Passo 5 — Validar consistência com `EXECUTION/task-*.md`

Se uma spec já existe em `DARE/EXECUTION/task-<id>.md`:
- Mantenha o mesmo `id` no YAML
- Aponte `spec_file: EXECUTION/task-<id>.md`
- Se a `complexity` ou `depends_on` mudou, atualize **também** a spec e o `TASKS.md`

Se uma task nova entrou no YAML mas não tem spec:
- Crie a spec correspondente em `EXECUTION/task-<id>.md`

Se uma task saiu do YAML mas a spec ficou:
- Pergunte ao usuário: arquivar (mover para `EXECUTION/_archived/`) ou apagar?

### Passo 6 — Validar grafo

- Sem ciclos (DAG = Directed Acyclic Graph)
- Todos os `depends_on` apontam para `id`s existentes
- `id`s únicos
- Pelo menos 2 tasks no rank 0

### Passo 7 — Atualizar `DARE/TASKS.md`

Reflita o novo grafo na tabela master.

### Passo 8 — Mensagem ao usuário

> `dare-dag.yaml` regenerado:
> - Total de tasks: N
> - Ranks paralelos: N
> - Adicionadas: [...]
> - Removidas: [...]
> - Modificadas: [...]
>
> Revise e aprove. Para executar: invoque a skill `dare-dag-run`.

## Quando NÃO usar esta skill

- Se você nunca rodou `dare-blueprint` antes — use `dare-blueprint` primeiro
- Se o BLUEPRINT está desatualizado — atualize o BLUEPRINT primeiro
- Se você só quer executar o grafo já aprovado — use `dare-dag-run`
- Se você quer build + run num único passo — use `dare-dag-runner`

## Licença

Esta skill é parte do DARE Method e está sob licença MIT (D-001).
