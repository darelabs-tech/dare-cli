# `dare review`

Audita a implementação atual de uma task específica, realizando análise estática e confrontando o código-fonte gerado com as especificações declaradas nos artefatos da pasta `DARE/EXECUTION/`.

## Uso

```bash
dare review [TASK_ID] [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--files <PATHS>` | Lista explícita de caminhos para auditar (ignora a detecção automática baseada na spec da task) |
| `--from-agent` | Integra dados semânticos enviados diretamente de um agente de IA durante um loop interativo |
| `--strict` | Eleva qualquer aviso (warning) a erro de validação (resultando em `ok = false`) |
| `--fail-on <SEVERITY>` | Nível de falha que causa exit code 1. Valores: `error`, `warning` ou `never` (padrão: `error`) |
| `--ai` | Executa enriquecimento semântico via LLM (apenas insere avisos de stub no relatório atual) |
| `--json` | Saída formatada em JSON estruturado |

---

## O que é auditado?

O `dare review` realiza varreduras rápidas e determinísticas orientadas a linhas sobre os arquivos de código correspondentes à task, verificando as seguintes regras:

### 1. Detecção de Stubs e Esboços
- Procura por assinaturas vazias, stubs ou funções contendo apenas `TODO`, `FIXME`, `panic!`, `todo!()` ou `unimplemented!()` no código de produção.

### 2. Validação de Mocks
- Garante que mocks e dados fictícios (stubbing) de APIs ou bancos de dados estejam restritos apenas a arquivos de teste (ex: caminhos contendo `test`, `spec` ou similares). Mocks no código de produção acendem alertas vermelhos de erro.

### 3. Limites de Arquivo
- Cap máximo de **1.048.576 bytes** (1MB) por arquivo. Arquivos que excederem o limite são ignorados e geram o aviso `file_too_large`.

### 4. Skip de Arquivos Binários
- Apenas arquivos de texto pertencentes à allowlist de linguagens são lidos e processados (ex: `.rs`, `.ts`, `.py`, `.go`, `.php`, `.rb`, `.toml`, `.yml`, `.json`, `.md`).

---

## Exemplos de Uso

```bash
# Executa a auditoria para a task-001
dare review task-001

# Executa em modo estrito, falhando com qualquer warning
dare review task-001 --strict

# Audita arquivos específicos diretamente
dare review --files "src/auth/jwt.rs,src/main.rs"
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "ok": false,
  "summary": "Review failed.",
  "findings": [
    {
      "filePath": "src/auth/jwt.rs",
      "lineNumber": 45,
      "columnNumber": 12,
      "severity": "error",
      "ruleId": "production_mock",
      "message": "Uso de mock detectado em código de produção: 'mock_token_validation'"
    },
    {
      "filePath": "src/auth/jwt.rs",
      "lineNumber": 89,
      "columnNumber": 5,
      "severity": "warning",
      "ruleId": "todo_marker",
      "message": "Marcador TODO pendente: 'TODO: implementar expiração de token'"
    }
  ],
  "unmetSemanticRequirements": []
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | A auditoria passou conforme os critérios definidos em `--fail-on` |
| `1` | A auditoria falhou (encontrou erros, warnings no modo estrito ou requisitos semânticos não atendidos) |
| `2` | Erro de uso nos argumentos fornecidos |
| `3` | O arquivo de especificação da task (`DARE/EXECUTION/{id}.md`) não foi encontrado |
| `4` | Caminhos fora da sandbox (path jail) ou argumentos inválidos |
| `5` | Erro inesperado de I/O ao ler os arquivos |
