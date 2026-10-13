# DESIGN — Paridade do `dare init` Rust com o npm 3.18.1

## Problema
O `dare init` em Rust (4.0.0) não produz o mesmo resultado que o `dare init` do npm (`@dewtech/dare-cli@3.18.1`, repositório `dare-method`):

1. **Menu diferente.** Rust mostra uma lista plana com as 11 stacks (backend e MCP misturados), seguida de "Fullstack frontend: none/react/vue". Não existe a pergunta **Project structure** (Monorepo / Backend only / Frontend only / MVC / MCP Server). A stack escolhida vira o backend, sem que o usuário saiba.
2. **Layout errado.** Caso real: `dare-rust init api-nest-validations` com `node-nestjs` + `vue` colocou o backend na raiz e o frontend em `frontend/`. O npm gera `backend/` + `frontend/` (monorepo). Com "Frontend only", o npm põe o Vite na raiz, sem backend.
3. **Scaffold vazio.** Cada stack Rust tem 2 arquivos stub (`node-nestjs`: `package.json` + `src/main.ts`, sem `tsconfig.json`). O frontend tem 3 arquivos sem `index.html`, `vite.config` ou `tsconfig`. Nada compila. O npm renderiza de 25 a 60 arquivos por stack e roda os scaffolders oficiais de frontend, Rails e Leptos.
4. **Configuração e arquivos comuns ausentes.** Rust não pergunta IDE, GraphRAG, MCP Server nem toolchain auto/native/docker.
   - Instala os 4 harnesses sempre.
   - Não cria `DARE/README.md`, `DARE/EXECUTION/`, `templates/`, `dare-graph.yml` nem o bloco DARE do `.gitignore`.
   - Grava um `dare.config.json` com outro schema: `stack` em vez de `backend`, que o próprio `dare-verify` não lê.
5. **Arquivos a mais.** Rust gera `Dockerfile` e `docker-compose.yml`, que o npm não gera. `--toolchain docker` é gravado mas não tem efeito.

A causa raiz está nos ciclos 046/047: eles declararam fora de escopo os scaffolders oficiais e a geração completa ("só skeleton"), e esse desvio não entrou na tabela de diferenças do DEC-048.

## Objetivo
`dare4 init` (Rust) e `dare init` (npm) devem apresentar **os mesmos menus** e gerar **o mesmo scaffold** para a mesma combinação de respostas ou flags.

## Requisitos funcionais
- **RF-01 Menu interativo idêntico** (`commands/init.ts` linhas 37–240 do npm): mesmas perguntas, ordem, condicionais (`when`), rótulos com emoji, valores, defaults e mensagens de validação.
  - Project name
  - Project structure
  - MVC framework
  - MCP language, transport e capabilities (checkbox, `tools` marcado)
  - Backend stack (7)
  - Frontend stack (5)
  - Cargo workspace layout e crate prefix
  - Primary IDE / Agent (6)
  - GraphRAG backend
  - Enable DARE MCP Server (confirm, default sim)
  - Toolchain (auto / native / docker, default auto)
- **RF-02 Flags não interativas idênticas**: `--stack <id>`, `--fullstack` (booleano, só `ruby-rails-8`/`php-laravel` → structure `mvc`), `--mcp node-ts|python|rust|go`, `--transport stdio|sse|http`, `--toolchain native|docker|auto` (default `auto`), `--non-interactive`.
  - O modo não interativo entra quando `--non-interactive`, `--stack` ou `--mcp` está presente.
  - Defaults do npm: `ide=cursor`, `graphrag=sqlite`, `mcp=false`, nome `my-dare-project`.
  - Mensagens de erro iguais.
  - Extras exclusivos do Rust (`--dir`, `--force`, `--check`, `--json`) continuam como adições.
- **RF-03 Layout por estrutura** igual ao npm:
  - `backend`, `mvc` e `mcp-server` → raiz.
  - `frontend` → raiz.
  - `monorepo` → `backend/` + `frontend/`.
  - Rust monorepo (`rust-axum` + `rust-leptos*`) → `crates/server` + `crates/web` (single) ou `crates/<prefix>-{core,server,web,cli}` (multi), com workspace na raiz.
  - Verificação de diretório vazio igual ao npm (ignora `.git` e `.gitkeep`).
- **RF-04 Scaffold de backend e MCP** a partir dos mesmos templates de `templates/stacks/<id>/` do npm, com as mesmas listas de arquivos, variáveis (`projectName`, `crateName`, `projectModule`, `moduleName`, `defaultTransport`, `llmProvider`), renomeação `gitignore`→`.gitignore` e checagem de segredos em `.env.example`.
- **RF-05 Scaffolders externos**, com a mesma resolução de toolchain auto/native/docker:
  - React/Vue: `npx -y degit vitejs/vite/packages/create-vite/template-<react-ts|vue-ts> . --force`, depois `npm install` e renomeação do `package.json`.
  - Rails: `rails new . --database=postgresql --skip-bundle --skip-git --skip-test --skip-ci [--api]`, ou Docker `ruby:3.3`, ou templates offline, seguido do overlay DARE.
  - Leptos SSR e CSR: arquivos gerados e `cargo fetch`.
- **RF-06 Arquivos comuns**:
  - `DARE/README.md` e `DARE/EXECUTION/`
  - `dare.config.json` com o mesmo schema e a mesma ordem de chaves do npm (`name`, `structure`, `backend`, `frontend`, `ide`, `graphrag{…}`, `mcp`, `toolchain`, `version`, `review`, `refine`, `verification`, `hooks`, `guard`, `drift`, e `mcp*` quando a estrutura é MCP)
  - merge do `.gitignore`
  - `templates/` (6 templates de método)
  - `dare-graph.yml` conforme o backend do GraphRAG
- **RF-07 IDE seletiva**: instala apenas os arquivos da IDE escolhida, com o mesmo conteúdo do npm: `CLAUDE.md`, `.claude/commands` (49), `.claude/settings.json`, `.cursorrules`, `.cursor/commands`, `.cursor/rules` + `skill-<stack>.mdc`, `AGENTS.md`, `.agents/skills` (48), `.agents/workflows`, `.antigravityrules`.
- **RF-08 Saída de console**: cabeçalho, mensagens de progresso, "Project … created successfully!" e bloco de next steps iguais ao npm.
- **RF-09 `dare bootstrap`** passa a ler o novo schema (`backend`/`structure`) e continua aceitando o schema antigo (`stack`).

## Desvios intencionais em relação ao npm (bugs corrigidos)
Decisão do usuário: replicar o npm, mas **corrigir os bugs**. Cada correção fica registrada em `docs/compatibility/cli-init-bootstrap.md` para ser portada também para o npm.

| # | Bug no npm 3.18.1 | Comportamento no Rust |
|---|---|---|
| B-01 | `claude-hybrid` ("Claude Code + Cursor") instala só os arquivos do Claude | Instala Claude **e** Cursor |
| B-02 | `.claude/settings.json` de MCP node/rust/go usa `py_compile`/`pytest`/`ruff` | Comandos da linguagem MCP escolhida |
| B-03 | Mojibake (`âœ…`, `â†’`) em `settings.json`, `.cursorrules` e textos PT-BR | UTF-8 correto (`✅`, `→`) |
| B-04 | No Windows, `hasCommand` não encontra `npm`/`npx`/`rails` (`.cmd`), então `auto` sempre cai para Docker | Resolução via PATH + PATHEXT |
| B-05 | Frontend `none` gera `skill-none.mdc` | Não gera |
| B-06 | Next steps de MCP rust/go mostram `npm install` / `npm run dev` | Comandos da linguagem (cargo / go) |
| B-07 | Next steps multi-crate usam o slug do nome, não o `cratePrefix` | Usam o `cratePrefix` |
| B-08 | Em monorepo Leptos, `.cargo/` fica vazio no crate web | Diretório não é criado |
| B-09 | `--force` do Rust apaga um diretório pré-existente se o init falhar | Rollback remove só o que o init criou |

Outros desvios aceitos:
- `dare.config.json.version` recebe a versão do binário Rust (`4.x`).
- `Dockerfile` e `docker-compose.yml` deixam de ser gravados pelo init. O npm só grava esses arquivos no caminho legado `skipBootstrap`. Confirmado pelo usuário em 2026-10-12.
  - O Docker continua presente de duas formas: (a) como toolchain dos scaffolders (RF-05: `node:20-alpine`, `ruby:3.3`, `dare-rust-leptos:1`); (b) na fase 1 de containerização do blueprint (`/dare-docker`), igual ao npm.

## Fora de escopo
- Mudar o npm (as correções B-xx serão propostas lá em PR separado).
- Gerar os documentos de fase (DESIGN.md, BLUEPRINT.md, TASKS.md) no init. O npm também não gera.
- Código morto do npm (`bootstrapNestjs`, `composer create-project` etc.), que o init 3.18.1 não chama.

## Aceite
- Para cada combinação da matriz de paridade (BLUEPRINT §6), a árvore de arquivos e o conteúdo gerado pelo Rust são iguais ao golden gerado pelo npm, exceto os desvios listados acima.
  - Arquivos baixados da rede (template do Vite, `node_modules`, saída do `rails new`) são comparados pelo plano de comandos (argv), não pelo conteúdo.
- O cenário `node-nestjs` + `vue` em monorepo gera `backend/` com o scaffold NestJS completo e `frontend/` com o Vite vue-ts.
- O backend gerado para node-nestjs, python-fastapi, rust-axum e go-gin compila ou passa na checagem de sintaxe no CI quando a ferramenta está disponível.
- Gates: `cargo fmt --all -- --check`, `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo audit`, `cargo deny check`.
- Nenhum merge na `main` sem aprovação humana explícita.
