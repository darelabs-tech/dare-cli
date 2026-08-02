# `dare blueprint`

Gera o `DARE/BLUEPRINT.md` com arquitetura técnica completa a partir do `DARE/DESIGN.md` aprovado. Também produz `DARE/TASKS.md`, `DARE/dare-dag.yaml` e `DARE/EXECUTION/task-*.md`.

## Uso

```bash
dare blueprint [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--ai` | Enriquece o BLUEPRINT.md via LLM após geração determinística |
| `--provider <PROVIDER>` | Provider de IA (requer `--ai`; padrão: `codex`) |
| `--json` | Saída em JSON estruturado |

## Fluxo de geração

```
dare blueprint
      │
      ▼
  find_project_root()
      │
      ▼
  Lê DARE/DESIGN.md  ──── ausente? → exit 3 (NotFound)
      │
      ▼
  Gera bundle determinístico
  (BLUEPRINT.md + TASKS.md + dare-dag.yaml + EXECUTION/)
      │
      ├── --ai? → enriquece seções com markers AGENT:BEGIN/END
      │
      ▼
  Escreve em staging: .dare/blueprint-stage-<pid>/
      │
      ▼
  dare_dag::validate (staged DAG)
      │
      ├── Falhou? → exit 1 + purge staging
      │
      ▼
  Plano: keep (sem marker managed) vs write (managed ou ausente)
      │
      ▼
  Cópia atômica → DARE/
      │
      ▼
  BlueprintReport
```

> **Garantia all-or-nothing:** O staging impede que um `dare-dag.yaml` inválido seja promovido para `DARE/`. Se a validação falhar, nenhum arquivo é escrito.

## Arquivos gerados

| Arquivo | Descrição |
|---|---|
| `DARE/BLUEPRINT.md` | Arquitetura técnica com trade-offs, stack, modelo de dados |
| `DARE/TASKS.md` | Lista de tasks atômicas com prioridades |
| `DARE/dare-dag.yaml` | Grafo de dependências entre tasks |
| `DARE/EXECUTION/task-001.md` | Spec individual da task 001 |
| `DARE/EXECUTION/task-NNN.md` | … e assim por diante |

## Marker `dare:managed`

Artefatos gerados pelo `dare blueprint` incluem o marker na primeira linha útil:

```markdown
<!-- dare:managed -->
# BLUEPRINT: Meu Projeto
...
```

Se você customizar manualmente um artefato **removendo** esse marker, o próximo `dare blueprint` preservará seu conteúdo (não sobrescreverá). Para forçar regeneração:

```bash
dare blueprint --force   # sobrescreve mesmo artefatos não-managed
```

## Enriquecimento com IA (`--ai`)

Com `--ai`, o DARE CLI enriquece as seguintes seções do BLUEPRINT.md via LLM:

- `trade-offs` — análise adicional de trade-offs arquiteturais
- `stack` — justificativas de escolhas técnicas
- `data-model` — campos e relacionamentos expandidos
- `endpoints` — contratos de API/CLI detalhados

```bash
dare blueprint --ai                          # provider padrão (codex)
dare blueprint --ai --provider anthropic     # Claude
dare blueprint --ai --provider openai        # GPT-4o
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "status": "ok",
  "artifacts": {
    "blueprint": "DARE/BLUEPRINT.md",
    "tasks": "DARE/TASKS.md",
    "dag": "DARE/dare-dag.yaml",
    "execution_count": 8
  },
  "dag_valid": true,
  "ai_enriched": false
}
```

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso — validação ok, writes promovidos |
| `1` | Validação do DAG falhou ou erro interno |
| `2` | Uso inválido (`--provider` sem `--ai`) |
| `3` | `DARE/DESIGN.md` não encontrado |
| `4` | Input inválido (root nulo, oversize, path fora do projeto) |
| `5` | Erro de I/O |

## Próximo passo

Revise o `DARE/BLUEPRINT.md` gerado e, se aprovado:

```bash
dare execute task-001
```
