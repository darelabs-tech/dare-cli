# Changelog

Acompanhe o histórico de alterações do DARE CLI.

---

## v4.0.0 (Versão Atual)

A versão v4.0.0 representa um marco para o ecossistema DARE Labs: **substituição total da CLI legada baseada em Node.js/npm por uma CLI 100% nativa em Rust**, sem dependências externas de runtime e cross-platform.

### Adicionado
- **Arquitetura 100% Nativa (Rust):** Binários compilados nativamente distribuídos para Linux, macOS e Windows com MSRV 1.85.0.
- **DAG Task Runner:** Engine baseada no algoritmo de Kahn (topological sort) com suporte a paralelismo de tasks e controle de estados (`pending`, `ready`, `running`, `done`, `failed`, `split`).
- **GraphRAG local:** Engine de contexto híbrida baseada em SQLite + FTS5 nativo. Busca combinando LIKE/FTS5, travessias BFS e pontuação RRF (k=60). Habilitação de Neo4j configurável para grandes projetos.
- **AST Parser nativo:** Crate `dare-ast` baseada em gramáticas tree-sitter de alta velocidade para Rust, Python, Go, PHP, Ruby, JS e TS.
- **Segurança com Guard:** Assinaturas criptográficas Ed25519 (.minisig), sanitização de Unicode (bloqueio homograph) e análise de injeções de segredos locais.
- **Harnesses para IDEs:** Adaptadores unificados e automatizados gerando arquivos de regras e comandos para Antigravity, Claude Code, Cursor e Codex.
- **Painel de Controle:** Servidor local baseado em Axum para o painel de telemetria e controle de status REST compatível.
- **Servidor MCP:** Servidor Model Context Protocol embutido na CLI para economizar consumo de tokens de contexto arquitetural nas IDEs compatíveis.
- **Self-Update e Rollback:** Comando `dare self update` integrado a assinaturas digitais via Cosign para atualizações seguras de binários.

### Depreciado
- **Legacy npm Package:** O pacote npm legado `@dewtech/dare-cli@3.18.1` foi movido para o status **legacy** (fim de suporte ativo, apenas patches críticos de segurança até a janela de encerramento). Todos os usuários devem migrar para os binários nativos.
