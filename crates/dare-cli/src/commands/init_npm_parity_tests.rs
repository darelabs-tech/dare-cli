//! Parity suite: Rust `dare init` vs npm `@dewtech/dare-cli@3.18.1` goldens
//! (cycle rust-init-npm-parity-011, task-010).
//!
//! For every case in `tests/fixtures/golden/init-npm/` this checks:
//! 1. the interactive question tree (`prompts.json`) — messages, choices, defaults;
//! 2. the resolved config (`case.json` `generator_config`);
//! 3. the file tree incl. empty dirs (`tree.txt`);
//! 4. every file hash (`hashes.txt`) and full content of code-generated files;
//! 5. external commands (`commands.jsonl`) and console output (`stdout.txt`).
//!
//! The documented bug fixes B-01..B-10 are applied as explicit expected patches.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use dare_assets::sha256_hex;
use dare_core::{CoreError, CoreResult};
use dare_scaffold::project::console::MemoryConsole;
use dare_scaffold::project::generator::FRAMEWORK_VERSION;
use dare_scaffold::project::tools::RecordingToolRunner;
use dare_scaffold::project::{default_crate_prefix, ProjectConfig};
use serde_json::Value;

use crate::commands::init::{
    execute_init_with, is_non_interactive, resolve_interactive, resolve_non_interactive,
    success_message, InitCliOpts, INIT_HEADER,
};
use crate::commands::init_prompts::Prompter;

const TOOLS: &[&str] = &[
    "npx", "npm", "cargo", "rails", "docker", "gem", "composer", "python", "go",
];

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden/init-npm")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

// ─── Golden-driven prompter ─────────────────────────────────────────────────

/// Answers from `prompts.json` and asserts every question matches npm.
struct GoldenPrompter {
    slug: String,
    project_name: String,
    asked: Vec<Value>,
    next: usize,
}

impl GoldenPrompter {
    fn new(slug: &str, project_name: &str, prompts: &Value) -> Self {
        let asked = prompts
            .as_array()
            .expect("prompts array")
            .iter()
            .filter(|q| q["asked"] == Value::Bool(true))
            .cloned()
            .collect();
        Self {
            slug: slug.to_string(),
            project_name: project_name.to_string(),
            asked,
            next: 0,
        }
    }

    fn take(&mut self, kind: &str, message: &str) -> Value {
        let q = self
            .asked
            .get(self.next)
            .unwrap_or_else(|| panic!("[{}] unexpected extra question '{message}'", self.slug))
            .clone();
        self.next += 1;
        assert_eq!(
            q["message"], message,
            "[{}] question #{} message",
            self.slug, self.next
        );
        assert_eq!(q["type"], kind, "[{}] '{message}' type", self.slug);
        q
    }

    fn assert_done(&self) {
        assert_eq!(
            self.next,
            self.asked.len(),
            "[{}] npm asked more questions: {:?}",
            self.slug,
            self.asked[self.next..]
                .iter()
                .map(|q| q["message"].clone())
                .collect::<Vec<_>>()
        );
    }
}

fn choice_labels(q: &Value) -> Vec<String> {
    q["choices"]
        .as_array()
        .expect("choices")
        .iter()
        .map(|c| c["name"].as_str().expect("name").to_string())
        .collect()
}

fn choice_index(q: &Value, value: &Value) -> usize {
    q["choices"]
        .as_array()
        .expect("choices")
        .iter()
        .position(|c| &c["value"] == value)
        .unwrap_or_else(|| panic!("no choice with value {value} in {q}"))
}

impl Prompter for GoldenPrompter {
    fn input(
        &mut self,
        message: &str,
        default: &str,
        validate: &dyn Fn(&str) -> Result<(), String>,
    ) -> CoreResult<String> {
        let q = self.take("input", message);
        let mut want_default = q["default"].as_str().unwrap_or_default().to_string();
        if q["name"] == "cratePrefix" {
            // B-10: npm derives the default from `answers.name`, which is unset when
            // the name is positional, so it always falls back to "app".
            want_default = default_crate_prefix(&self.project_name);
        }
        assert_eq!(default, want_default, "[{}] '{message}' default", self.slug);
        let answer = q["answer"].as_str().expect("input answer").to_string();
        validate(&answer).map_err(CoreError::invalid_input)?;
        Ok(answer)
    }

    fn select(&mut self, message: &str, items: &[&str], default: usize) -> CoreResult<usize> {
        let q = self.take("list", message);
        assert_eq!(
            items,
            choice_labels(&q),
            "[{}] '{message}' choices",
            self.slug
        );
        let want_default = q.get("default").map_or(0, |d| choice_index(&q, d));
        assert_eq!(default, want_default, "[{}] '{message}' default", self.slug);
        Ok(choice_index(&q, &q["answer"]))
    }

    fn checkbox(
        &mut self,
        message: &str,
        items: &[&str],
        checked: &[bool],
    ) -> CoreResult<Vec<usize>> {
        let q = self.take("checkbox", message);
        assert_eq!(
            items,
            choice_labels(&q),
            "[{}] '{message}' choices",
            self.slug
        );
        let want_checked: Vec<bool> = q["choices"]
            .as_array()
            .expect("choices")
            .iter()
            .map(|c| c["checked"].as_bool().unwrap_or(false))
            .collect();
        assert_eq!(checked, want_checked, "[{}] '{message}' checked", self.slug);
        Ok(q["answer"]
            .as_array()
            .expect("checkbox answer")
            .iter()
            .map(|v| choice_index(&q, v))
            .collect())
    }

    fn confirm(&mut self, message: &str, default: bool) -> CoreResult<bool> {
        let q = self.take("confirm", message);
        assert_eq!(
            Value::Bool(default),
            q["default"],
            "[{}] '{message}' default",
            self.slug
        );
        Ok(q["answer"].as_bool().expect("confirm answer"))
    }
}

// ─── Inputs ──────────────────────────────────────────────────────────────────

fn cli_opts(command: &[String]) -> InitCliOpts {
    // command = ["dare", "init", <name>, flags...]
    let mut o = InitCliOpts {
        name: command.get(2).cloned(),
        ..InitCliOpts::default()
    };
    let mut it = command.iter().skip(3);
    while let Some(flag) = it.next() {
        let mut val = || it.next().cloned();
        match flag.as_str() {
            "--stack" => o.stack = val(),
            "--mcp" => o.mcp = val(),
            "--transport" => o.transport = val(),
            "--toolchain" => o.toolchain = val(),
            "--fullstack" => o.fullstack = Some(String::new()),
            "--non-interactive" => o.non_interactive = true,
            other => panic!("unsupported golden flag {other}"),
        }
    }
    o
}

fn config_as_generator_json(cfg: &ProjectConfig) -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    let mut put = |k: &str, v: Option<Value>| {
        if let Some(v) = v {
            m.insert(k.to_string(), v);
        }
    };
    let s = |x: &str| Some(Value::String(x.to_string()));
    put("name", s(&cfg.name));
    put("structure", s(cfg.structure.as_str()));
    put("backend", cfg.backend.and_then(|b| s(b.as_str())));
    put("frontend", cfg.frontend.and_then(|f| s(f.as_str())));
    put(
        "mcpTransport",
        cfg.mcp_transport.and_then(|t| s(t.as_str())),
    );
    put("mcpLanguage", cfg.mcp_language.and_then(|l| s(l.as_str())));
    put(
        "mcpFeatures",
        cfg.mcp_features.as_ref().map(|fs| {
            Value::Array(
                fs.iter()
                    .map(|f| Value::String(f.as_str().into()))
                    .collect(),
            )
        }),
    );
    put("ide", s(cfg.ide.as_str()));
    put("graphrag", s(cfg.graphrag.as_str()));
    put("mcp", Some(Value::Bool(cfg.mcp)));
    put("toolchain", s(cfg.toolchain.as_str()));
    put(
        "rustWorkspaceLayout",
        cfg.rust_workspace_layout.and_then(|l| s(l.as_str())),
    );
    put("cratePrefix", cfg.crate_prefix.as_deref().and_then(s));
    put("outputDir", s(&format!("<case-root>/{}", cfg.name)));
    m
}

// ─── Output capture ─────────────────────────────────────────────────────────

/// Sorted files + empty dirs (trailing `/`), posix, relative to `root`.
fn tree(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("read_dir")
            .map(|e| e.expect("entry").path())
            .collect();
        entries.sort();
        if entries.is_empty() && dir != root {
            let rel = dir
                .strip_prefix(root)
                .expect("rel")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(format!("{rel}/"));
        }
        for p in entries {
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                out.push(
                    p.strip_prefix(root)
                        .expect("rel")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn content_hash(bytes: &[u8]) -> String {
    if bytes.contains(&0) {
        sha256_hex(bytes)
    } else {
        sha256_hex(lf(&String::from_utf8_lossy(bytes)).as_bytes())
    }
}

struct Normalizer {
    roots: Vec<String>,
}

impl Normalizer {
    fn new(case_root: &Path) -> Self {
        let native = case_root.to_string_lossy().into_owned();
        let forward = native.replace('\\', "/");
        Self {
            roots: vec![native, forward],
        }
    }

    fn apply(&self, s: &str) -> String {
        s.lines()
            .map(|line| {
                let mut l = line.to_string();
                let mut hit = false;
                for r in &self.roots {
                    if l.contains(r.as_str()) {
                        l = l.replace(r.as_str(), "<case-root>");
                        hit = true;
                    }
                }
                if hit {
                    l = l.replace('\\', "/");
                }
                l
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ─── Expected patches (B-xx) ────────────────────────────────────────────────

/// B-03: npm writes double-encoded UTF-8 in generated text.
fn fix_mojibake(s: &str) -> String {
    s.replace("âœ…", "✅")
        .replace("â†’", "→")
        .replace("â€”", "—")
}

fn patch_stored(slug: &str, rel: &str, golden: &str) -> String {
    let mut s = fix_mojibake(&lf(golden));
    if rel == "dare.config.json" {
        s = s.replace(
            "\"version\": \"3.18.1\"",
            &format!("\"version\": \"{FRAMEWORK_VERSION}\""),
        );
    }
    if slug == "mcp-rust-http-claude-code" && rel == ".claude/settings.json" {
        // B-02: MCP settings use the server language's commands.
        s = s
            .replace("python -m py_compile main.py", "cargo build")
            .replace("Bash(pytest)", "Bash(cargo test)")
            .replace("ruff check .", "cargo clippy")
            .replace("&& pytest &&", "&& cargo test &&");
    }
    s
}

fn patch_tree(slug: &str, mut expected: Vec<String>) -> Vec<String> {
    // B-05: no skill-none.mdc; B-08: no empty `.cargo/` in Leptos members.
    expected.retain(|p| {
        p != ".cursor/rules/skill-none.mdc"
            && !(p.starts_with("crates/") && p.ends_with("/.cargo/"))
    });
    let _ = slug;
    expected
}

fn patch_stdout(slug: &str, golden: &str) -> String {
    let mut s = lf(golden);
    if slug == "mcp-rust-http-claude-code" {
        // B-06: Rust MCP next steps.
        s = s
            .replace("  2. npm install\n", "  2. cargo build\n")
            .replace(
                "  6. npm run inspect  ← test with MCP Inspector\n",
                "  6. npx @modelcontextprotocol/inspector cargo run  ← test with MCP Inspector\n",
            )
            .replace("  7. npm run dev\n", "  7. cargo run\n");
    }
    if slug == "monorepo-rust-axum-leptos-csr-multi-ars-cursor" {
        // B-07: multi-crate line shows the chosen prefix.
        s = s.replace(
            "crates/arsenal-web-core  |  crates/arsenal-web-server  |  crates/arsenal-web-web  |  crates/arsenal-web-cli",
            "crates/ars-core  |  crates/ars-server  |  crates/ars-web  |  crates/ars-cli",
        );
    }
    s
}

/// B-01: `claude-hybrid` also gets the Cursor set; strip it before comparing.
fn split_b01_extras(slug: &str, actual: Vec<String>) -> (Vec<String>, Vec<String>) {
    if slug != "monorepo-rust-axum-leptos-single-claude-hybrid" {
        return (actual, Vec::new());
    }
    actual
        .into_iter()
        .partition(|p| !(p == ".cursorrules" || p.starts_with(".cursor/")))
}

// ─── The suite ──────────────────────────────────────────────────────────────

fn run_case(slug: &str) -> Vec<String> {
    let dir = fixtures_root().join(slug);
    let case: Value = serde_json::from_str(&read(&dir.join("case.json"))).expect("case.json");
    let command: Vec<String> = case["command"]
        .as_array()
        .expect("command")
        .iter()
        .map(|v| v.as_str().expect("arg").to_string())
        .collect();
    let project_dir = case["project_dir"].as_str().expect("project_dir");
    let interactive = case["mode"] == "interactive";
    let mut failures = Vec::new();

    let tmp = tempfile::tempdir().expect("tempdir");
    let case_root = tmp.path().to_path_buf();
    let opts = cli_opts(&command);

    // 1–2. prompts + resolved config
    let cfg = if interactive {
        let prompts: Value =
            serde_json::from_str(&read(&dir.join("prompts.json"))).expect("prompts.json");
        let mut p = GoldenPrompter::new(slug, project_dir, &prompts);
        let cfg = resolve_interactive(&opts, &case_root, &mut p).expect("resolve interactive");
        p.assert_done();
        cfg
    } else {
        resolve_non_interactive(&opts, &case_root).expect("resolve cli")
    };
    assert_eq!(interactive, !is_non_interactive(&opts), "[{slug}] mode");
    let want_cfg: BTreeMap<String, Value> =
        serde_json::from_value(case["generator_config"].clone()).expect("generator_config");
    let got_cfg = config_as_generator_json(&cfg);
    if got_cfg != want_cfg {
        failures.push(format!("config: got {got_cfg:?}\n  want {want_cfg:?}"));
    }

    // Generate with recorded tools (npm goldens ran on POSIX → `--user`).
    let mut runner = RecordingToolRunner::with_tools(TOOLS);
    runner.docker_user = Some("<uid>:<gid>".into());
    let mut console = MemoryConsole::default();
    let rails_offline = case["env"]["DARE_RAILS_OFFLINE"] == "1";
    execute_init_with(
        &cfg,
        &case_root,
        false,
        rails_offline,
        &mut runner,
        &mut console,
    )
    .unwrap_or_else(|e| panic!("[{slug}] init failed: {e}"));
    let root = case_root.join(project_dir);
    let norm = Normalizer::new(&case_root);

    // 3. tree
    let expected_tree = patch_tree(
        slug,
        read(&dir.join("tree.txt"))
            .lines()
            .map(str::to_string)
            .collect(),
    );
    let (actual_tree, b01_extras) = split_b01_extras(slug, tree(&root));
    if !b01_extras.is_empty() {
        for must in [
            ".cursorrules",
            ".cursor/rules/skill-rust-axum.mdc",
            ".cursor/rules/skill-rust-leptos.mdc",
        ] {
            if !b01_extras.iter().any(|p| p == must) {
                failures.push(format!("B-01: missing {must}"));
            }
        }
    } else if slug == "monorepo-rust-axum-leptos-single-claude-hybrid" {
        failures.push("B-01: Cursor files not installed for claude-hybrid".into());
    }
    let want: BTreeSet<_> = expected_tree.iter().cloned().collect();
    let got: BTreeSet<_> = actual_tree.iter().cloned().collect();
    let missing: Vec<_> = want.difference(&got).collect();
    let extra: Vec<_> = got.difference(&want).collect();
    if !missing.is_empty() || !extra.is_empty() {
        failures.push(format!("tree: missing {missing:?}\n  extra {extra:?}"));
    }

    // 4. contents
    let stored: BTreeSet<String> = case["stored_files"]
        .as_array()
        .expect("stored_files")
        .iter()
        .map(|v| v.as_str().expect("path").to_string())
        .collect();
    for line in read(&dir.join("hashes.txt")).lines() {
        let (hash, rel) = line.split_once("  ").expect("hash line");
        if !got.contains(rel) {
            continue; // reported by the tree check (or removed by a B-xx patch)
        }
        let bytes = std::fs::read(root.join(rel)).expect("read generated");
        if stored.contains(rel) {
            let golden = read(&dir.join("files").join(format!("{rel}.golden")));
            let want = patch_stored(slug, rel, &golden);
            let got_s = lf(&String::from_utf8_lossy(&bytes));
            if got_s != want {
                failures.push(format!(
                    "content {rel}:\n--- want\n{want}\n--- got\n{got_s}"
                ));
            }
        } else if content_hash(&bytes) != hash {
            failures.push(format!("hash {rel}"));
        }
    }

    // 5a. commands
    let want_cmds: Vec<Value> = read(&dir.join("commands.jsonl"))
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("command json"))
        .collect();
    let got_cmds: Vec<Value> = runner
        .commands
        .iter()
        .map(|c| {
            let cwd = c.cwd.strip_prefix(&case_root).expect("cwd under case root");
            serde_json::json!({
                "cwd": cwd.to_string_lossy().replace('\\', "/"),
                "argv": c.argv.iter().map(|a| norm.apply(a)).collect::<Vec<_>>(),
            })
        })
        .collect();
    if got_cmds != want_cmds {
        failures.push(format!(
            "commands: got {got_cmds:#?}\n  want {want_cmds:#?}"
        ));
    }

    // 5b. console output (header, spinner, scaffold log, success / next steps)
    let mut out = String::new();
    if interactive {
        out.push_str(&format!("{INIT_HEADER}\n"));
    }
    out.push_str(&format!("- Creating project {}...\n", cfg.name));
    for line in &console.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&success_message(&cfg, !interactive));
    let got_out = norm.apply(&out);
    let want_out = patch_stdout(slug, &read(&dir.join("stdout.txt")));
    if got_out.trim_end() != want_out.trim_end() {
        failures.push(format!("stdout:\n--- want\n{want_out}\n--- got\n{got_out}"));
    }

    // 5c. exec bits (POSIX only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let want_exec: BTreeSet<String> = read(&dir.join("executables.txt"))
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect();
        let got_exec: BTreeSet<String> = got
            .iter()
            .filter(|p| !p.ends_with('/'))
            .filter(|p| {
                std::fs::metadata(root.join(p)).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
            })
            .cloned()
            .collect();
        if got_exec != want_exec {
            failures.push(format!("executables: got {got_exec:?} want {want_exec:?}"));
        }
    }

    failures
}

fn case_slugs() -> Vec<String> {
    let index: Value =
        serde_json::from_str(&read(&fixtures_root().join("index.json"))).expect("index");
    index["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .map(|v| v.as_str().expect("slug").to_string())
        .collect()
}

#[test]
fn init_matches_npm_goldens() {
    let slugs = case_slugs();
    assert!(slugs.len() >= 27, "expected the full parity matrix");
    let mut report = Vec::new();
    for slug in &slugs {
        let failures = run_case(slug);
        if !failures.is_empty() {
            report.push(format!("## {slug}\n{}", failures.join("\n")));
        }
    }
    assert!(
        report.is_empty(),
        "{} of {} cases differ from npm:\n\n{}",
        report.len(),
        slugs.len(),
        report.join("\n\n")
    );
}

#[test]
fn reported_case_is_in_the_matrix() {
    assert!(case_slugs()
        .iter()
        .any(|s| s == "monorepo-node-nestjs-vue-claude-code"));
}
