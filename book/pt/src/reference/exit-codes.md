# Exit Codes

O DARE CLI segue uma convenção uniforme de exit codes em todos os comandos.

| Código | Nome | Quando |
|---|---|---|
| `0` | `Ok` | Sucesso |
| `1` | `Internal` | Erro interno inesperado (bug, panic) |
| `2` | `Usage` | Uso incorreto do CLI (flags inválidas, `--interactive` sem TTY) |
| `3` | `NotFound` | Recurso não encontrado (arquivo, task, nó do grafo) |
| `4` | `InvalidInput` | Input inválido (campo vazio, oversize, path fora do projeto) |
| `5` | `Io` | Erro de I/O (leitura/escrita de arquivo, permissões) |
| `6` | `Network` | Erro de rede (download de release, API externa) |
| `7` | `Conflict` | Conflito de estado (task já completa, arquivo existe sem `--force`) |
| `8` | `Timeout` | Timeout de operação (gate, agente, rede) |

## Uso em scripts

```bash
dare dag next --json
if [ $? -ne 0 ]; then
  echo "Nenhuma task disponível"
fi
```

```powershell
dare info --json
if ($LASTEXITCODE -ne 0) {
  Write-Error "DARE não inicializado neste projeto"
}
```

## Exit codes são contratos de breaking change

Alterar o significado de um exit code existente é considerado **breaking change** e requer:

1. ADR aprovado
2. Entrada no CHANGELOG com `BREAKING`
3. Migration note

Veja [ADR-002](https://github.com/darelabs-tech/dare-cli/blob/main/docs/adr/ADR-002.md) para o contrato completo de saída JSON e exit codes.
