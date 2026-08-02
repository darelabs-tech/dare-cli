# `dare dna`

Extrai as convenções, estilo, bibliotecas e preferências do projeto legado (Fase 0 do Brownfield) e escreve o relatório de convenções `DARE/PROJECT-DNA.md`. Esse artefato serve como regra de contexto acionável para agentes de IA durante a reimplementação.

## Uso

```bash
dare dna [OPTIONS]
```

## Flags

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--dir <PATH>` | path | cwd | Diretório raiz do projeto |
| `--check` | bool | false | Modo diagnóstico: apenas analisa e exibe na tela (zero mutações no disco) |
| `--ast` | bool | false | Habilita a análise estática avançada via AST (tree-sitter) de amostras de código |
| `--json` | bool | false | Saída estruturada formatada em JSON |

---

## O que a extração de DNA analisa?

O comando realiza o diagnóstico das convenções do projeto através das seguintes categorias:

### 1. Ferramental (`tooling`)
Detecta gerenciadores de pacotes (ex.: `npm`, `pnpm`, `cargo`), edições de linguagem (ex.: `rustEdition: 2021`), versões de interpretadores ou runtimes.

### 2. Estilo de Nomenclatura (`naming`)
Analisa os arquivos de código para computar o estilo preferencial de nomenclatura (via votação de maioria) para arquivos, funções e variáveis. Valores comuns: `snake_case`, `kebab-case`, `camelCase`, `PascalCase`.

### 3. Arquitetura (`architecture`)
Detecta padrões de organização do código como camadas ativas (ex.: `controllers`, `services`, `repositories`, `models`), acoplamento e quantidade de entidades.

### 4. Testes (`tests`)
Mapeia a pasta e layout de testes (ex.: testes inline vs arquivos dedicados) e frameworks de teste ativos.

### 5. Bibliotecas e Histórico
- **Libraries (`libraries`):** Lista até as **25 dependências/bibliotecas** mais comuns usadas nos manifestos.
- **Commits (`commits`):** Executa `git log -n 20` para ler os últimos 20 commits do histórico e identificar estilo de mensagens e padrões adotados.

---

## Amostragem de AST (`--ast`)

Para manter a performance da CLI, a análise de AST utiliza um limite máximo:
- Analisa no máximo **32 arquivos** amostrados do projeto.
- Ignora arquivos que excedem **524.288 bytes** (512KB).
- Pula diretórios de compilação ou dependências instaladas como `target/`, `node_modules/` ou `.git/`.

---

## Exemplos de Uso

```bash
# Extrai as convenções do projeto e gera o arquivo DARE/PROJECT-DNA.md
dare dna

# Executa apenas análise diagnóstica sem gravar nada
dare dna --check

# Executa com análise de AST ativada para obter dados profundos de nomenclatura e arquitetura
dare dna --ast
```

## Estrutura dos Arquivos Gerados

Se executado em modo normal (sem `--check`), grava na raiz do projeto:
- **`DARE/PROJECT-DNA.md`:** Guia estruturado de estilo e convenções que a IA deve seguir.
- **`DARE/dna-facts.json`:** Consolidação estruturada de fatos com schema version 1.

---

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Extração do DNA concluída com sucesso (ou com `--check`) |
| `2` | Uso de argumentos inválidos |
| `3` | O diretório informado em `--dir` não existe |
| `4` | Caminho inválido (path safety reject) ou fora da sandbox |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
