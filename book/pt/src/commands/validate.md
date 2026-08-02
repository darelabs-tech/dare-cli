# `dare validate`

Valida a integridade do grafo do DAG definido em `DARE/dare-dag.yaml`. É um utilitário estritamente **read-only** ideal para ser utilizado em pre-commit hooks, pipelines de CI ou antes de iniciar uma execução de tasks.

## Uso

```bash
dare validate [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--dag <PATH>` | Path relativo ou absoluto para o arquivo de DAG (padrão: `DARE/dare-dag.yaml`) |
| `--strict` | Modo estrito: qualquer aviso (warning) é tratado como falha/erro |
| `--json` | Saída formatada em JSON estruturado |

## O que é validado?

O processo de validação verifica regras de modelagem do grafo e integridade dos arquivos:

### 1. Estrutura do Grafo
- **Ciclos (Grafo Acíclico):** Utiliza o algoritmo de Kahn para detectar se há dependências cíclicas (ex.: Task A depende de Task B, que depende de Task A). Em caso de ciclo, exibe o caminho cíclico normalizado a partir do menor ID lexicográfico.
- **Referências Quebradas:** Garante que todas as tasks declaradas no campo `depends_on` realmente existem no grafo.

### 2. Validação de Tasks
- **Campos Obrigatórios:** Garante que cada task possui `id`, `title` e `status`.
- **Status Válido:** O status de cada task deve pertencer a `{pending, ready, running, done, failed}`.
- **Complexidade:** Se houver campo de complexidade, deve ser um dos valores case-sensitive: `LOW`, `MED` ou `HIGH`.

### 3. Integridade de Arquivos
- **Existence checks:** Verifica se os arquivos apontados por `spec_file` (geralmente sob `DARE/EXECUTION/`) existem fisicamente no disco.

---

## Exemplos de Uso

```bash
# Validação básica
dare validate

# Validação estrita (warnings geram falha)
dare validate --strict

# Validando um arquivo em localização alternativa
dare validate --dag DARE/meu-outro-dag.yaml
```

## Saída JSON (`--json`)

### Validação Com Sucesso
```json
{
  "schemaVersion": 1,
  "ok": true,
  "errors": [],
  "warnings": []
}
```

### Falha na Validação (Exit Code 1)
```json
{
  "schemaVersion": 1,
  "ok": false,
  "errors": [
    {
      "code": "cycle_detected",
      "message": "Ciclo detectado no grafo: task-001 -> task-002 -> task-001"
    },
    {
      "code": "missing_spec_file",
      "message": "Spec file DARE/EXECUTION/task-003.md não encontrado para a task-003"
    }
  ],
  "warnings": []
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Sucesso (`report.ok == true`) |
| `1` | Validação falhou (`report.ok == false` devido a erros ou warnings sob `--strict`) |
| `2` | Erro de uso das flags do CLI |
| `3` | Arquivo do DAG não encontrado |
| `4` | Entrada inválida (como caminho fora da sandbox do projeto ou erro de parse no YAML) |
| `5` | Erro inesperado de I/O ao ler o arquivo |
