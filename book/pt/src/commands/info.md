# `dare info`

Diagnóstico **read-only** do ambiente DARE no projeto atual. Mostra versão, integridade da instalação, status do projeto e dos harnesses de agentes.

## Uso

```bash
dare info [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--root <PATH>` | Diretório raiz alternativo (padrão: cwd) |
| `--json` | Saída em JSON estruturado |

## O que verifica?

O `dare info` faz walk do diretório raiz e lê (sem modificar) os seguintes arquivos:

| Arquivo | O que extrai |
|---|---|
| `dare.config.json` | Nome do projeto, stack, toolchain, schema version |
| `DARE/TASKS.md` ou `DARE/TASKS-*.md` | Progresso das tasks (contagem DONE/PENDING) |
| `dare-graph.yml` ou `DARE/dare-graph.yml` | Config do grafo |
| `.dare/state.json` | Estado do DAG |

> **Garantia de segurança:** `dare info` é estritamente read-only. Nenhum arquivo é criado ou modificado.

## Saída humana

```
DARE CLI v4.0.0 (stable)

Project
  name:    meu-projeto
  stack:   rust
  root:    /home/user/meu-projeto

Methodology
  DESIGN.md:     ✅ present
  BLUEPRINT.md:  ✅ present
  TASKS.md:      ✅ 12 tasks (8 done / 4 pending)
  dare-dag.yaml: ✅ valid

Harnesses
  antigravity:   ✅ installed
  claude:        ✅ installed
  cursor:        ✅ installed
  codex:         ✅ installed

Assets
  embedded:      ✅ all ok (47 files)
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "cli": {
    "version": "4.0.0",
    "channel": "stable"
  },
  "project": {
    "name": "meu-projeto",
    "stack": "rust",
    "toolchain": "stable",
    "root": "/home/user/meu-projeto",
    "dareInitialized": true
  },
  "methodology": {
    "designPresent": true,
    "blueprintPresent": true,
    "tasks": {
      "total": 12,
      "done": 8,
      "pending": 4
    },
    "dagPresent": true,
    "dagValid": true
  },
  "harnesses": {
    "antigravity": "ok",
    "claude": "ok",
    "cursor": "ok",
    "codex": "ok"
  },
  "assets": {
    "ok": true,
    "count": 47
  }
}
```

## Detecção de root

O `dare info` detecta o root do projeto procurando por (na ordem):

1. `dare.config.json`
2. `DARE/` (diretório)
3. `Cargo.toml` (projetos Rust brownfield)

## Resolução de TASKS

- Usa `DARE/TASKS.md` se existir
- Senão, ordena lexicograficamente `DARE/TASKS-*.md` e usa o primeiro
- Contagem por heurística de texto: `✅` ou `DONE` = done; `⏳` ou `PENDING` = pending

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `1` | Erro interno |
| `3` | Root não encontrado |
| `5` | Erro de I/O |
