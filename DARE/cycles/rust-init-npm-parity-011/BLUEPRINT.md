# BLUEPRINT — Paridade do `dare init` Rust com o npm 3.18.1

Referência npm: `dare-method/packages/cli` (commit `274c3b2`): `src/commands/init.ts`, `src/utils/project-generator.ts`, `src/utils/stack-bootstrap.ts`, `src/utils/templates.ts`, `src/stacks/*/scaffold.ts` e `templates/`.

## 1. Componentes atuais e o que muda

| Componente | Hoje | Depois |
|---|---|---|
| `dare-cli/src/commands/init_prompts.rs` | Lista plana de stacks + fullstack + toolchain | Árvore de prompts idêntica à do npm (RF-01) |
| `dare-cli/src/commands/init.rs` | `InitFlags`/`InitRequest` com `stack` + `fullstack <react\|vue>` | `ProjectConfig` igual ao do npm (RF-02); pipeline novo |
| `dare-scaffold` | `GenericScaffolder` com 2 stubs por stack + 7 arquivos AX + `frontend/` | Módulos `project`, `stacks`, `tools`, `common` (abaixo); AX/Docker removidos do init |
| `assets/stacks/*` | Stubs | Substituídos pelos templates do npm (`templates/stacks/**`) |
| `assets/ide/**` | Não existe (harness gera a partir da capability-matrix) | Cópia de `templates/ide/{claude,cursor,antigravity}` (174 arquivos, ~750 KB) |
| `dare-harness` | Usado pelo init com 4 harnesses fixos | Continua servindo ao `discover`/`update`; o init usa o instalador por IDE novo |
| `bootstrap.rs` | Lê `stack` | Lê `backend`/`structure` (com fallback para `stack`) e reaproveita o pipeline de stacks |

## 2. Modelo de dados (`dare-scaffold::project`)
```rust
pub enum Structure { Monorepo, Backend, Frontend, Mvc, McpServer }
pub enum BackendStack { RubyRails8, RustAxum, NodeNestjs, PythonFastapi, PhpLaravel, GoGin, GoStdlib }
pub enum FrontendStack { React, Vue, RustLeptos, RustLeptosCsr, None }
pub enum McpLanguage { NodeTs, Python, Rust, Go }
pub enum McpTransport { Stdio, Sse, HttpStream }      // serializa como "http-stream", igual ao npm
pub enum McpFeature { Tools, Resources, Prompts }
pub enum Ide { ClaudeCode, Codex, Cursor, Antigravity, Hybrid, ClaudeHybrid }
pub enum GraphragBackend { Sqlite, Json, Neo4j }
pub enum ToolchainMode { Auto, Native, Docker }
pub enum RustWorkspaceLayout { Single, Multi }

pub struct ProjectConfig {
    name, structure, backend: Option<_>, frontend: Option<_>,
    mcp_language, mcp_transport, mcp_features: Option<Vec<_>>,
    ide, graphrag, mcp: bool, toolchain, rust_workspace_layout, crate_prefix,
    output_dir: ProjectRoot,
}
```
Os valores serializados (`serde(rename_all = "kebab-case")` e afins) são exatamente as strings `value` do npm.

## 3. Pipeline (`generate_project_structure`, espelha `project-generator.ts:95-254`)
1. `assert_within_cwd` (validação de nome igual a `init-validation.ts`), depois `ensure_dir`.
2. `assert_output_dir_is_empty` nos diretórios da tabela de layout.
3. `run_stack_bootstrap`, na ordem backend → frontend → MCP → raiz do workspace Rust. Imprime `✓ Stack scaffold complete.`
4. Cria `DARE/` e `DARE/EXECUTION/`.
5. `dare.config.json`: ordem de chaves e defaults de `verification`/`hooks`/`guard`/`drift` copiados das fontes npm; `version` = versão do binário.
6. Merge do `.gitignore` (bloco `# DARE Framework`, extras de python no MCP).
7. Arquivos de IDE, conforme a seleção (§5).
8. `DARE/README.md` (`generateSharedConfig`).
9. `templates/` (6 templates de método).
10. `dare-graph.yml`, somente se ainda não existir.

**Rollback:** journal das paths criadas. Em caso de erro, remove só o que o init criou (corrige B-09). Diretórios criados por processos externos ficam registrados no journal pela raiz do diretório de destino, que é verificada vazia antes.

## 4. Stacks e ferramentas externas
- **Renderer** (`stacks::render`):
  - Substitui `{{ var }}`/`{{var}}` em `.hbs`, `.tpl`, `.tera`, `.j2`, `.jinja2` e `.mustache`, e remove a extensão.
  - Os demais arquivos são copiados sem alteração.
  - Variável indefinida gera erro (igual ao `throwOnUndefined`/`strict` do npm).
  - Inventário confirmado: nenhum template usa blocos de controle, então não é preciso adicionar um motor de templates.
- **Scaffolders por stack** (`stacks::<id>`): lista `TEMPLATE_FILES` e variáveis idênticas a cada `scaffold.ts`.
  - Os arquivos de Rails seguem a substituição `<%= k %>`/`<%= k -%>` própria do `RailsScaffold`, incluindo as variantes fullstack e o esqueleto offline.
  - Pós-render: `emit()` nos valores do `.env.example`, com os padrões de segredo do `dna-emitter.ts`.
- **Ferramentas** (`tools::StackTool`): resolução auto/native/docker com as mesmas mensagens do npm.
  - Detecta o comando via PATH + PATHEXT, sem spawn (reaproveita a lógica de `dare-ai::doctor::program_resolves`; corrige B-04).
  - Execução via `dare_core::process::SafeCommand` + `ProcessRunner`. No Windows, executa os shims `.cmd` resolvidos.
  - Docker: `docker run --rm -v <dir>:/app -w /app [--user uid:gid] <image> …`.
  - Imagens: `node:20-alpine` (Vite), `ruby:3.3` (Rails), `ghcr.io/dewtech-technologies/dare-rust-leptos:1` (Leptos).
- **Workspace Rust** (`stacks::rust_workspace`): `Cargo.toml` raiz, `.cargo/config.toml`, crates `-core`/`-cli` no layout multi e `.gitignore`, byte a byte como em `createRustFullstackWorkspace`.

## 5. IDE (`ide::install`)

| IDE | Arquivos |
|---|---|
| `cursor` / `hybrid` | `.cursorrules` (`generateCursorRules`/`generateMcpCursorRules`), `.cursor/commands/*`, `.cursor/rules/*` + `skill-<stack>.mdc` |
| `claude-code` | `CLAUDE.md`, `.claude/commands/*` (49), `.claude/settings.json` (`generateClaudeSettings` com as correções B-02 e B-03) |
| `claude-hybrid` | Claude + Cursor (B-01) |
| `codex` | `AGENTS.md` (`generateCodexRules`/`generateMcpCodexRules`), `.agents/skills/*`, `.agents/workflows/` |
| `antigravity` / `hybrid` | `.antigravityrules`, `.agents/skills/*`, `.agents/workflows/` |

Os geradores de texto de `templates.ts` (`generateCursorRules`, `generateAntigravityRules`, `generateCodexRules`, `generateStackSkill`, `generateMcp*`) são portados como funções Rust com os mesmos textos, já com o mojibake corrigido.

## 6. Matriz de paridade (golden)
Golden gerado **a partir do npm**, com o script `scripts/parity/gen-init-golden.mjs`:
- roda o `dist/` do `dare-method` com um PATH de *shims* (`npx`, `npm`, `cargo`, `rails`, `docker`) que gravam o argv em `commands.jsonl` e retornam 0, sem acesso à rede;
- salva a árvore, o hash SHA-256 de cada arquivo e o plano de comandos em `crates/dare-cli/tests/fixtures/golden/init-npm/<caso>/`;
- o teste Rust (`tests/init_npm_parity.rs`) roda o `dare` com o mesmo PATH de shims e compara árvore, conteúdo e argv;
- as diferenças B-xx são aplicadas como *patches esperados* declarados no próprio teste (lista explícita, sem curinga).

Casos (sufixo `/<ide>`):
1. `backend/<cada uma das 7 stacks>/cursor` (não interativo)
2. `mvc/ruby-rails-8` e `mvc/php-laravel` (`--fullstack`; Rails em modo offline e docker-plan)
3. `mcp/<node-ts|python|rust|go>/stdio`, `mcp/node-ts/sse`, `mcp/node-ts/http`
4. `monorepo/node-nestjs+vue/claude-code` (**caso reportado pelo usuário**)
5. `monorepo/python-fastapi+react/codex`
6. `frontend/vue/antigravity` e `frontend/react/hybrid`
7. `monorepo/rust-axum+rust-leptos/single/claude-hybrid` e `monorepo/rust-axum+rust-leptos-csr/multi(prefix=ars)/cursor`
8. GraphRAG `json` e `neo4j` (sobre o caso 1 com node-nestjs)

Prompts interativos: os testes injetam um `Prompter` falso (trait sobre o dialoguer) e verificam a sequência de perguntas, rótulos, defaults e condicionais de cada caminho. Um snapshot textual da árvore de prompts é comparado com `init.ts`.

## 7. Compatibilidade e migração
- `--fullstack` passa de `--fullstack <react|vue>` (Rust 4.0) para flag booleana (npm). `--fullstack react|vue` é rejeitado com uma mensagem que orienta usar o modo interativo (monorepo). O npm não tem flag não interativa para monorepo ou frontend.
  - **Ponto para revisão:** aceitar ou não o alias legado `--fullstack react` → monorepo.
- `dare.config.json`: `bootstrap`, `verify` e `info` leem `backend`, com fallback para `stack` em projetos 4.0.
- `docs/compatibility/cli-init-bootstrap.md` e `DECISION-LOG` (novo DEC) registram a reversão do escopo "skeleton only" do 047 e a lista B-01..B-09.

## 8. Gates
`cargo fmt --all -- --check` · `cargo build --workspace` · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo audit` · `cargo deny check`.

Sem dependências novas previstas. Se alguma entrar, ela passa por `cargo audit` e `cargo deny`.
