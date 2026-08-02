# `dare graph`

Gerencia o grafo de conhecimento semântico e estruturado do GraphRAG. Permite indexar o código-fonte e os requisitos do projeto, realizar buscas híbridas rápidas e exportar diagramas visuais.

## Uso

```bash
dare graph <ACTION> [OPTIONS]
```

## Ações Disponíveis

| Ação | Descrição |
|---|---|
| `ingest` | Varre o projeto e indexa código-fonte, requisitos e relações no banco de dados local |
| `query <TEXT>` | Executa uma busca híbrida (FTS5 + Travessia BFS) no grafo a partir de uma query |
| `show <NODE_ID>` | Exibe detalhes e metadados estruturados sobre um nó específico do grafo |
| `stats` | Mostra estatísticas de uso, quantidade de nós e arestas armazenados |
| `export` | Exporta a estrutura atual do grafo em formato JSON ou formato visual Mermaid |

---

## Processamento de Ingestão (`dare graph ingest`)

A indexação realiza uma análise rápida e segura das conexões do código:
- **Scan Inteligente:** Pula pastas de compilação ou de pacotes como `node_modules/`, `target/` e `.git/`.
- **Análise Semântica:** Utiliza expressões regulares para mapear dependências, chamadas de métodos, relacionamentos entre arquivos e requisitos definidos.
- **Hash de Conteúdo:** Gera um hash SHA-256 (`contentHash`) para cada arquivo indexado no metadado do nó para evitar re-indexações desnecessárias caso o arquivo não tenha sido alterado.
- **Limites de Ingestão:** Processa no máximo **4.096 arquivos** por rodada, com limite individual de **1MB** por arquivo.

---

## Busca Híbrida e Ranking (`dare graph query`)

A busca no grafo combina duas técnicas para garantir que o contexto retornado seja o mais preciso possível:

1. **Keyword Search (LIKE / FTS5):** Busca direta por correspondência de texto no banco SQLite.
2. **BFS Expansion (Travessia):** A partir dos nós encontrados na primeira etapa, executa uma travessia BFS (Breadth-First Search) para trazer nós vizinhos relevantes. O padrão é expandir **2 hops** de distância no grafo (máximo configurável de **5 hops**), com limite de fanout máximo de **200 arestas**.
3. **RRF (Reciprocal Rank Fusion):** Combina as pontuações e ranqueia os resultados de forma otimizada usando a fórmula RRF com constante $k = 60$. Os empates são decididos pelo maior score e depois pelo ID em ordem alfabética.

---

## Tipos de Nós e Relações Congelados

O banco de dados SQLite (`.dare/graph.db`) armazena os seguintes tipos de dados:

### Nós (`NodeType`)
- `requirement` (Requisitos de negócio do `DESIGN.md`)
- `task` (Tasks de implementação)
- `file` (Arquivos do repositório)
- `code_symbol` (Structs, funções, classes extraídas)
- `schema` (Tabelas de banco de dados)
- `endpoint` (Contratos de API e CLI)
- `component` (Módulos de UI ou lógicos)
- `entity`, `concept`, `gate`, `pattern`, `formal-gate`

### Arestas (`EdgeType`)
- `depends_on` (Dependência entre nós)
- `implements` (Código que implementa um requisito/task)
- `uses`, `references`, `related_to`, `contains`, `extends`, `verified_by`, `affects`, `derives_from`, `evidenced_by`, `exhibits`, `proven_by`

---

## Exemplos de Uso

```bash
# Executa a varredura e ingestão dos arquivos do projeto no grafo
dare graph ingest

# Consulta o grafo por referências sobre "JWT"
dare graph query "JWT"

# Exibe estatísticas sobre o volume de nós indexados
dare graph stats

# Exporta o grafo em formato JSON estruturado
dare graph export --format json
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Operação do grafo executada com sucesso |
| `2` | Uso de argumentos inválidos |
| `3` | Arquivo do banco de dados do grafo ou nó solicitado não encontrado |
| `4` | Entrada inválida (como query vazia ou caminho fora da sandbox) |
| `5` | Falha inesperada de I/O na leitura ou gravação do SQLite/JSON |
