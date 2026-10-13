//! Stack bootstrap — port of npm `utils/stack-bootstrap.ts` (reachable paths).

use std::path::Path;

use dare_core::{CoreError, CoreResult};
use serde_json::Value;

use super::config::{npm_slug, BackendStack, FrontendStack, McpLanguage, ToolchainMode};
use super::console::{banner, Console, Tone};
use super::ordered_json::OrderedJson;
use super::stacks::{scaffold_rails, scaffold_registry_stack, StackScaffoldOpts};
use super::tools::{StackTool, ToolRunner, ToolSpec};
use super::writer::ProjectWriter;

/// Shared context threaded through every bootstrap step.
pub struct BootstrapCtx<'a> {
    pub writer: &'a mut ProjectWriter,
    pub runner: &'a mut dyn ToolRunner,
    pub console: &'a mut dyn Console,
    /// `DARE_RAILS_OFFLINE=1` (read once by the caller).
    pub rails_offline: bool,
}

/// npm `bootstrapBackend`.
pub fn bootstrap_backend(
    cx: &mut BootstrapCtx<'_>,
    stack: BackendStack,
    dir: &Path,
    project_name: &str,
    mode: ToolchainMode,
    is_monorepo: bool,
    fullstack: bool,
) -> CoreResult<()> {
    if stack == BackendStack::RubyRails8 {
        return bootstrap_ruby_rails(cx, dir, project_name, mode, fullstack);
    }
    bootstrap_via_registry(cx, stack.as_str(), dir, project_name, is_monorepo, None)
}

/// npm `bootstrapMcp`.
pub fn bootstrap_mcp(
    cx: &mut BootstrapCtx<'_>,
    language: McpLanguage,
    dir: &Path,
    project_name: &str,
    transport: &str,
) -> CoreResult<()> {
    let stack_id = match language {
        McpLanguage::NodeTs => "mcp-node-ts",
        McpLanguage::Python => "mcp-python",
        McpLanguage::Rust => "mcp-rust",
        McpLanguage::Go => "mcp-go",
    };
    bootstrap_via_registry(cx, stack_id, dir, project_name, false, Some(transport))
}

fn bootstrap_via_registry(
    cx: &mut BootstrapCtx<'_>,
    stack_id: &str,
    dir: &Path,
    project_name: &str,
    is_monorepo: bool,
    mcp_transport: Option<&str>,
) -> CoreResult<()> {
    banner(
        cx.console,
        &format!("Scaffolding {stack_id} (DARE-shaped) in {}", dir.display()),
    );
    let opts = StackScaffoldOpts {
        stack_id,
        project_name,
        is_monorepo,
        mcp_transport,
    };
    scaffold_registry_stack(cx.writer, dir, &opts).map(|_| ())
}

/// npm `bootstrapRubyRails` — `rails new` (native/Docker) + DARE overlay, or
/// the offline template runtime when no toolchain exists.
fn bootstrap_ruby_rails(
    cx: &mut BootstrapCtx<'_>,
    dir: &Path,
    project_name: &str,
    mode: ToolchainMode,
    fullstack: bool,
) -> CoreResult<()> {
    banner(
        cx.console,
        &format!(
            "Bootstrapping Rails 8 ({}) in {}",
            if fullstack { "full-stack" } else { "API-only" },
            dir.display()
        ),
    );
    let mut flags = vec![
        "--database=postgresql",
        "--skip-bundle",
        "--skip-git",
        "--skip-test",
        "--skip-ci",
    ];
    if !fullstack {
        flags.push("--api");
    }

    let force_offline = cx.rails_offline;
    let has_native_rails = !force_offline && cx.runner.has_command("rails");
    let has_docker = !force_offline && cx.runner.has_command("docker");

    #[derive(PartialEq)]
    enum Plan {
        Native,
        Docker,
        Offline,
    }
    let plan = if force_offline {
        Plan::Offline
    } else if mode == ToolchainMode::Docker {
        Plan::Docker
    } else if has_native_rails {
        Plan::Native
    } else if mode == ToolchainMode::Auto && has_docker {
        Plan::Docker
    } else {
        Plan::Offline
    };

    if plan == Plan::Offline {
        let reason = if mode == ToolchainMode::Native {
            "--toolchain=native selected but `rails` is not on PATH"
        } else {
            "neither `rails` nor Docker is available"
        };
        cx.console.log(
            Tone::Yellow,
            &format!(
                "⚠  {reason} — using DARE's offline Rails templates (less complete than `rails new`).\n   Install Ruby+Rails or Docker, or re-run with --toolchain docker, for a full app."
            ),
        );
        return scaffold_rails(cx.writer, dir, project_name, fullstack, false);
    }

    let tool = StackTool::resolve(
        ToolSpec {
            native_cmd: "rails",
            native_hint:
                "Install Ruby 3.3+ and Rails 8: https://rubyonrails.org/ (gem install rails)".into(),
            docker_image: "ruby:3.3",
            image_has_entrypoint: false,
        },
        dir,
        if plan == Plan::Native {
            ToolchainMode::Native
        } else {
            ToolchainMode::Docker
        },
        &*cx.runner,
        cx.console,
    )?;

    if tool.using_docker() {
        let rails_new = format!("rails new . {}", flags.join(" "));
        let script = format!(
            "export GEM_HOME=/tmp/.dare-gems GEM_PATH=/tmp/.dare-gems PATH=\"/tmp/.dare-gems/bin:$PATH\" && gem install rails -v '~> 8.0' --no-document && {rails_new}"
        );
        tool.run_other(cx.runner, cx.console, "sh", &["-c", &script])?;
    } else {
        let mut args = vec!["new", "."];
        args.extend(flags.iter().copied());
        tool.run(cx.runner, cx.console, &args)?;
    }
    scaffold_rails(cx.writer, dir, project_name, fullstack, true)
}

/// npm `bootstrapFrontend`.
pub fn bootstrap_frontend(
    cx: &mut BootstrapCtx<'_>,
    stack: FrontendStack,
    dir: &Path,
    project_name: &str,
    mode: ToolchainMode,
    is_monorepo: bool,
) -> CoreResult<()> {
    match stack {
        FrontendStack::React => {
            bootstrap_vite(cx, dir, "react-ts", mode)?;
            try_rename_npm_project(cx, dir, project_name)
        }
        FrontendStack::Vue => {
            bootstrap_vite(cx, dir, "vue-ts", mode)?;
            try_rename_npm_project(cx, dir, project_name)
        }
        FrontendStack::RustLeptos => {
            bootstrap_leptos_fullstack(cx, dir, project_name, mode, is_monorepo)
        }
        FrontendStack::RustLeptosCsr => {
            bootstrap_leptos_csr(cx, dir, project_name, mode, is_monorepo)
        }
        FrontendStack::None => Err(CoreError::invalid_input("Unknown frontend stack: none")),
    }
}

fn bootstrap_vite(
    cx: &mut BootstrapCtx<'_>,
    dir: &Path,
    template: &str,
    mode: ToolchainMode,
) -> CoreResult<()> {
    banner(
        cx.console,
        &format!("Bootstrapping Vite ({template}) in {}", dir.display()),
    );
    let npm = StackTool::resolve(
        ToolSpec {
            native_cmd: "npm",
            native_hint: "Install Node.js: https://nodejs.org/".into(),
            docker_image: "node:20-alpine",
            image_has_entrypoint: false,
        },
        dir,
        mode,
        &*cx.runner,
        cx.console,
    )?;
    let source = format!("vitejs/vite/packages/create-vite/template-{template}");
    npm.run_other(
        cx.runner,
        cx.console,
        "npx",
        &["-y", "degit", &source, ".", "--force"],
    )?;
    npm.run(cx.runner, cx.console, &["install"])
}

/// npm `tryRenameNpmProject` — rewrite `package.json` `name` (2-space JSON).
fn try_rename_npm_project(
    cx: &mut BootstrapCtx<'_>,
    dir: &Path,
    project_name: &str,
) -> CoreResult<()> {
    let pj = dir.join("package.json");
    let Ok(raw) = std::fs::read_to_string(&pj) else {
        return Ok(());
    };
    let Ok(mut data) = serde_json::from_str::<OrderedJson>(&raw) else {
        return Ok(());
    };
    let name = OrderedJson::Scalar(Value::String(npm_package_name(project_name)));
    if !data.set("name", name) {
        return Ok(());
    }
    if let Ok(body) = serde_json::to_string_pretty(&data) {
        cx.writer.write(&pj, format!("{body}\n"))?;
    }
    Ok(())
}

/// `projectName.toLowerCase().replace(/[^a-z0-9-]+/g, '-')` (no trim).
fn npm_package_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut out = String::new();
    let mut in_run = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
            out.push(ch);
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    out
}

/// npm `sanitizeCrateName`.
fn sanitize_crate_name(name: &str) -> String {
    let s = npm_slug(name);
    if s.is_empty() {
        "app".into()
    } else {
        s
    }
}

fn cargo_spec(extra_hint: &str) -> ToolSpec {
    ToolSpec {
        native_cmd: "cargo",
        native_hint: [
            "Install Rust toolchain: https://www.rust-lang.org/tools/install",
            "Then: rustup target add wasm32-unknown-unknown",
            extra_hint,
        ]
        .join("\n  "),
        docker_image: "ghcr.io/dewtech-technologies/dare-rust-leptos:1",
        image_has_entrypoint: false,
    }
}

const LEPTOS_STYLE: &str = ":root {
  font-family: system-ui, sans-serif;
}

body {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}

h1 {
  color: #b7410e;
}

button {
  padding: 0.5rem 1rem;
  font-size: 1rem;
  cursor: pointer;
}
";

fn leptos_ssr_cargo_toml(crate_name: &str) -> String {
    [
        "[package]".to_string(),
        format!("name = \"{crate_name}\""),
        "version = \"0.1.0\"".into(),
        "edition = \"2021\"".into(),
        String::new(),
        "[lib]".into(),
        "crate-type = [\"cdylib\", \"rlib\"]".into(),
        String::new(),
        "[dependencies]".into(),
        "axum = { version = \"0.7\", optional = true }".into(),
        "console_error_panic_hook = \"0.1\"".into(),
        "leptos = { version = \"0.7\", features = [] }".into(),
        "leptos_axum = { version = \"0.7\", optional = true }".into(),
        "leptos_meta = { version = \"0.7\" }".into(),
        "leptos_router = { version = \"0.7\", features = [] }".into(),
        "tokio = { version = \"1\", features = [\"full\"], optional = true }".into(),
        "tower = { version = \"0.5\", optional = true }".into(),
        "tower-http = { version = \"0.6\", features = [\"fs\"], optional = true }".into(),
        "wasm-bindgen = \"0.2\"".into(),
        String::new(),
        "[features]".into(),
        "default = []".into(),
        "hydrate = [\"leptos/hydrate\"]".into(),
        "ssr = [".into(),
        "  \"dep:axum\",".into(),
        "  \"dep:leptos_axum\",".into(),
        "  \"dep:tokio\",".into(),
        "  \"dep:tower\",".into(),
        "  \"dep:tower-http\",".into(),
        "  \"leptos/ssr\",".into(),
        "  \"leptos_meta/ssr\",".into(),
        "  \"leptos_router/ssr\",".into(),
        "]".into(),
        String::new(),
        "[package.metadata.leptos]".into(),
        format!("output-name = \"{crate_name}\""),
        "site-root = \"target/site\"".into(),
        "site-pkg-dir = \"pkg\"".into(),
        "style-file = \"style/main.scss\"".into(),
        "browserquery = \"defaults\"".into(),
        "env = \"DEV\"".into(),
        "bin-features = [\"ssr\"]".into(),
        "lib-features = [\"hydrate\"]".into(),
        String::new(),
        "[[bin]]".into(),
        format!("name = \"{crate_name}\""),
        "path = \"src/main.rs\"".into(),
        "required-features = [\"ssr\"]".into(),
        String::new(),
        "[profile.release]".into(),
        "codegen-units = 1".into(),
        "lto = true".into(),
        String::new(),
        "[profile.wasm-release]".into(),
        "inherits = \"release\"".into(),
        "opt-level = \"z\"".into(),
        String::new(),
    ]
    .join("\n")
}

const LEPTOS_SSR_LIB: &str = "pub mod app;

#[cfg(feature = \"hydrate\")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
";

const LEPTOS_SSR_APP: &str = "use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::Router, path};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang=\"en\">
            <head>
                <meta charset=\"UTF-8\"/>
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text=\"__PROJECT_NAME__\"/>
        <Router>
            <main>
                <Routes fallback=|| view! { <p>\"Not found.\"</p> }>
                    <Route path=path!(\"/\") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <h1>\"__PROJECT_NAME__\"</h1>
        <button on:click=move |_| set_count.update(|n| *n += 1)>
            \"Count: \" {count}
        </button>
    }
}
";

const LEPTOS_SSR_MAIN: &str = "#[cfg(feature = \"ssr\")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use __CRATE_IDENT__::app::{shell, App};

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    println!(\"Listening on http://{}\", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = \"ssr\"))]
pub fn main() {}
";

fn bootstrap_leptos_fullstack(
    cx: &mut BootstrapCtx<'_>,
    dir: &Path,
    project_name: &str,
    mode: ToolchainMode,
    is_monorepo: bool,
) -> CoreResult<()> {
    banner(
        cx.console,
        &format!(
            "Bootstrapping Leptos fullstack (SSR + WASM) in {}",
            dir.display()
        ),
    );
    let cargo = StackTool::resolve(
        cargo_spec("      cargo install cargo-leptos --locked --version 0.2.22"),
        dir,
        mode,
        &*cx.runner,
        cx.console,
    )?;
    let crate_name = sanitize_crate_name(project_name);
    let crate_ident = crate_name.replace('-', "_");

    cx.writer.ensure_dir(&dir.join("src"))?;
    cx.writer.ensure_dir(&dir.join("style"))?;
    cx.writer.ensure_dir(&dir.join("public"))?;
    // B-08: npm always creates `.cargo/`, leaving it empty inside a monorepo.
    if !is_monorepo {
        cx.writer.ensure_dir(&dir.join(".cargo"))?;
    }
    cx.writer
        .write(&dir.join("Cargo.toml"), leptos_ssr_cargo_toml(&crate_name))?;
    cx.writer.write(&dir.join("src/lib.rs"), LEPTOS_SSR_LIB)?;
    cx.writer.write(
        &dir.join("src/app.rs"),
        LEPTOS_SSR_APP.replace("__PROJECT_NAME__", project_name),
    )?;
    cx.writer.write(
        &dir.join("src/main.rs"),
        LEPTOS_SSR_MAIN.replace("__CRATE_IDENT__", &crate_ident),
    )?;
    cx.writer
        .write(&dir.join("style/main.scss"), LEPTOS_STYLE)?;
    if !is_monorepo {
        cx.writer.write(
            &dir.join(".cargo/config.toml"),
            "# DARE: Do NOT add a global [build] target here.
# Mixed workspaces (Leptos WASM + native crates like napi-rs or aya) will break
# if cargo tries to compile all crates for the same target.
# cargo-leptos manages wasm32-unknown-unknown target per-crate automatically.
",
        )?;
    }
    cargo.run(cx.runner, cx.console, &["fetch"])?;
    remove_monorepo_lock(dir, is_monorepo)
}

fn remove_monorepo_lock(dir: &Path, is_monorepo: bool) -> CoreResult<()> {
    if is_monorepo {
        let lock = dir.join("Cargo.lock");
        if lock.exists() {
            std::fs::remove_file(&lock)
                .map_err(|e| CoreError::io(format!("remove Cargo.lock: {e}")))?;
        }
    }
    Ok(())
}

fn leptos_csr_cargo_toml(crate_name: &str) -> String {
    [
        "[package]".to_string(),
        format!("name = \"{crate_name}\""),
        "version = \"0.1.0\"".into(),
        "edition = \"2021\"".into(),
        String::new(),
        "[lib]".into(),
        "crate-type = [\"cdylib\", \"rlib\"]".into(),
        String::new(),
        "[dependencies]".into(),
        "leptos = { version = \"0.7\", features = [\"csr\"] }".into(),
        "console_error_panic_hook = \"0.1\"".into(),
        "wasm-bindgen = \"0.2\"".into(),
        String::new(),
        "[profile.release]".into(),
        "lto = true".into(),
        "opt-level = \"z\"".into(),
        String::new(),
    ]
    .join("\n")
}

const LEPTOS_CSR_LIB: &str = "use leptos::prelude::*;

pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <main>
            <h1>\"__PROJECT_NAME__\"</h1>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                \"Count: \" {count}
            </button>
        </main>
    }
}
";

const LEPTOS_CSR_INDEX: &str = "<!doctype html>
<html lang=\"en\">
  <head>
    <meta charset=\"UTF-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
    <title>__PROJECT_NAME__</title>
    <link data-trunk rel=\"scss\" href=\"style/main.scss\" />
    <link data-trunk rel=\"rust\" href=\"Cargo.toml\" data-wasm-opt=\"z\" />
  </head>
  <body></body>
</html>
";

const TRUNK_TOML: &str = "[build]
target = \"index.html\"
dist = \"dist\"

[watch]
ignore = [\"./dist\"]

[serve]
port = 3001
open = false
";

fn bootstrap_leptos_csr(
    cx: &mut BootstrapCtx<'_>,
    dir: &Path,
    project_name: &str,
    mode: ToolchainMode,
    is_monorepo: bool,
) -> CoreResult<()> {
    banner(
        cx.console,
        &format!(
            "Bootstrapping Leptos CSR (WASM + trunk) in {}",
            dir.display()
        ),
    );
    let cargo = StackTool::resolve(
        cargo_spec("      cargo install trunk --locked"),
        dir,
        mode,
        &*cx.runner,
        cx.console,
    )?;
    let crate_name = sanitize_crate_name(project_name);

    cx.writer.ensure_dir(&dir.join("src"))?;
    cx.writer.ensure_dir(&dir.join("style"))?;
    if !is_monorepo {
        cx.writer.ensure_dir(&dir.join(".cargo"))?;
    }
    cx.writer
        .write(&dir.join("Cargo.toml"), leptos_csr_cargo_toml(&crate_name))?;
    // npm uses String.replace with a string pattern here: first occurrence only.
    cx.writer.write(
        &dir.join("src/lib.rs"),
        LEPTOS_CSR_LIB.replacen("__PROJECT_NAME__", project_name, 1),
    )?;
    cx.writer.write(
        &dir.join("index.html"),
        LEPTOS_CSR_INDEX.replace("__PROJECT_NAME__", project_name),
    )?;
    cx.writer.write(&dir.join("Trunk.toml"), TRUNK_TOML)?;
    cx.writer
        .write(&dir.join("style/main.scss"), LEPTOS_STYLE)?;
    if !is_monorepo {
        cx.writer.write(
            &dir.join(".cargo/config.toml"),
            "# DARE: Do NOT add a global [build] target here.
# trunk manages wasm32-unknown-unknown target automatically via its own build pipeline.
",
        )?;
    }
    cargo.run(cx.runner, cx.console, &["fetch"])?;
    remove_monorepo_lock(dir, is_monorepo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::console::MemoryConsole;
    use crate::project::tools::RecordingToolRunner;

    #[test]
    fn rails_plans() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut w = ProjectWriter::new();
        let mut r = RecordingToolRunner::with_tools(&["docker"]);
        let mut c = MemoryConsole::default();
        let mut cx = BootstrapCtx {
            writer: &mut w,
            runner: &mut r,
            console: &mut c,
            rails_offline: false,
        };
        bootstrap_backend(
            &mut cx,
            BackendStack::RubyRails8,
            tmp.path(),
            "shop",
            ToolchainMode::Auto,
            false,
            true,
        )
        .expect("rails docker");
        let argv = &r.commands[0].argv;
        assert_eq!(argv[0], "docker");
        assert!(argv.iter().any(|a| a == "ruby:3.3"));
        assert!(argv.last().unwrap().ends_with(
            "rails new . --database=postgresql --skip-bundle --skip-git --skip-test --skip-ci"
        ));

        let tmp2 = tempfile::tempdir().expect("tempdir");
        let mut w2 = ProjectWriter::new();
        let mut r2 = RecordingToolRunner::with_tools(&["rails", "docker"]);
        let mut c2 = MemoryConsole::default();
        let mut cx2 = BootstrapCtx {
            writer: &mut w2,
            runner: &mut r2,
            console: &mut c2,
            rails_offline: true,
        };
        bootstrap_backend(
            &mut cx2,
            BackendStack::RubyRails8,
            tmp2.path(),
            "shop",
            ToolchainMode::Auto,
            false,
            false,
        )
        .expect("offline");
        assert!(r2.commands.is_empty());
        assert!(tmp2.path().join("bin/rails").exists());
    }

    #[test]
    fn vite_commands_and_rename() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("package.json"),
            "{\n  \"name\": \"vite-project\",\n  \"private\": true\n}\n",
        )
        .unwrap();
        let mut w = ProjectWriter::new();
        let mut r = RecordingToolRunner::with_tools(&["npm"]);
        let mut c = MemoryConsole::default();
        let mut cx = BootstrapCtx {
            writer: &mut w,
            runner: &mut r,
            console: &mut c,
            rails_offline: false,
        };
        bootstrap_frontend(
            &mut cx,
            FrontendStack::Vue,
            tmp.path(),
            "My App",
            ToolchainMode::Native,
            true,
        )
        .expect("vue");
        assert_eq!(
            r.commands[0].argv,
            [
                "npx",
                "-y",
                "degit",
                "vitejs/vite/packages/create-vite/template-vue-ts",
                ".",
                "--force"
            ]
        );
        assert_eq!(r.commands[1].argv, ["npm", "install"]);
        let pkg = std::fs::read_to_string(tmp.path().join("package.json")).unwrap();
        assert_eq!(pkg, "{\n  \"name\": \"my-app\",\n  \"private\": true\n}\n");
    }

    #[test]
    fn leptos_monorepo_skips_cargo_config() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut w = ProjectWriter::new();
        let mut r = RecordingToolRunner::with_tools(&["cargo"]);
        let mut c = MemoryConsole::default();
        let mut cx = BootstrapCtx {
            writer: &mut w,
            runner: &mut r,
            console: &mut c,
            rails_offline: false,
        };
        bootstrap_frontend(
            &mut cx,
            FrontendStack::RustLeptos,
            tmp.path(),
            "web",
            ToolchainMode::Native,
            true,
        )
        .expect("leptos");
        assert!(!tmp.path().join(".cargo").exists());
        assert_eq!(r.commands[0].argv, ["cargo", "fetch"]);
        let main = std::fs::read_to_string(tmp.path().join("src/main.rs")).unwrap();
        assert!(main.contains("use web::app::{shell, App};"));
    }
}
