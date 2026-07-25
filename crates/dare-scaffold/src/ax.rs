//! Seven AX artifact generators (BLUEPRINT-046 §0.3).

use dare_core::{CoreError, CoreResult};
use serde_json::{json, Value};

use crate::render::scan_secrets;
use crate::types::{StackKind, StackMetadata};

/// Exact count of AX artifacts per stack.
pub const AX_ARTIFACT_COUNT: usize = 7;

/// OpenAPI stub document version.
pub const OPENAPI_STUB_VERSION: &str = "3.0.3";

/// Canonical relative paths for the 7 AX artifacts (order frozen in §0.3).
pub fn ax_artifact_paths(meta: &StackMetadata) -> Vec<String> {
    vec![
        "llms.txt".to_string(),
        "README.md".to_string(),
        ".env.example".to_string(),
        "openapi.json".to_string(),
        "Dockerfile".to_string(),
        "docker-compose.yml".to_string(),
        meta.rate_limit_rel.clone(),
    ]
}

/// Generate the 7 AX files as `(relative_path, content)` pairs.
///
/// Runs a secret scan on every body; any hit → `InvalidInput`.
pub fn generate_ax_files(
    meta: &StackMetadata,
    project_name: &str,
) -> CoreResult<Vec<(String, String)>> {
    let paths = ax_artifact_paths(meta);
    debug_assert_eq!(paths.len(), AX_ARTIFACT_COUNT);

    let mut out = Vec::with_capacity(AX_ARTIFACT_COUNT);
    for path in &paths {
        let content = match path.as_str() {
            "llms.txt" => render_llms_txt(meta, project_name),
            "README.md" => render_readme(meta, project_name),
            ".env.example" => render_env_example(meta, project_name),
            "openapi.json" => render_openapi(meta, project_name)?,
            "Dockerfile" => render_dockerfile(meta, project_name),
            "docker-compose.yml" => render_docker_compose(meta, project_name),
            _ if path == meta.rate_limit_rel.as_str() => {
                render_rate_limit(meta, project_name)
            }
            other => {
                return Err(CoreError::Internal(format!(
                    "unexpected AX path `{other}` for stack `{}`",
                    meta.id
                )));
            }
        };
        scan_secrets(&content)?;
        out.push((path.clone(), content));
    }
    Ok(out)
}

fn render_llms_txt(meta: &StackMetadata, project_name: &str) -> String {
    let kind = match meta.kind {
        StackKind::Backend => "HTTP backend",
        StackKind::Mcp => "MCP server",
    };
    let endpoints = match meta.kind {
        StackKind::Backend => {
            "- `GET /healthz` — liveness\n- `GET /openapi.json` — OpenAPI stub\n".to_string()
        }
        StackKind::Mcp => {
            "- Transport: stdio (default)\n- No HTTP paths in OpenAPI stub\n".to_string()
        }
    };
    format!(
        "# {project_name}\n\
         \n\
         > DARE-scaffolded {kind} using stack `{stack}` ({lang}).\n\
         \n\
         ## Stack\n\
         \n\
         - id: `{stack}`\n\
         - language: `{lang}`\n\
         - kind: `{kind_label}`\n\
         \n\
         ## Bootstrap\n\
         \n\
         - Copy `.env.example` to `.env` and fill non-secret placeholders\n\
         - See `README.md` § Bootstrap for stack-specific commands\n\
         \n\
         ## Endpoints\n\
         \n\
         {endpoints}\
         \n\
         ## Docs\n\
         \n\
         - `README.md` — human bootstrap\n\
         - `openapi.json` — API / MCP surface stub\n\
         - `llms.txt` — this file\n",
        stack = meta.id,
        lang = meta.language,
        kind_label = match meta.kind {
            StackKind::Backend => "backend",
            StackKind::Mcp => "mcp",
        },
    )
}

fn render_readme(meta: &StackMetadata, project_name: &str) -> String {
    format!(
        "# {project_name}\n\
         \n\
         Scaffolded with DARE stack `{stack}` ({lang}).\n\
         \n\
         ## Bootstrap\n\
         \n\
         1. Copy `.env.example` to `.env` (placeholders only — no real secrets).\n\
         2. Install language tooling for `{lang}`.\n\
         3. Run the stack entrypoint documented in your template skeleton.\n\
         4. Optional: `docker compose up` using the generated compose file.\n\
         \n\
         ## Docs\n\
         \n\
         - [`llms.txt`](./llms.txt) — agent discovery surface\n\
         - [`openapi.json`](./openapi.json) — OpenAPI stub\n\
         - Rate-limit starter: `{rate}`\n",
        stack = meta.id,
        lang = meta.language,
        rate = meta.rate_limit_rel,
    )
}

fn render_env_example(meta: &StackMetadata, project_name: &str) -> String {
    format!(
        "# Environment template for {project_name} (stack `{stack}`)\n\
         # Copy to .env and fill values. Never commit real secrets.\n\
         \n\
         APP_NAME={project_name}\n\
         PORT=\n\
         HOST=\n\
         LOG_LEVEL=info\n",
        stack = meta.id,
    )
}

fn render_openapi(meta: &StackMetadata, project_name: &str) -> CoreResult<String> {
    let doc: Value = match meta.kind {
        StackKind::Backend => json!({
            "openapi": OPENAPI_STUB_VERSION,
            "info": {
                "title": project_name,
                "version": "0.1.0",
                "description": format!("DARE scaffold OpenAPI stub for stack `{}`", meta.id),
            },
            "paths": {
                "/healthz": {
                    "get": {
                        "summary": "Liveness probe",
                        "operationId": "healthz",
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    }
                }
            }
        }),
        StackKind::Mcp => json!({
            "openapi": OPENAPI_STUB_VERSION,
            "info": {
                "title": format!("{project_name} MCP"),
                "version": "0.1.0",
                "description": format!(
                    "MCP OpenAPI stub for stack `{}` — no HTTP paths",
                    meta.id
                ),
            },
            "paths": {}
        }),
    };
    serde_json::to_string_pretty(&doc).map_err(|e| {
        CoreError::Internal(format!("failed to serialize openapi.json: {e}"))
    })
}

fn render_dockerfile(meta: &StackMetadata, project_name: &str) -> String {
    match meta.language.as_str() {
        "typescript" | "javascript" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM node:22-alpine AS build\n\
             WORKDIR /app\n\
             COPY package*.json ./\n\
             RUN npm ci\n\
             COPY . .\n\
             RUN npm run build\n\
             \n\
             FROM node:22-alpine\n\
             WORKDIR /app\n\
             ENV NODE_ENV=production\n\
             COPY --from=build /app ./\n\
             USER node\n\
             EXPOSE 3000\n\
             CMD [\"node\", \"dist/main.js\"]\n",
            stack = meta.id,
        ),
        "python" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM python:3.12-slim AS build\n\
             WORKDIR /app\n\
             COPY pyproject.toml ./\n\
             RUN pip install --no-cache-dir .\n\
             COPY . .\n\
             \n\
             FROM python:3.12-slim\n\
             WORKDIR /app\n\
             COPY --from=build /usr/local /usr/local\n\
             COPY --from=build /app /app\n\
             USER nobody\n\
             EXPOSE 8000\n\
             CMD [\"python\", \"-m\", \"uvicorn\", \"app.main:app\", \"--host\", \"0.0.0.0\", \"--port\", \"8000\"]\n",
            stack = meta.id,
        ),
        "php" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM php:8.3-cli AS build\n\
             WORKDIR /app\n\
             COPY composer.json ./\n\
             RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer \\\n\
                 && composer install --no-dev --prefer-dist\n\
             COPY . .\n\
             \n\
             FROM php:8.3-cli\n\
             WORKDIR /app\n\
             COPY --from=build /app /app\n\
             USER www-data\n\
             EXPOSE 8000\n\
             CMD [\"php\", \"-S\", \"0.0.0.0:8000\", \"-t\", \"public\"]\n",
            stack = meta.id,
        ),
        "ruby" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM ruby:3.3-slim AS build\n\
             WORKDIR /app\n\
             COPY Gemfile* ./\n\
             RUN bundle install --without development test\n\
             COPY . .\n\
             \n\
             FROM ruby:3.3-slim\n\
             WORKDIR /app\n\
             COPY --from=build /usr/local/bundle /usr/local/bundle\n\
             COPY --from=build /app /app\n\
             USER nobody\n\
             EXPOSE 3000\n\
             CMD [\"bundle\", \"exec\", \"rails\", \"server\", \"-b\", \"0.0.0.0\"]\n",
            stack = meta.id,
        ),
        "rust" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM rust:1.85-bookworm AS build\n\
             WORKDIR /app\n\
             COPY Cargo.toml Cargo.lock* ./\n\
             COPY src ./src\n\
             RUN cargo build --release\n\
             \n\
             FROM debian:bookworm-slim\n\
             WORKDIR /app\n\
             COPY --from=build /app/target/release/{project_name} /usr/local/bin/app\n\
             USER nobody\n\
             EXPOSE 8080\n\
             CMD [\"/usr/local/bin/app\"]\n",
            stack = meta.id,
        ),
        "go" => format!(
            "# Dockerfile for {project_name} ({stack})\n\
             FROM golang:1.23-bookworm AS build\n\
             WORKDIR /src\n\
             COPY go.mod go.sum* ./\n\
             RUN go mod download\n\
             COPY . .\n\
             RUN CGO_ENABLED=0 go build -o /out/app ./cmd/server\n\
             \n\
             FROM gcr.io/distroless/static-debian12\n\
             COPY --from=build /out/app /app\n\
             USER nonroot:nonroot\n\
             EXPOSE 8080\n\
             ENTRYPOINT [\"/app\"]\n",
            stack = meta.id,
        ),
        other => format!(
            "# Dockerfile for {project_name} ({stack}) — language `{other}`\n\
             FROM alpine:3.20\n\
             WORKDIR /app\n\
             COPY . .\n\
             USER nobody\n\
             CMD [\"echo\", \"override CMD for stack {stack}\"]\n",
            stack = meta.id,
        ),
    }
}

fn render_docker_compose(meta: &StackMetadata, project_name: &str) -> String {
    let port = match meta.language.as_str() {
        "typescript" | "javascript" | "ruby" => 3000,
        "python" | "php" => 8000,
        _ => 8080,
    };
    format!(
        "# docker-compose for {project_name} (stack `{stack}`)\n\
         services:\n\
           app:\n\
             build: .\n\
             ports:\n\
               - \"{port}:{port}\"\n\
             env_file:\n\
               - .env\n\
             healthcheck:\n\
               test: [\"CMD\", \"wget\", \"-qO-\", \"http://127.0.0.1:{port}/healthz\"]\n\
               interval: 30s\n\
               timeout: 5s\n\
               retries: 3\n\
               start_period: 10s\n",
        stack = meta.id,
    )
}

fn render_rate_limit(meta: &StackMetadata, project_name: &str) -> String {
    let stack = meta.id.as_str();
    let path = meta.rate_limit_rel.as_str();
    if path.ends_with(".ts") {
        format!(
            "/** AX rate-limit starter for `{project_name}` (stack `{stack}`). */\n\
             export const RATE_LIMIT_WINDOW_MS = 60_000;\n\
             export const RATE_LIMIT_MAX = 100;\n\
             \n\
             export function assertWithinLimit(count: number): boolean {{\n\
               return count < RATE_LIMIT_MAX;\n\
             }}\n"
        )
    } else if path.ends_with(".py") {
        format!(
            "\"\"\"AX rate-limit starter for `{project_name}` (stack `{stack}`).\"\"\"\n\
             \n\
             RATE_LIMIT_WINDOW_SECONDS = 60\n\
             RATE_LIMIT_MAX = 100\n\
             \n\
             \n\
             def assert_within_limit(count: int) -> bool:\n\
                 return count < RATE_LIMIT_MAX\n"
        )
    } else if path.ends_with(".php") {
        format!(
            "<?php\n\
             \n\
             declare(strict_types=1);\n\
             \n\
             namespace App\\Http\\Middleware;\n\
             \n\
             /**\n\
              * AX rate-limit starter for `{project_name}` (stack `{stack}`).\n\
              */\n\
             final class RateLimitStarter\n\
             {{\n\
                 public const WINDOW_SECONDS = 60;\n\
                 public const MAX_REQUESTS = 100;\n\
             }}\n"
        )
    } else if path.ends_with(".rb") {
        format!(
            "# frozen_string_literal: true\n\
             \n\
             # AX rate-limit starter for `{project_name}` (stack `{stack}`).\n\
             # Wire Rack::Attack (or equivalent) using these defaults.\n\
             module RackAttackStarter\n\
               WINDOW_SECONDS = 60\n\
               MAX_REQUESTS = 100\n\
             end\n"
        )
    } else if path.ends_with(".rs") {
        format!(
            "//! AX rate-limit starter for `{project_name}` (stack `{stack}`).\n\
             \n\
             pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;\n\
             pub const RATE_LIMIT_MAX: u32 = 100;\n\
             \n\
             pub fn within_limit(count: u32) -> bool {{\n\
                 count < RATE_LIMIT_MAX\n\
             }}\n"
        )
    } else if path.ends_with(".go") {
        format!(
            "package ratelimit\n\
             \n\
             // AX rate-limit starter for `{project_name}` (stack `{stack}`).\n\
             const (\n\
             \tWindowSeconds = 60\n\
             \tMaxRequests   = 100\n\
             )\n\
             \n\
             // WithinLimit reports whether count is under the max.\n\
             func WithinLimit(count int) bool {{\n\
             \treturn count < MaxRequests\n\
             }}\n"
        )
    } else {
        format!(
            "# AX rate-limit starter for `{project_name}` (stack `{stack}`)\n\
             # path: {path}\n\
             WINDOW_SECONDS=60\n\
             MAX_REQUESTS=100\n"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{list_stack_ids, scaffolder_for};
    use crate::render::SECRET_SCAN_NEEDLES;

    #[test]
    fn ax_paths_for_each_stack() {
        for &id in list_stack_ids() {
            let meta = scaffolder_for(id).expect("known stack").metadata();
            let paths = ax_artifact_paths(meta);
            assert_eq!(
                paths.len(),
                AX_ARTIFACT_COUNT,
                "stack `{id}` must expose exactly {AX_ARTIFACT_COUNT} AX paths"
            );
            assert_eq!(paths[0], "llms.txt");
            assert_eq!(paths[1], "README.md");
            assert_eq!(paths[2], ".env.example");
            assert_eq!(paths[3], "openapi.json");
            assert_eq!(paths[4], "Dockerfile");
            assert_eq!(paths[5], "docker-compose.yml");
            assert_eq!(paths[6], meta.rate_limit_rel);
            assert!(
                !paths[6].is_empty(),
                "rate_limit_rel must be non-empty for `{id}`"
            );
        }
        assert_eq!(list_stack_ids().len(), 11);
    }

    #[test]
    fn ax_openapi_mcp_stub_empty_paths() {
        for &id in list_stack_ids() {
            let meta = scaffolder_for(id).expect("known stack").metadata();
            if meta.kind != StackKind::Mcp {
                continue;
            }
            let files = generate_ax_files(meta, "demo-mcp").expect("generate");
            let openapi = files
                .iter()
                .find(|(p, _)| p == "openapi.json")
                .map(|(_, c)| c.as_str())
                .expect("openapi.json present");
            let v: Value = serde_json::from_str(openapi).expect("valid json");
            assert_eq!(v["openapi"], OPENAPI_STUB_VERSION);
            assert!(
                v["paths"].as_object().is_some_and(|o| o.is_empty()),
                "MCP `{id}` openapi paths must be empty object, got {}",
                v["paths"]
            );
            assert!(
                v["info"]["title"].as_str().is_some_and(|t| t.contains("demo-mcp")),
                "title should include project name"
            );
        }
    }

    #[test]
    fn ax_readme_has_required_sections() {
        for &id in list_stack_ids() {
            let meta = scaffolder_for(id).expect("known stack").metadata();
            let files = generate_ax_files(meta, "acme-app").expect("generate");
            let readme = files
                .iter()
                .find(|(p, _)| p == "README.md")
                .map(|(_, c)| c.as_str())
                .expect("README.md present");
            assert!(
                readme.contains("## Bootstrap"),
                "README for `{id}` missing ## Bootstrap"
            );
            assert!(
                readme.contains("## Docs"),
                "README for `{id}` missing ## Docs"
            );
            assert!(
                readme.contains("acme-app"),
                "README for `{id}` must include project_name"
            );
            assert!(
                readme.contains(id),
                "README for `{id}` must include stack id"
            );
        }
    }

    #[test]
    fn ax_secret_scan_on_generated() {
        for &id in list_stack_ids() {
            let meta = scaffolder_for(id).expect("known stack").metadata();
            let files = generate_ax_files(meta, "safe-app").expect("clean generate");
            assert_eq!(files.len(), AX_ARTIFACT_COUNT);
            for (path, content) in &files {
                scan_secrets(content).unwrap_or_else(|e| {
                    panic!("generated `{path}` for `{id}` failed secret scan: {e}");
                });
                assert!(
                    !content.contains('\r'),
                    "AX content for `{path}` must use LF only"
                );
            }
        }
    }

    #[test]
    fn ax_secret_scan_rejects_needles() {
        for needle in SECRET_SCAN_NEEDLES {
            let err = scan_secrets(&format!("leak {needle} value")).expect_err("must reject");
            match err {
                CoreError::InvalidInput(msg) => {
                    assert_eq!(msg, "template contains forbidden secret pattern");
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }
        // case-insensitive
        let err = scan_secrets("API_KEY=should-fail").expect_err("case fold");
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn ax_backend_openapi_includes_healthz() {
        let meta = scaffolder_for("rust-axum").expect("stack").metadata();
        let files = generate_ax_files(meta, "api-svc").expect("generate");
        let openapi = files
            .iter()
            .find(|(p, _)| p == "openapi.json")
            .map(|(_, c)| c.as_str())
            .expect("openapi");
        let v: Value = serde_json::from_str(openapi).expect("json");
        assert!(v["paths"]["/healthz"].is_object());
    }
}
