# `dare reverse`

Executa engenharia reversa estrita em um projeto legado (Fase 0 do Brownfield) para mapear sua estrutura, inferir o domínio de negócio, e gerar os diagramas de fluxos correspondentes nos artefatos da pasta `DARE/`.

## Uso

```bash
dare reverse [OPTIONS]
```

## Flags

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--dir <PATH>` | path | cwd | Diretório raiz do projeto legado |
| `--check` | bool | false | Modo diagnóstico: apenas analisa sem escrever nenhum arquivo no disco (exibe em stdout) |
| `--deep` | bool | false | Habilita a geração profunda de stubs ERD e diagramas C4 adicionais |
| `--modules <LIST>` | string | todos | Lista separada por vírgula de módulos específicos para analisar |
| `--ast` | bool | false | Habilita o parse estático de arquivos via AST nativo (tree-sitter) |
| `--no-excalidraw` | bool | false | Desativa a geração de diagramas de arquitetura e fluxos em formato Excalidraw (default: habilitado) |
| `--json` | bool | false | Saída estruturada formatada em JSON |

---

## O que a engenharia reversa faz?

O comando `dare reverse` executa as seguintes etapas no diretório especificado:

### 1. Detecção de Módulos (Heurística)
Localiza os limites físicos de cada módulo analisando a estrutura de pastas:
- Identifica sub-crates (`crates/*`), diretórios de fonte comuns (`src/`, `app/`) ou diretórios estruturais no nível superior.
- Mapeia até **64 módulos** no máximo para evitar sobrecarga.

### 2. Análise do Código e AST (`--ast`)
Ao ativar o parsing com tree-sitter:
- Analisa até **200 arquivos** com limite individual de **1.048.576 bytes** (1MB) por arquivo.
- Extrai metadados do código como: endpoints expostos, tabelas/entidades, dependências e imports principais.

### 3. Geração de Artefatos
Se executado sem a flag `--check`, escreve em `DARE/`:
- **`DARE/IDEIA.md`:** Documento de visão geral do sistema, conceitos de negócio e escopo macro.
- **`DARE/REVERSE/module-*.md`:** Um documento para cada módulo detectado descrevendo responsabilidades e fluxos.
- **`DARE/REVERSE/reverse-facts.json`:** Fatos consolidados sobre a engenharia reversa (usado por `dare migrate`).
- **`modules.excalidraw`:** Desenho esquemático visual da estrutura de módulos e dependências do projeto.

---

## Exemplos de Uso

```bash
# Executa a engenharia reversa básica no projeto legado
dare reverse

# Apenas analisa e exibe o report em stdout
dare reverse --check

# Executa engenharia reversa profunda incluindo analise sintática de AST
dare reverse --ast --deep
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "projectRoot": "/home/user/legado",
  "stacks": ["node"],
  "modules": [
    {
      "id": "auth-service",
      "path": "crates/auth-service",
      "languages": ["rust"],
      "loc": 4500,
      "file_count": 24,
      "depends_on": ["db-helper"]
    }
  ],
  "ast": {
    "files_scanned": 150,
    "entities": ["User", "Session"],
    "endpoints": ["POST /login", "GET /logout"],
    "warnings": []
  },
  "deep": false
}
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Engenharia reversa concluída com sucesso (ou com `--check`) |
| `2` | Uso de argumentos inválidos |
| `3` | O diretório especificado em `-d`/`--dir` não foi encontrado |
| `4` | Caminho fora da sandbox ou projeto inválido sem root |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
