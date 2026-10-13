//! IDE / agent files — port of npm `utils/templates.ts` rule generators and the
//! `writeCursorFiles` / `generateClaudeFiles` / `generateCodexFiles` /
//! `writeAntigravityFiles` installers in `project-generator.ts`.
//!
//! Deviations (DESIGN cycle 011): B-01 `claude-hybrid` also installs Cursor,
//! B-02 MCP `.claude/settings.json` uses the MCP language's commands,
//! B-03 UTF-8 text instead of npm's double-encoded mojibake,
//! B-05 no `skill-none.mdc` when the frontend is `none`.

use std::path::Path;

use dare_assets::EmbeddedAssets;
use dare_core::{CoreError, CoreResult};

use super::config::{FrontendStack, Ide, McpLanguage, ProjectConfig, Structure};
use super::ordered_json::{arr, obj, s};
use super::writer::ProjectWriter;

const NPM_IDE_PREFIX: &str = "npm/ide/";

/// Copy every embedded file under `npm/ide/<src_prefix>` into `dest_dir`
/// (recursive, overwrite — npm `fs.copy`). `top_level_only` mirrors the
/// `readdir` + `isFile()` loop used for Cursor rules.
fn copy_embedded_tree(
    writer: &mut ProjectWriter,
    src_prefix: &str,
    dest_dir: &Path,
    top_level_only: bool,
    skip: &[&str],
) -> CoreResult<usize> {
    let prefix = format!("{NPM_IDE_PREFIX}{src_prefix}/");
    let mut paths: Vec<String> = EmbeddedAssets::iter()
        .filter(|p| p.starts_with(&prefix))
        .map(|p| p.into_owned())
        .collect();
    paths.sort();
    let mut n = 0;
    for p in paths {
        let rel = &p[prefix.len()..];
        if top_level_only && rel.contains('/') {
            continue;
        }
        if skip.contains(&rel) {
            continue;
        }
        let file = EmbeddedAssets::get(&p)
            .ok_or_else(|| CoreError::internal(format!("asset missing: {p}")))?;
        writer.write(&dest_dir.join(rel), file.data.as_ref())?;
        n += 1;
    }
    Ok(n)
}

fn embedded_ide_file(rel: &str) -> CoreResult<std::borrow::Cow<'static, [u8]>> {
    let key = format!("{NPM_IDE_PREFIX}{rel}");
    EmbeddedAssets::get(&key)
        .map(|f| f.data)
        .ok_or_else(|| CoreError::internal(format!("asset missing: {key}")))
}

// ─── Rule generators (templates.ts) ─────────────────────────────────────────

/// `TemplateConfig` / `McpTemplateConfig` inputs.
struct RuleCtx<'a> {
    backend: Option<&'a str>,
    frontend: Option<&'a str>,
    graphrag: &'a str,
    mcp: bool,
    mcp_transport: &'a str,
    mcp_language: &'a str,
    mcp_features: String,
}

impl<'a> RuleCtx<'a> {
    fn from(cfg: &'a ProjectConfig) -> Self {
        Self {
            backend: cfg.backend.map(|b| b.as_str()),
            frontend: cfg.frontend.map(|f| f.as_str()),
            graphrag: cfg.graphrag.as_str(),
            mcp: cfg.mcp,
            mcp_transport: cfg.mcp_transport.map_or("stdio", |t| t.as_str()),
            mcp_language: cfg.mcp_language.map_or("node-ts", |l| l.as_str()),
            mcp_features: cfg.mcp_features.as_ref().map_or_else(
                || "tools".to_string(),
                |fs| fs.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", "),
            ),
        }
    }
}

fn cursor_backend_rules(backend: &str) -> &'static str {
    match backend {
        "rust-axum" => {
            "## Backend: Rust/Axum
- Use Rust idioms and patterns (no unwrap in production)
- Prefer async/await with Tokio runtime
- Use Axum extractors for request handling
- Handle all errors with thiserror/anyhow
- Run `cargo clippy` and `cargo test` before completing any task (Ralph Loop)
- Use SQLx for database queries with compile-time verification"
        }
        "node-nestjs" => {
            "## Backend: Node.js/NestJS
- Use NestJS decorators and dependency injection
- Define DTOs with class-validator for all inputs
- Use TypeORM or Prisma for database access
- Write Jest unit and e2e tests for all services
- Follow NestJS module structure"
        }
        "python-fastapi" => {
            "## Backend: Python/FastAPI
- Use Pydantic v2 for all data validation
- Type all functions with PEP 484 type hints
- Use async/await for all IO operations
- Follow PEP 8 and use ruff for linting
- Write pytest tests with coverage"
        }
        "php-laravel" => {
            "## Backend: PHP/Laravel
- Follow PSR-12 coding standards
- Use FormRequests for all input validation
- Use API Resources for all responses
- Write PHPUnit feature and unit tests
- Use Eloquent ORM with proper relationships"
        }
        _ => "",
    }
}

fn cursor_frontend_rules(frontend: &str) -> &'static str {
    match frontend {
        "react" => {
            "## Frontend: React 18+
- Use functional components with hooks only
- TypeScript for all components and hooks
- Prefer React Query for server state management
- Use Zustand or Context API for client state
- Write Vitest + Testing Library tests"
        }
        "vue" => {
            "## Frontend: Vue 3+
- Use Composition API with <script setup> syntax
- TypeScript for all components
- Use Pinia for state management
- Use Vue Router for navigation
- Write Vitest + Vue Test Utils tests"
        }
        "rust-leptos" => {
            "## Frontend: Leptos fullstack (Rust SSR + WASM)
- Use `#[component]` macro — no class components
- State: `signal()`, `Resource` for async, `Action` for mutations
- Loading: `Suspense`/`Transition` — never block the render
- Server functions: `#[server]` macro (ssr feature only)
- Shared types: `#[cfg_attr(feature = \"ssr\", derive(sqlx::FromRow))]`
- Avoid: `tokio::spawn` on client, `panic!` in components, `Effect` for fetch
- Build: `cargo leptos build` | Test: `cargo test --workspace` | Dev: `cargo leptos watch`"
        }
        "rust-leptos-csr" => {
            "## Frontend: Leptos CSR (Rust WASM + trunk)
- Pure client-side WASM — no SSR, no `#[server]` functions
- Use `#[component]` macro, `signal()` for state, `Resource` for async
- Build tool: `trunk build` (NOT cargo leptos) | Dev: `trunk serve`
- Test: `cargo test --workspace` | Deploy: `dist/` is fully static"
        }
        _ => "",
    }
}

fn generate_cursor_rules(c: &RuleCtx<'_>) -> String {
    let backend = c.backend.map_or("", cursor_backend_rules);
    let frontend = c.frontend.map_or("", cursor_frontend_rules);
    let graphrag = c.graphrag;
    let context = if c.mcp {
        "- Query MCP Server at http://localhost:3000 for context instead of reading full files
- Use POST /context/query with {\"type\": \"file\"|\"task\"|\"dependency\", \"query\": \"...\"}"
    } else {
        "- Use DARE/BLUEPRINT.md as the single source of truth for context"
    };
    format!(
        "# DARE Framework - Cursor Rules

## DARE Methodology
You are an AI assistant following the DARE methodology:
- **D**esign: Define requirements and architecture
- **A**rchitect: Create technical blueprint and task graph
- **R**eview: Validate implementation against blueprint
- **E**xecute: Implement tasks following the DAG

## Core Rules
- Always read DARE/BLUEPRINT.md before implementing any feature
- Update DARE/TASKS.md status after completing each task
- Never skip the Ralph Loop (build → test → lint) before marking a task as DONE
- Human approval is required before merging to main branch
- Context is king: use MCP Server queries instead of re-reading large files

## Project Structure
- DARE/ - Methodology files (DESIGN.md, BLUEPRINT.md, TASKS.md, dare-dag.yaml)
- DARE/EXECUTION/ - Task execution logs

{backend}

{frontend}

## GraphRAG Context ({graphrag})
{context}

## Ralph Loop (Mandatory before DONE)
1. Build the project (cargo build / npm run build / etc)
2. Run tests (cargo test / npm test / pytest / etc)
3. Run linter (cargo clippy / eslint / ruff / phpstan)
4. Only mark task as DONE if all 3 steps pass
"
    )
}

fn stack_lines(c: &RuleCtx<'_>) -> (String, String) {
    (
        c.backend
            .map_or(String::new(), |b| format!("- Backend: {b}")),
        c.frontend
            .map_or(String::new(), |f| format!("- Frontend: {f}")),
    )
}

fn generate_antigravity_rules(c: &RuleCtx<'_>) -> String {
    let (backend, frontend) = stack_lines(c);
    let graphrag = c.graphrag;
    let context = if c.mcp {
        "- Query MCP Server at http://localhost:3000 for context
- Avoid reading full files; use targeted queries"
    } else {
        "- Read DARE/BLUEPRINT.md for context"
    };
    format!(
        "# DARE Framework - Antigravity Rules

## Agent Configuration
You are an autonomous AI agent following the DARE methodology.
Execute tasks from DARE/dare-dag.yaml in parallel when dependencies allow.

## DARE Phases
- **Design**: Read DARE/DESIGN.md for requirements
- **Architect**: Read DARE/BLUEPRINT.md for technical spec
- **Review**: Validate against blueprint before marking DONE
- **Execute**: Implement tasks, update DARE/TASKS.md

## Stack
{backend}
{frontend}

## Context Strategy ({graphrag})
{context}

## Execution Rules
- Always check task dependencies before starting
- Update task status in DARE/TASKS.md in real-time
- Run Ralph Loop before marking any task as DONE
- Request human review for architectural decisions
"
    )
}

fn generate_codex_rules(c: &RuleCtx<'_>) -> String {
    let (backend, frontend) = stack_lines(c);
    let graphrag = c.graphrag;
    let context = if c.mcp {
        "- Prefer querying the DARE MCP/context server for targeted context.
- Avoid reading large files when a focused graph/context query is enough."
    } else {
        "- Use `DARE/BLUEPRINT.md` as the primary source of context."
    };
    format!(
        "# AGENTS.md

## DARE Methodology
You are Codex working in a DARE project from the terminal.

- Design lives in `DARE/DESIGN.md`
- Architecture and task graph live in `DARE/BLUEPRINT.md`, `DARE/TASKS.md`, and `DARE/dare-dag.yaml`
- Execution details live in `DARE/EXECUTION/`
- Repository skills live in `.agents/skills` and may be invoked explicitly with `$skill-name`

## Terminal-First Workflow
- Prefer `dare <command>` over IDE chat workflows.
- For autonomous execution, use `dare execute --agent --driver codex`.
- For one-off Codex runs, use `codex exec --sandbox workspace-write --ask-for-approval never \"<task>\"`.
- Keep changes scoped to the selected task and summarize what changed.

## Core Rules
- Always read `DARE/BLUEPRINT.md` before implementing a feature.
- Check `DARE/dare-dag.yaml` task dependencies before editing.
- Never mark a task DONE unless the Ralph Loop passes.
- Do not fabricate implementation progress. If blocked, report the blocker.
- Use `dare review`, `dare validate`, and `dare guard` when the task touches shared behavior, task state, or agent-facing artifacts.

## Stack
{backend}
{frontend}

## Context ({graphrag})
{context}

## Ralph Loop
Before completing a task:
1. Build
2. Test
3. Lint
4. Mark DONE only after all gates pass
"
    )
}

fn generate_mcp_cursor_rules(c: &RuleCtx<'_>) -> String {
    let lang_rules = if c.mcp_language == "python" {
        "## Language: Python
- Use FastMCP (@mcp.tool, @mcp.resource, @mcp.prompt decorators)
- Type all arguments — FastMCP builds JSON schema from type hints
- Use docstrings as tool/resource/prompt descriptions
- Test: npx @modelcontextprotocol/inspector python main.py
- Lint: ruff check . && mypy ."
    } else {
        "## Language: TypeScript
- Import from '@modelcontextprotocol/sdk/server/index.js' and types
- Always define strict inputSchema for every tool (Claude depends on it)
- Use zod for runtime validation when needed
- Test: npm run inspect (MCP Inspector)
- Build: npm run build before marking any task DONE"
    };
    let transport_note = match c.mcp_transport {
        "stdio" => "- Transport: stdio — server communicates via stdin/stdout, no HTTP port needed",
        "sse" => "- Transport: SSE — server exposes GET /sse and POST /messages endpoints",
        _ => "- Transport: HTTP Stream — use StreamableHTTPServerTransport",
    };
    let context = if c.mcp {
        "- Query MCP Server at http://localhost:3000 for context instead of reading full files"
    } else {
        "- Use DARE/BLUEPRINT.md as the single source of truth"
    };
    let (transport, features, graphrag) = (c.mcp_transport, &c.mcp_features, c.graphrag);
    format!(
        "# DARE Framework - Cursor Rules (MCP Server Project)

## DARE Methodology
You are an AI assistant following the DARE methodology:
- **D**esign: Define MCP server requirements and capabilities
- **A**rchitect: Create technical blueprint and task graph
- **R**eview: Validate implementation against blueprint
- **E**xecute: Implement tools, resources, and prompts following the DAG

## Core Rules
- Always read DARE/BLUEPRINT.md before implementing any tool or resource
- Update DARE/TASKS.md status after completing each task
- Never skip the Ralph Loop (build → test → inspect) before marking a task as DONE
- Test every tool with MCP Inspector before marking DONE
- Validate tool inputSchema matches actual handler logic exactly

## MCP Server Configuration
- Transport: {transport}
- Features: {features}
{transport_note}

{lang_rules}

## MCP Best Practices
- Keep tool names snake_case and descriptive
- Return structured content arrays, not plain strings
- Handle unknown tool/resource/prompt names with explicit errors
- Never expose secrets via tool outputs or resource contents
- Document every tool argument in inputSchema description fields

## GraphRAG Context ({graphrag})
{context}

## Ralph Loop (Mandatory before DONE)
1. Build (npm run build / python -m py_compile)
2. Test (npm test / pytest)
3. Inspect with MCP Inspector to verify tool contracts
4. Only mark DONE if all 3 steps pass
"
    )
}

fn generate_mcp_antigravity_rules(c: &RuleCtx<'_>) -> String {
    let context = if c.mcp {
        "- Query MCP Server at http://localhost:3000 for project context"
    } else {
        "- Read DARE/BLUEPRINT.md for context"
    };
    let (language, transport, features, graphrag) =
        (c.mcp_language, c.mcp_transport, &c.mcp_features, c.graphrag);
    format!(
        "# DARE Framework - Antigravity Rules (MCP Server Project)

## Agent Configuration
You are an autonomous AI agent implementing an MCP server using the DARE methodology.
Execute tasks from DARE/dare-dag.yaml in parallel when dependencies allow.

## DARE Phases
- **Design**: Read DARE/DESIGN.md — what tools/resources/prompts does this MCP server expose?
- **Architect**: Read DARE/BLUEPRINT.md — tool schemas, transport, auth strategy
- **Review**: Test each tool with MCP Inspector before marking DONE
- **Execute**: Implement tasks, update DARE/TASKS.md

## MCP Stack
- Language: {language}
- Transport: {transport}
- Features: {features}

## Implementation Rules
- Each tool must have a strict inputSchema — Claude uses it to call the tool
- Test with MCP Inspector after implementing each tool
- Never skip error handling for unknown tool names

## Context Strategy ({graphrag})
{context}

## Execution Rules
- Always check task dependencies before starting
- Update task status in DARE/TASKS.md in real-time
- Run Ralph Loop before marking any task as DONE
- Request human review for transport or auth design decisions
"
    )
}

fn generate_mcp_codex_rules(c: &RuleCtx<'_>) -> String {
    let context = if c.mcp {
        "- Prefer targeted DARE MCP/context queries over rereading large files."
    } else {
        "- Use `DARE/BLUEPRINT.md` as the primary source of context."
    };
    let (language, transport, features, graphrag) =
        (c.mcp_language, c.mcp_transport, &c.mcp_features, c.graphrag);
    format!(
        "# AGENTS.md

## DARE Methodology
You are Codex implementing an MCP server from the terminal.

- Design lives in `DARE/DESIGN.md`
- Architecture and tool contracts live in `DARE/BLUEPRINT.md`
- Task order lives in `DARE/dare-dag.yaml`
- Repository skills live in `.agents/skills`

## MCP Server
- Language: {language}
- Transport: {transport}
- Features: {features}

## Terminal-First Workflow
- Prefer `dare <command>` over IDE chat workflows.
- For autonomous execution, use `dare execute --agent --driver codex`.
- For one-off Codex runs, use `codex exec --sandbox workspace-write --ask-for-approval never \"<task>\"`.

## Implementation Rules
- Every tool must have a strict input schema.
- Return structured content, not ad hoc strings, unless the protocol requires text.
- Test tool contracts with an MCP inspector or equivalent smoke test.
- Never expose secrets through tool outputs or resources.
- Do not mark DONE until build, tests, lint, and MCP inspection pass.

## Context ({graphrag})
{context}
"
    )
}

/// `generateStackSkill` (project-generator.ts).
fn generate_stack_skill(stack: &str) -> String {
    let fixed = match stack {
        "rust-leptos" => Some("---
description: Leptos fullstack (SSR + WASM) development skill
---
# Leptos Fullstack Skill
- Use `#[component]` macro for all components — no class components
- State: `signal()` (RwSignal), `ReadSignal`/`WriteSignal` — fine-grained reactivity
- Async data: `Resource` for fetching, never `Effect` that calls fetch
- Mutations: `Action` for server calls (forms, submits)
- Loading states: `Suspense` and `Transition` — never block the render
- Conditionals: `Show`, `For` components — not if/for in view!
- Server functions: `#[server]` macro — only available with `ssr` feature
- Shared types (server + WASM): use `#[cfg_attr(feature = \"ssr\", derive(sqlx::FromRow))]`
- Avoid: `tokio::spawn` on client (no async runtime in WASM), `panic!` in components, `wasm_bindgen` direct
- Build: `cargo leptos build` (not `cargo build`)
- Test: `cargo test --workspace` (not `cargo leptos test` — that doesn't exist)
- Lint: `cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
- Dev server: `cargo leptos watch` (port 3000 + hot reload on port 3001)
"),
        "rust-leptos-csr" => Some("---
description: Leptos CSR (WASM + trunk) development skill
---
# Leptos CSR Skill
- Pure client-side WASM — no server rendering, no `#[server]` functions
- Use `#[component]` macro for all components
- State: `signal()`, `create_memo()` for derived state
- Async data: `Resource` — never `Effect` that calls fetch
- Entry point: `leptos::mount::mount_to_body(App)` called from `main()`
- Build tool: `trunk build` (NOT `cargo leptos build` — wrong tool for CSR)
- Dev server: `trunk serve` (port 3001 by default, configured in Trunk.toml)
- Test: `cargo test --workspace`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check`
- Deploy: `trunk build --release` produces `dist/` — static files, deployable to any CDN
"),
        "rust-axum" => Some("---\ndescription: Rust/Axum API development skill\n---\n# Rust/Axum Skill\n- Use Axum for HTTP routing\n- Use Tokio for async runtime\n- Use SQLx for database\n- Run clippy and cargo test\n"),
        "go-gin" => Some("---\ndescription: Go/Gin API development skill\n---\n# Go + Gin Skill\n- Use Gin for HTTP routing (github.com/gin-gonic/gin)\n- Project layout: cmd/api/main.go (entrypoint), internal/handlers, internal/middleware, internal/services, internal/repository\n- Use context.Context propagation in every handler/service\n- Use struct tags for binding/validation (binding:\"required\" on DTOs)\n- For SQL: prefer database/sql + sqlx, parametrize ALL queries, no string concat\n- Errors: return wrapped errors (fmt.Errorf(\"...: %w\", err)), not panics\n- Tests: use net/http/httptest + table-driven tests; place *_test.go alongside the package\n- Ralph Loop: `go build ./...`, `go test ./...`, `go vet ./...`\n"),
        "go-stdlib" => Some("---\ndescription: Go API development with stdlib only (no framework)\n---\n# Go stdlib Skill (net/http only)\n- Use `net/http` from stdlib — NO framework (no Gin, Echo, Chi, Fiber). Adding one defeats the purpose of this stack.\n- Routing: `http.NewServeMux()` with the Go 1.22+ pattern syntax: `mux.HandleFunc(\"GET /api/v1/users/{id}\", h)`, `mux.HandleFunc(\"POST /api/v1/users\", h)`. Use `r.PathValue(\"id\")` to extract path params.\n- Project layout: `cmd/api/main.go` (entrypoint), `internal/handlers/`, `internal/middleware/`, `internal/services/`, `internal/repository/`.\n- Middleware = function wrapping http.Handler. Compose with `middleware.Recover(middleware.Logger(mux))` style — no framework chain.\n- JSON: `json.NewDecoder(r.Body).Decode(&dto)` for input, `json.NewEncoder(w).Encode(obj)` for output. Always set `Content-Type: application/json` and the status code explicitly.\n- Validation: prefer `github.com/go-playground/validator/v10` for struct tags (lightweight, no framework lock-in). For trivial cases, manual validation in the handler is fine.\n- Logging: `log/slog` from stdlib. `slog.Info(\"...\", \"key\", val)` style. Configure JSON handler in main.\n- Context propagation: `r.Context()` flows through every layer. NEVER use context.Background() inside handlers.\n- For SQL: prefer `database/sql` + `github.com/jmoiron/sqlx` or `github.com/jackc/pgx/v5`. Parametrize ALL queries.\n- Errors: `fmt.Errorf(\"doing X: %w\", err)`. Map domain errors to HTTP status in the handler layer, not deeper.\n- Tests: `net/http/httptest` + table-driven. `httptest.NewRequest` + `httptest.NewRecorder`. Test each handler in isolation.\n- Ralph Loop: `go build ./...`, `go test ./...`, `go vet ./...`.\n- Avoid: middleware libraries that pull in heavyweight deps; ORM (use sqlx); reflection-based DI (wire by hand in main).\n"),
        "node-nestjs" => Some("---\ndescription: Node.js/NestJS development skill\n---\n# NestJS Skill\n- Use NestJS modules and DI\n- Use DTOs with class-validator\n- Write Jest tests\n"),
        "python-fastapi" => Some("---\ndescription: Python/FastAPI development skill\n---\n# FastAPI Skill\n- Use Pydantic for validation\n- Use async endpoints\n- Write pytest tests\n"),
        "php-laravel" => Some("---\ndescription: PHP/Laravel development skill\n---\n# Laravel Skill\n- Use FormRequests\n- Use API Resources\n- Write PHPUnit tests\n"),
        "react" => Some("---\ndescription: React development skill\n---\n# React Skill\n- Use functional components\n- Use TypeScript\n- Use React Query\n- Write Vitest tests\n"),
        "vue" => Some("---\ndescription: Vue 3 development skill\n---\n# Vue Skill\n- Use Composition API\n- Use Pinia\n- Write Vitest tests\n"),
        _ => None,
    };
    fixed.map_or_else(
        || format!("---\ndescription: {stack} skill\n---\n# {stack} Skill\n"),
        str::to_string,
    )
}

/// `generateMcpStackSkill` (project-generator.ts).
fn generate_mcp_stack_skill(language: &str) -> &'static str {
    if language == "python" {
        "---
description: Python MCP server development skill
---
# Python MCP Skill
- Use FastMCP for rapid server development
- Decorate tools with @mcp.tool(), resources with @mcp.resource(), prompts with @mcp.prompt()
- Use type hints — FastMCP derives the JSON schema automatically
- Test with: npx @modelcontextprotocol/inspector python main.py
- Use mcp.run() for stdio, mcp.run(transport=\"sse\") for SSE
"
    } else {
        "---
description: Node.js/TypeScript MCP server development skill
---
# TypeScript MCP Skill
- Import Server from @modelcontextprotocol/sdk/server/index.js
- Use StdioServerTransport for CLI tools, SSEServerTransport for web integrations
- Define tools with ListToolsRequestSchema + CallToolRequestSchema handlers
- Keep inputSchema strict — Claude uses it to call your tools correctly
- Test with: npm run inspect (uses @modelcontextprotocol/inspector)
- Build before shipping: npm run build
"
    }
}

/// `generateClaudeSettings` with B-02 (MCP language commands) and B-03.
pub fn generate_claude_settings(cfg: &ProjectConfig) -> String {
    let frontend = cfg.frontend;
    let backend = cfg.backend.map(|b| b.as_str());
    let leptos_full = frontend == Some(FrontendStack::RustLeptos);
    let leptos_csr = frontend == Some(FrontendStack::RustLeptosCsr);
    let mcp_lang = (cfg.structure == Structure::McpServer)
        .then(|| cfg.mcp_language.unwrap_or(McpLanguage::NodeTs));

    let (build, test, lint) = if leptos_full {
        (
            "cargo leptos build --release",
            "cargo test --workspace",
            "cargo clippy --all-features -- -D warnings",
        )
    } else if leptos_csr {
        (
            "trunk build --release",
            "cargo test --workspace",
            "cargo clippy --all-features -- -D warnings",
        )
    } else if backend == Some("rust-axum") {
        ("cargo build", "cargo test", "cargo clippy")
    } else if let Some(lang) = mcp_lang {
        match lang {
            McpLanguage::Python => ("python -m py_compile main.py", "pytest", "ruff check ."),
            McpLanguage::NodeTs => ("npm run build", "npm test", "npx eslint src"),
            McpLanguage::Rust => ("cargo build", "cargo test", "cargo clippy"),
            McpLanguage::Go => ("go build ./...", "go test ./...", "go vet ./..."),
        }
    } else if backend == Some("python-fastapi") {
        ("python -m py_compile main.py", "pytest", "ruff check .")
    } else if backend == Some("ruby-rails-8") {
        ("bin/rails zeitwerk:check", "bin/rails test", "bin/rubocop")
    } else if backend == Some("php-laravel") {
        (
            "composer install --no-interaction",
            "php artisan test",
            "vendor/bin/pint --test",
        )
    } else {
        ("npm run build", "npm test", "npx eslint src")
    };

    let echo =
        format!("echo \"✅ File saved. Remember to run Ralph Loop: {build} && {test} && {lint}\"");
    let value = obj(vec![
        (
            "permissions",
            obj(vec![(
                "allow",
                arr(vec![
                    s("Bash(git:*)"),
                    s(&format!("Bash({build})")),
                    s(&format!("Bash({test})")),
                    s(&format!("Bash({lint})")),
                    s("Read(DARE/**)"),
                    s("Write(DARE/**)"),
                    s("Read(src/**)"),
                    s("Write(src/**)"),
                ]),
            )]),
        ),
        (
            "hooks",
            obj(vec![(
                "PostToolUse",
                arr(vec![obj(vec![
                    ("matcher", s("Write")),
                    (
                        "hooks",
                        arr(vec![obj(vec![
                            ("type", s("command")),
                            ("command", s(&echo)),
                        ])]),
                    ),
                ])]),
            )]),
        ),
    ]);
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

// ─── Installers ──────────────────────────────────────────────────────────────

fn write_cursor_files(
    writer: &mut ProjectWriter,
    dir: &Path,
    cfg: &ProjectConfig,
) -> CoreResult<()> {
    let rules_dir = dir.join(".cursor/rules");
    writer.ensure_dir(&rules_dir)?;
    writer.ensure_dir(&dir.join(".cursor/commands"))?;
    copy_embedded_tree(
        writer,
        "cursor/.cursor/commands",
        &dir.join(".cursor/commands"),
        false,
        &[],
    )?;
    copy_embedded_tree(
        writer,
        "cursor/.cursor/rules",
        &rules_dir,
        true,
        &["skill-laravel-api.mdc"],
    )?;
    if cfg.structure == Structure::McpServer {
        let lang = cfg.mcp_language.map_or("node-ts", |l| l.as_str());
        writer.write(
            &rules_dir.join("skill-mcp-server.mdc"),
            generate_mcp_stack_skill(lang),
        )?;
        return Ok(());
    }
    match cfg.backend.map(|b| b.as_str()) {
        Some("php-laravel") => {
            let bytes = embedded_ide_file("cursor/.cursor/rules/skill-laravel-api.mdc")?;
            writer.write(&rules_dir.join("skill-laravel-api.mdc"), bytes.as_ref())?;
        }
        Some(b) => writer.write(
            &rules_dir.join(format!("skill-{b}.mdc")),
            generate_stack_skill(b),
        )?,
        None => {}
    }
    // B-05: npm treats the string 'none' as truthy and writes skill-none.mdc.
    if let Some(f) = cfg.frontend.filter(|f| *f != FrontendStack::None) {
        writer.write(
            &rules_dir.join(format!("skill-{}.mdc", f.as_str())),
            generate_stack_skill(f.as_str()),
        )?;
    }
    Ok(())
}

fn write_antigravity_files(writer: &mut ProjectWriter, dir: &Path) -> CoreResult<()> {
    let skills = dir.join(".agents/skills");
    writer.ensure_dir(&skills)?;
    copy_embedded_tree(writer, "antigravity/.agents/skills", &skills, false, &[])?;
    writer.ensure_dir(&dir.join(".agents/workflows"))
}

fn generate_claude_files(
    writer: &mut ProjectWriter,
    dir: &Path,
    cfg: &ProjectConfig,
) -> CoreResult<()> {
    let claude_md = embedded_ide_file("claude/CLAUDE.md")?;
    writer.write(&dir.join("CLAUDE.md"), claude_md.as_ref())?;
    let commands = dir.join(".claude/commands");
    writer.ensure_dir(&commands)?;
    copy_embedded_tree(writer, "claude/.claude/commands", &commands, false, &[])?;
    writer.write(
        &dir.join(".claude/settings.json"),
        generate_claude_settings(cfg),
    )
}

/// IDE section of `generateProjectStructure` (cursor → claude → codex → antigravity).
pub fn install_ide_files(
    writer: &mut ProjectWriter,
    dir: &Path,
    cfg: &ProjectConfig,
) -> CoreResult<()> {
    let ctx = RuleCtx::from(cfg);
    let is_mcp = cfg.structure == Structure::McpServer;
    // B-01: claude-hybrid ("Claude Code + Cursor") installs Cursor too.
    if matches!(cfg.ide, Ide::Cursor | Ide::Hybrid | Ide::ClaudeHybrid) {
        let rules = if is_mcp {
            generate_mcp_cursor_rules(&ctx)
        } else {
            generate_cursor_rules(&ctx)
        };
        writer.write(&dir.join(".cursorrules"), rules)?;
        write_cursor_files(writer, dir, cfg)?;
    }
    if matches!(cfg.ide, Ide::ClaudeCode | Ide::ClaudeHybrid) {
        generate_claude_files(writer, dir, cfg)?;
    }
    if cfg.ide == Ide::Codex {
        let rules = if is_mcp {
            generate_mcp_codex_rules(&ctx)
        } else {
            generate_codex_rules(&ctx)
        };
        writer.write(&dir.join("AGENTS.md"), rules)?;
        write_antigravity_files(writer, dir)?;
    }
    if matches!(cfg.ide, Ide::Antigravity | Ide::Hybrid) {
        let rules = if is_mcp {
            generate_mcp_antigravity_rules(&ctx)
        } else {
            generate_antigravity_rules(&ctx)
        };
        writer.write(&dir.join(".antigravityrules"), rules)?;
        write_antigravity_files(writer, dir)?;
    }
    Ok(())
}

/// `writeDareTemplates` — copies `templates/ide/cursor/templates/` to `templates/`.
pub fn write_dare_templates(writer: &mut ProjectWriter, dir: &Path) -> CoreResult<()> {
    let dest = dir.join("templates");
    writer.ensure_dir(&dest)?;
    copy_embedded_tree(writer, "cursor/templates", &dest, false, &[])?;
    Ok(())
}
