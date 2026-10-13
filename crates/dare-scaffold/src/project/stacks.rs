//! Backend / MCP stack scaffolders — port of npm `src/stacks/*/scaffold.ts`.
//!
//! Templates are the npm package's `templates/stacks/<id>/` tree, embedded
//! under `assets/npm/stacks/` (see `scripts/sync-npm-templates.py`).

use std::path::Path;

use dare_assets::EmbeddedAssets;
use dare_core::{CoreError, CoreResult};

use super::render::{render_template_str, TemplateVars};
use super::stack_table::template_files;
use super::writer::ProjectWriter;

const LLM_PROVIDER_DEFAULT: &str = "dummy";

/// Embedded npm stack template bytes.
pub fn stack_template(
    stack_id: &str,
    template: &str,
) -> CoreResult<std::borrow::Cow<'static, [u8]>> {
    let key = format!("npm/stacks/{stack_id}/{template}");
    EmbeddedAssets::get(&key)
        .map(|f| f.data)
        .ok_or_else(|| CoreError::internal(format!("Template not found: {key}")))
}

fn stack_template_str(stack_id: &str, template: &str) -> CoreResult<String> {
    let bytes = stack_template(stack_id, template)?;
    String::from_utf8(bytes.into_owned()).map_err(|e| {
        CoreError::internal(format!("template {stack_id}/{template} is not UTF-8: {e}"))
    })
}

/// Options shared by the registry scaffolders (npm `ScaffoldOpts`).
#[derive(Debug, Clone)]
pub struct StackScaffoldOpts<'a> {
    pub stack_id: &'a str,
    pub project_name: &'a str,
    pub is_monorepo: bool,
    /// MCP scaffolder transport (`stdio` | `sse` | `http`).
    pub mcp_transport: Option<&'a str>,
}

fn vars_for(opts: &StackScaffoldOpts<'_>) -> CoreResult<TemplateVars> {
    let name = opts.project_name.to_string();
    let snake = name.replace('-', "_");
    let transport = opts.mcp_transport.unwrap_or("stdio").to_string();
    let mut v = TemplateVars::new();
    v.insert("projectName", name.clone());
    v.insert("llmProvider", LLM_PROVIDER_DEFAULT.to_string());
    match opts.stack_id {
        "node-nestjs" => {
            v.insert("hasRealtime", "false".into());
        }
        "python-fastapi" => {
            v.insert("projectModule", snake);
        }
        "php-laravel" => {}
        "rust-axum" => {
            v.insert("crateName", snake);
            v.insert("isMonorepo", opts.is_monorepo.to_string());
        }
        "go-gin" | "go-stdlib" => {
            v.insert("moduleName", name);
        }
        "mcp-node-ts" => {
            v.insert("defaultTransport", transport);
        }
        "mcp-python" => {
            v.insert("projectModule", snake);
            v.insert("defaultTransport", transport);
        }
        "mcp-rust" => {
            v.insert("crateName", snake);
            v.insert("defaultTransport", transport);
        }
        "mcp-go" => {
            v.insert("moduleName", name);
            v.insert("defaultTransport", transport);
        }
        other => return Err(CoreError::invalid_input(format!("Unknown stack: {other}"))),
    }
    Ok(v)
}

/// Run a registry scaffolder (every stack except Rails): render each
/// `TEMPLATE_FILES` entry into `dir`, then validate `.env.example`.
pub fn scaffold_registry_stack(
    writer: &mut ProjectWriter,
    dir: &Path,
    opts: &StackScaffoldOpts<'_>,
) -> CoreResult<Vec<String>> {
    let files = template_files(opts.stack_id)
        .ok_or_else(|| CoreError::invalid_input(format!("Unknown stack: {}", opts.stack_id)))?;
    let vars = vars_for(opts)?;
    let mut written = Vec::with_capacity(files.len());
    for (template, dest) in files {
        let source = stack_template_str(opts.stack_id, template)?;
        let rendered = render_template_str(template, &source, &vars)?;
        writer.write(&dir.join(dest), rendered.as_bytes())?;
        written.push((*dest).to_string());
    }
    let env = std::fs::read_to_string(dir.join(".env.example"))
        .map_err(|e| CoreError::io(format!("read .env.example: {e}")))?;
    validate_env_example(&env)?;
    written.sort();
    Ok(written)
}

// ─── .env.example secret gate (npm dna-emitter.ts SECRET_PATTERNS) ──────────

fn longest_run(value: &str, pred: impl Fn(char) -> bool) -> usize {
    let mut best = 0;
    let mut cur = 0;
    for c in value.chars() {
        if pred(c) {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 0;
        }
    }
    best
}

fn has_prefixed_run(
    value: &str,
    prefix: &str,
    min: usize,
    pred: impl Fn(char) -> bool + Copy,
) -> bool {
    value.match_indices(prefix).any(|(i, _)| {
        value[i + prefix.len()..]
            .chars()
            .take_while(|c| pred(*c))
            .count()
            >= min
    })
}

fn secret_pattern(value: &str) -> Option<&'static str> {
    if longest_run(value, |c| c.is_ascii_alphanumeric() || c == '+' || c == '/') >= 40 {
        return Some("/[A-Za-z0-9+/]{40,}={0,2}/");
    }
    if longest_run(value, |c| c.is_ascii_digit() || ('a'..='f').contains(&c)) >= 32 {
        return Some("/[a-f0-9]{32,}/");
    }
    if let Some(i) = value.find("-----BEGIN ") {
        let rest = &value[i + "-----BEGIN ".len()..];
        if let Some(j) = rest.find(" PRIVATE KEY-----") {
            if rest[..j]
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == ' ')
            {
                return Some("/-----BEGIN [A-Z ]+ PRIVATE KEY-----/");
            }
        }
    }
    if has_prefixed_run(value, "sk-", 40, |c| c.is_ascii_alphanumeric()) {
        return Some("/sk-[A-Za-z0-9]{40,}/");
    }
    if has_prefixed_run(value, "AKIA", 16, |c| {
        c.is_ascii_uppercase() || c.is_ascii_digit()
    }) {
        return Some("/AKIA[0-9A-Z]{16}/");
    }
    None
}

/// Port of npm `validateEnvExample` — rejects secret-looking values.
pub fn validate_env_example(content: &str) -> CoreResult<()> {
    for (i, raw) in content.split('\n').enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(eq) = trimmed.find('=') else {
            continue;
        };
        let value = trimmed[eq + 1..].trim();
        if value.is_empty() {
            continue;
        }
        if let Some(pattern) = secret_pattern(value) {
            return Err(CoreError::invalid_input(format!(
                ".env.example line {}: value '{value}' matches secret pattern {pattern}",
                i + 1
            )));
        }
    }
    Ok(())
}

// ─── Rails (npm ruby-rails-8/scaffold.ts RailsScaffold) ──────────────────────

const RAILS: &str = "ruby-rails-8";

const RAILS_DIRS: &[&str] = &[
    "app/handlers",
    "app/services",
    "app/repositories",
    "app/models",
    "app/presenters",
    "app/middleware",
    "app/jobs",
    "app/controllers/concerns",
    "config/initializers",
    "lib/llm/providers",
    "lib/llm/prompts",
    "lib/llm/validators",
    "lib/llm/cache",
    "lib/llm/rate_limit",
    "app/channels/application_cable",
    "lib/tasks",
    "config/environments",
    "bin",
    "db/migrate",
    "spec/services",
    "spec/handlers",
    "spec/channels",
    "spec/factories",
    "spec/api",
    "spec/support",
    "public",
    "tmp",
    ".dare",
    ".github/workflows",
];

const RAILS_FULLSTACK_DIRS: &[&str] = &[
    "app/views/layouts",
    "app/views/home",
    "app/helpers",
    "app/assets/stylesheets",
    "app/javascript/controllers",
];

const RAILS_INITIALIZERS: &[&str] = &["dare.rb", "rack_attack.rb", "rswag_api.rb", "zeitwerk.rb"];

const RAILS_USER_EXAMPLE: &[&str] = &[
    "app/models/user.rb",
    "app/repositories/user_repository.rb",
    "app/services/create_user_service.rb",
    "app/presenters/user_presenter.rb",
    "app/handlers/users_handler.rb",
    "db/migrate/20260101000001_create_users.rb",
    "spec/services/create_user_service_spec.rb",
    "spec/handlers/users_handler_spec.rb",
    "spec/factories/users.rb",
    "spec/api/users_spec.rb",
];

const RAILS_LLM: &[&str] = &[
    "lib/llm/providers/llm_provider.rb",
    "lib/llm/providers/openai_provider.rb",
    "lib/llm/providers/dummy_provider.rb",
    "lib/llm/prompts/prompt_loader.rb",
    "lib/llm/prompts/summarize_v1.jinja2",
    "lib/llm/validators/validator.rb",
    "lib/llm/validators/summarize_output_schema.json",
    "lib/llm/cache/llm_cache.rb",
    "lib/llm/rate_limit/token_bucket.rb",
];

const RAILS_CHANNELS: &[&str] = &[
    "app/channels/application_cable/connection.rb",
    "app/channels/application_cable/channel.rb",
    "app/channels/dare_updates_channel.rb",
    "app/channels/user_updates_channel.rb",
    "app/services/realtime_service.rb",
    "spec/channels/user_updates_channel_spec.rb",
];

const RAILS_SUMMARIZE: &[&str] = &[
    "app/handlers/summarize_handler.rb",
    "app/services/summarize_document_service.rb",
    "app/repositories/document_repository.rb",
    "spec/services/summarize_document_service_spec.rb",
    "spec/api/summarize_spec.rb",
    "spec/channels/dare_updates_channel_spec.rb",
];

const RAILS_RUNTIME_COPIED: &[&str] = &[
    "config/boot.rb",
    "config/environment.rb",
    "config/environments/development.rb",
    "config/environments/test.rb",
    "config/environments/production.rb",
    "config/puma.rb",
    "config/initializers/filter_parameter_logging.rb",
    "config.ru",
    "Rakefile",
    ".ruby-version",
    "app/models/application_record.rb",
    "app/jobs/application_job.rb",
    "bin/rails",
    "bin/rake",
    "bin/bundle",
    "db/seeds.rb",
    "public/404.html",
    "public/422.html",
    "public/500.html",
    "public/robots.txt",
];

const RAILS_ASSET_RUNTIME: &[&str] = &[
    "config/importmap.rb",
    "config/initializers/assets.rb",
    "app/assets/stylesheets/application.css",
    "app/javascript/application.js",
    "app/javascript/controllers/application.js",
    "app/javascript/controllers/index.js",
    "bin/dev",
];

/// `toModuleName` — CamelCase Ruby module name.
pub fn rails_module_name(app_name: &str) -> String {
    let camel: String = app_name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if camel.starts_with(|c: char| c.is_ascii_uppercase()) {
        camel
    } else {
        format!("App{camel}")
    }
}

/// `app_database` — snake_case DB name.
pub fn rails_database_name(app_name: &str) -> String {
    let lower = app_name.to_lowercase();
    let mut out = String::new();
    let mut in_run = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            in_run = false;
        } else if !in_run {
            out.push('_');
            in_run = true;
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "app".to_string()
    } else {
        trimmed
    }
}

fn rails_render(app_name: &str, source: &str) -> String {
    let vars: [(&str, String); 6] = [
        ("app_name", app_name.to_string()),
        ("app_module", rails_module_name(app_name)),
        ("app_database", rails_database_name(app_name)),
        ("ruby_version", ">= 3.3.0".to_string()),
        ("rails_version", "~> 8.0".to_string()),
        ("llm_provider", LLM_PROVIDER_DEFAULT.to_string()),
    ];
    let mut content = source.to_string();
    for (key, value) in &vars {
        content = content.replace(&format!("<%= {key} %>"), value);
        content = content.replace(&format!("<%= {key} -%>"), value);
    }
    content
}

struct RailsCtx<'a> {
    writer: &'a mut ProjectWriter,
    dir: &'a Path,
    app_name: &'a str,
}

impl RailsCtx<'_> {
    fn render_to(&mut self, template: &str, dest: &str) -> CoreResult<()> {
        let source = stack_template_str(RAILS, template)?;
        let out = rails_render(self.app_name, &source);
        self.writer.write(&self.dir.join(dest), out.as_bytes())
    }

    fn copy_to(&mut self, template: &str, dest: &str) -> CoreResult<()> {
        let bytes = stack_template(RAILS, template)?;
        self.writer.write(&self.dir.join(dest), bytes.as_ref())
    }

    fn copy_same(&mut self, rel: &str) -> CoreResult<()> {
        self.copy_to(rel, rel)
    }
}

/// Port of `RailsScaffold.generate` (verbose: false, all features enabled).
pub fn scaffold_rails(
    writer: &mut ProjectWriter,
    dir: &Path,
    app_name: &str,
    fullstack: bool,
    native_runtime_provided: bool,
) -> CoreResult<()> {
    for d in RAILS_DIRS {
        writer.ensure_dir(&dir.join(d))?;
    }
    if fullstack {
        for d in RAILS_FULLSTACK_DIRS {
            writer.ensure_dir(&dir.join(d))?;
        }
    }
    let mut cx = RailsCtx {
        writer,
        dir,
        app_name,
    };

    cx.render_to(
        if fullstack {
            "Gemfile.fullstack.erb"
        } else {
            "Gemfile.erb"
        },
        "Gemfile",
    )?;
    cx.render_to(
        if fullstack {
            "llms.fullstack.txt.erb"
        } else {
            "llms.txt.erb"
        },
        "llms.txt",
    )?;
    cx.render_to("config/dare.yml", "config/dare.yml")?;
    cx.copy_same(".dare/skills.yml")?;
    for f in RAILS_INITIALIZERS {
        cx.copy_same(&format!("config/initializers/{f}"))?;
    }
    cx.copy_same("app/controllers/concerns/problem_details.rb")?;
    cx.copy_to(
        if fullstack {
            "app/controllers/application_controller.fullstack.rb"
        } else {
            "app/controllers/application_controller.rb"
        },
        "app/controllers/application_controller.rb",
    )?;
    cx.copy_same("lib/tasks/dare.rake")?;
    for f in RAILS_USER_EXAMPLE
        .iter()
        .chain(RAILS_LLM)
        .chain(RAILS_CHANNELS)
    {
        cx.copy_same(f)?;
    }
    if fullstack {
        cx.render_to(
            "app/views/layouts/application.html.erb",
            "app/views/layouts/application.html.erb",
        )?;
        cx.render_to(
            "app/views/home/index.html.erb",
            "app/views/home/index.html.erb",
        )?;
        cx.copy_same("app/controllers/home_controller.rb")?;
        cx.copy_same("app/helpers/application_helper.rb")?;
        cx.copy_to("config/routes.fullstack.rb", "config/routes.rb")?;
    }
    if !native_runtime_provided {
        cx.render_to(
            if fullstack {
                "config/application.fullstack.rb"
            } else {
                "config/application.rb"
            },
            "config/application.rb",
        )?;
        cx.render_to("config/database.yml", "config/database.yml")?;
        cx.render_to("config/cable.yml", "config/cable.yml")?;
        cx.render_to("bin/setup", "bin/setup")?;
        for f in RAILS_RUNTIME_COPIED {
            cx.copy_same(f)?;
        }
        if fullstack {
            for f in RAILS_ASSET_RUNTIME {
                cx.copy_same(f)?;
            }
        }
    }
    for f in RAILS_SUMMARIZE {
        cx.copy_same(f)?;
    }
    cx.copy_same(".github/workflows/dare-ci.yml")?;
    cx.copy_same(".env.example")?;
    cx.copy_same("spec/rails_helper.rb")?;
    cx.copy_same("spec/swagger_helper.rb")?;

    if !native_runtime_provided {
        let mut stubs = vec!["bin/rails", "bin/rake", "bin/bundle", "bin/setup"];
        if fullstack {
            stubs.push("bin/dev");
        }
        for stub in stubs {
            let p = dir.join(stub);
            let bytes =
                std::fs::read(&p).map_err(|e| CoreError::io(format!("read {stub}: {e}")))?;
            cx.writer.write_executable(&p, bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_template_is_embedded() {
        for id in [
            "node-nestjs",
            "python-fastapi",
            "php-laravel",
            "rust-axum",
            "go-gin",
            "go-stdlib",
            "mcp-node-ts",
            "mcp-python",
            "mcp-rust",
            "mcp-go",
        ] {
            for (template, _) in template_files(id).expect("table") {
                stack_template(id, template).unwrap_or_else(|e| panic!("{id}: {e}"));
            }
        }
    }

    #[test]
    fn nestjs_scaffold_renders_full_project() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut w = ProjectWriter::new();
        let opts = StackScaffoldOpts {
            stack_id: "node-nestjs",
            project_name: "api-nest",
            is_monorepo: false,
            mcp_transport: None,
        };
        let files = scaffold_registry_stack(&mut w, tmp.path(), &opts).expect("scaffold");
        assert_eq!(files.len(), 31);
        assert!(tmp.path().join("tsconfig.json").is_file());
        assert!(tmp.path().join("src/app.module.ts").is_file());
        let pkg = std::fs::read_to_string(tmp.path().join("package.json")).expect("pkg");
        assert!(pkg.contains("\"api-nest\""), "{pkg}");
        assert!(!pkg.contains("{{"));
    }

    #[test]
    fn env_secret_gate_matches_npm_patterns() {
        assert!(validate_env_example("JWT_SECRET=change-me\n# x=AKIAABCDEFGHIJKLMNOP\n").is_ok());
        let err = validate_env_example("A=1\nKEY=AKIAABCDEFGHIJKLMNOP\n").unwrap_err();
        assert!(
            err.message().starts_with(".env.example line 2:"),
            "{}",
            err.message()
        );
        assert!(validate_env_example(&format!("H={}", "a".repeat(32))).is_err());
    }

    #[test]
    fn rails_names() {
        assert_eq!(
            rails_module_name("dare-labs-plataform"),
            "DareLabsPlataform"
        );
        assert_eq!(rails_module_name("1st-app"), "App1stApp");
        assert_eq!(rails_database_name("My-App"), "my_app");
    }

    #[test]
    fn rails_offline_api_and_overlay() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut w = ProjectWriter::new();
        scaffold_rails(&mut w, tmp.path(), "shop", false, false).expect("offline");
        assert!(tmp.path().join("bin/rails").is_file());
        assert!(tmp.path().join("tmp").is_dir());
        assert!(!tmp.path().join("config/routes.rb").exists());
        let app = std::fs::read_to_string(tmp.path().join("config/application.rb")).expect("app");
        assert!(!app.contains("<%="), "{app}");

        let tmp2 = tempfile::tempdir().expect("tempdir");
        let mut w2 = ProjectWriter::new();
        scaffold_rails(&mut w2, tmp2.path(), "shop", true, true).expect("overlay");
        assert!(tmp2.path().join("config/routes.rb").is_file());
        assert!(!tmp2.path().join("bin/rails").exists());
    }
}
