# `dare update`

Gerencia o planejamento e a atualização de templates, hooks, manifests e skills de IA no projeto atual, sincronizando as ferramentas instaladas com a versão mais recente do DARE CLI.

## Uso

```bash
dare update [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--dry-run` | Executa o planejador estritamente **read-only**, exibindo as ações de atualização planejadas sem realizar moficações no disco |
| `--target <HARNESS_ID>` | Filtra a atualização por um harness de agente específico (valores válidos: `cursor`, `antigravity`, `claude-code`, `codex`, `hybrid`, `claude-hybrid`) |
| `--dir <PATH>` | Diretório raiz do projeto alternativo (padrão: cwd) |
| `--json` | Saída formatada em JSON estruturado |

> **Aviso Importante:** A execução real do update (sem a flag `--dry-run`) está em fase experimental. Chamar o comando sem `--dry-run` retornará erro de stub interno (Microplano 022). **Sempre use `--dry-run` neste estágio.**

---

## O que o planejador faz?

Ao rodar o `dare update --dry-run`, a CLI carrega o manifesto embutido `UpdateManifestV2` (JSON com os assets e hashes oficiais distribuídos com a CLI) e compara com o estado atual do projeto:

### 1. Classificação dos Arquivos
Cada arquivo inventariado no manifesto do DARE é verificado e classificado:
- **`missing`:** O arquivo não existe no projeto e será criado na atualização.
- **`identical`:** O arquivo existe no projeto e possui o SHA256 idêntico ao oficial da CLI.
- **`managed`:** O arquivo existe, possui alterações, mas contém o marcador de cabeçalho `<!-- dare:managed -->` ou `# dare:managed`. Ele será sobrescrito/atualizado.
- **`customized`:** O arquivo existe, foi modificado pelo usuário e **não** possui o marcador de gerenciamento. Ele **não** será tocado ou sobrescrito para preservar as customizações locais.

### 2. Expansão de Targets
A flag `--target` aceita atalhos que expandem filtros de agentes:
- `hybrid` expande e atualiza: `{cursor, antigravity}`
- `claude-hybrid` expande e atualiza: `{claude-code, cursor}`

---

## Exemplos de Uso

```bash
# Planeja a atualização de todas as ferramentas e exibe o relatório de modificações
dare update --dry-run

# Filtra o planejamento apenas para os harnesses do Cursor e Antigravity
dare update --dry-run --target hybrid

# Exibe o planejamento de atualização formatado em JSON
dare update --dry-run --json
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "cliVersion": "4.0.0",
  "projectRoot": "/home/user/meu-projeto",
  "target": "hybrid",
  "plan": {
    "creates": [
      {
        "path": ".agents/skills/dare-design/SKILL.md",
        "sha256": "4b7b25e791b8d69784df629007f3531b..."
      }
    ],
    "updates": [],
    "skips": [
      {
        "path": ".agents/AGENTS.md",
        "reason": "customized"
      }
    ]
  }
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Planejamento concluído com sucesso com `--dry-run` (mesmo com arquivos `customized` ignorados) |
| `1` | Chamada de execução de escrita sem `--dry-run` (ainda não implementado) ou erro interno do sistema |
| `2` | Uso de argumentos inválidos |
| `3` | Arquivo de manifesto embutido não encontrado |
| `4` | Entrada inválida (como caminho fora da sandbox ou erro de parsing) |
| `5` | Falha de I/O ao ler arquivos do projeto |
