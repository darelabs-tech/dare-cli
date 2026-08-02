# Como Contribuir

O DARE CLI é desenvolvido em Rust com código aberto sob a licença Apache-2.0. Agradecemos contribuições da comunidade para o aprimoramento da metodologia e ferramentas.

---

## Pré-requisitos de Desenvolvimento

Para compilar e testar o DARE CLI localmente, você precisa ter:
- **Rustup & Cargo** (Recomendado toolchain `1.85.0` ou mais recente)
- **Acesso à internet** (para restaurar dependências do Crates.io)
- **Git**

---

## Estrutura do Workspace Rust

O projeto está organizado como um Cargo Workspace para manter a modularidade e evitar acoplamento cíclico:

- **`crates/dare-core`:** Componente base contendo validações de segurança do filesystem, restrição de paths (jail) e estruturas de erros globais.
- **`crates/dare-ast`:** Engine sintática nativa baseada em gramáticas do tree-sitter.
- **`crates/dare-graph`:** SQLite, FTS5 e persistência de nos e arestas da engine GraphRAG.
- **`crates/dare-dag`:** Algoritmo de Kahn, parser do YAML do DAG e persistência de estado.
- **`crates/dare-harness`:** Adaptadores e instaladores de regras para Cursor, Claude, Antigravity e Codex.
- **`crates/dare-server`:** Servidor web local Axum para o painel de visualização e endpoints REST.
- **`crates/dare-self`:** Pipeline HTTP e de assinaturas do Cosign para controle de atualizações.
- **`crates/dare-cli`:** Ponto de entrada binário e comandos Clap.

---

## Fluxo de Setup e Testes

Execute os comandos a partir da raiz do projeto:

```bash
# Clone o repositório
git clone https://github.com/darelabs-tech/dare-cli.git
cd dare-cli

# Compila o binário em modo debug
cargo build

# Executa todos os testes unitários e de fumaça (smoke tests)
cargo test

# Executa testes específicos de um módulo
cargo test -p dare-dag
```

---

## Regras de Qualidade e Código

Antes de submeter um Pull Request, certifique-se de validar o código de acordo com as diretrizes do ecossistema Rust:

1. **Formatação de Código:**
   Garante que o código segue o padrão estrito de estilo.
   ```bash
   cargo fmt --check
   ```
2. **Linter e Warnings:**
   Não são permitidos warnings no código de produção.
   ```bash
   cargo clippy --workspace -- -D warnings
   ```
3. **Mutações e Breaking Changes:**
   Alterações na saída de JSONs estruturados, assinaturas ou significados de exit codes existentes devem ser documentados através de uma especificação de ADR (Architecture Decision Record) na pasta `docs/adr/`.
