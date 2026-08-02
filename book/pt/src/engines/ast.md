# AST Nativo

A engine de análise estática sintática nativa do DARE CLI, implementada na crate **`dare-ast`**, utiliza a biblioteca **`tree-sitter` v0.25.10** para analisar e extrair metadados e convenções do código-fonte do projeto de forma offline e de alta performance.

---

## Linguagens Suportadas e Gramáticas

O DARE possui parsers compilados embutidos (através de gramáticas mapeadas na crate `tree-sitter`) para as seguintes linguagens:

| Linguagem | Extensões | Versão da Gramática |
|---|---|---|
| TypeScript | `.ts`, `.mts`, `.cts` | `0.23.2` |
| TSX | `.tsx` | `0.23.2` (Parser TSX dedicado) |
| JavaScript | `.js`, `.mjs`, `.cjs` | `0.25.0` |
| Python | `.py` | `0.25.0` |
| Go | `.go` | `0.25.0` |
| PHP | `.php` | `0.24.2` |
| Ruby | `.rb` | `0.23.1` |
| Rust | `.rs` | `0.24.2` |

---

## Estrutura da Execução (`analyze_source`)

A análise de um arquivo de código-fonte realiza o seguinte fluxo:

1. **Detecção de Linguagem:** Mapeia a linguagem a partir da extensão do arquivo de forma case-insensitive.
2. **Limite de Tamanho:** Pula arquivos maiores que **2.097.152 bytes** (2MB) (`MAX_SOURCE_BYTES`) por segurança.
3. **Parseamento Sintático:** Executa o parser do `tree-sitter`.
4. **Extração de Entidades e Endpoints:** Varre a árvore sintática concreta (CST) extraindo:
   - Declarações de classes, structs, interfaces e tipos lógicos.
   - Chamadas de métodos HTTP em rotas web (normalizando os verbos em caixa alta ASCII, ex.: `GET`, `POST`, `PUT`, `DELETE`).
5. **Fallback por Expressões Regulares:** Caso o parsing sintático falhe, ou o arquivo seja de uma linguagem sem gramática compilada, a engine aciona automaticamente buscas orientadas a expressões regulares para recuperar entidades e chamadas a partir de padrões comuns de código.
6. **Deduplicação e Ordenação:** Combina as informações preferindo dados obtidos via AST nativo sobre o Regex. Ordena os resultados lexicograficamente por ID para manter o determinismo nos relatórios gerados.

---

## Onde é Utilizado?

A engine `dare-ast` alimenta diretamente as seguintes ferramentas da CLI:
- **`dare reverse`:** Para ler e detalhar o mapeamento de módulos, dependências internas e endpoints do sistema legado.
- **`dare dna`:** Para amostrar até **32 arquivos** (de até 512KB cada) e identificar regras de nomenclatura, imports de dependências e estilos de código preferidos no projeto brownfield.
- **`dare graph ingest`:** Para mapear e indexar relações estruturais de código no grafo local do GraphRAG.
