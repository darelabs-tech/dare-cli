# `dare discover`

Instala ou analisa a estrutura de um projeto já existente (brownfield) para habilitar o uso da metodologia DARE e seus harnesses de agentes de IA sem alterar nenhuma linha do seu código atual.

## Uso

```bash
dare discover [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--dir <PATH>` | Diretório raiz do projeto existente (padrão: cwd) |
| `--check` | Executa em modo estrito **read-only**, apenas reportando o diagnóstico sem gravar arquivos no disco |
| `--json` | Saída formatada em JSON estruturado |

---

## O que a detecção analisa?

Ao rodar o `dare discover`, a CLI realiza a análise das seguintes características do diretório:

### 1. Project Root & Git Root
- Realiza walk-up procurando por marcadores como `Cargo.toml`, `package.json`, `pyproject.toml`, `requirements.txt`, `.git` ou pastas `DARE/`.
- Localiza o diretório Git correspondente (se presente).

### 2. Stacks & Tecnologias
Detecta as tecnologias ativas no projeto a partir da presença de arquivos de manifesto com limite de leitura de 256KB por arquivo:
- **Rust:** Presença de `Cargo.toml` (identifica se é single-crate ou workspace).
- **Node.js:** Presença de `package.json`.
- **Python:** Presença de `pyproject.toml`, `requirements.txt` ou `setup.py`.

### 3. Conflitos de Stack
- Caso detecte múltiplas tecnologias incompatíveis (ex.: múltiplos manifestos de linguagens diferentes no mesmo diretório), o relatório listará a ocorrência na chave `conflicts`.

### 4. Estruturas Monorepo
- Identifica se o projeto atual é um monorepo ou workspace (lendo seções como `[workspace]` no Rust ou campos `workspaces` no package.json).

### 5. Status de Harnesses de Agentes
- Verifica a presença de scaffolds e configurações de agentes de IA compatíveis no diretório `.agents/skills/`.

---

## Exemplos de Uso

```bash
# Apenas diagnostica a stack atual e exibe o report sem alterar arquivos
dare discover --check

# Diagnóstico com saída estruturada JSON
dare discover --check --json
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "projectRoot": "/home/user/workspace/meu-projeto",
  "gitRoot": "/home/user/workspace/meu-projeto",
  "detectedStacks": ["rust"],
  "isMonorepo": false,
  "conflicts": [],
  "harnesses": [
    { "id": "antigravity", "present": false },
    { "id": "claude", "present": false },
    { "id": "codex", "present": false },
    { "id": "cursor", "present": false }
  ]
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | A análise foi concluída com sucesso com a flag `--check` (mesmo com conflitos) |
| `1` | Chamado sem a flag `--check` (instalação não implementada neste estágio, planejado para o comando `install` / Microplano 019) ou erro interno |
| `2` | Uso de argumentos inválidos |
| `3` | O diretório especificado em `--dir` não existe |
| `4` | Problemas de segurança de caminhos (path safety) ou dados inválidos |
| `5` | Falha inesperada ao ler arquivos no disco |
