# npm `dare init` goldens (cycle rust-init-npm-parity-011)

These fixtures record what the **published** npm CLI `@dewtech/dare-cli@3.18.1`
generates for each case of the parity matrix (BLUEPRINT §6). The Rust parity
suite (`crates/dare-cli/tests/init_npm_parity.rs`, task-010) runs `dare init`
with the same inputs and compares against them.

- Reference: the installed npm package (built `dist/`, LF templates), version
  3.18.1. On the maintainer machine it is at
  `C:\nvm4w\nodejs\node_modules\@dewtech\dare-cli`.
  Source used for reading: `dare-method/packages/cli/src` (`commands/init.ts`,
  `utils/project-generator.ts`, `utils/stack-bootstrap.ts`).
- Runtime: `node:22-bookworm@sha256:0e5f906573693feaa1e21057ebdcfdb5bd5021f050b2dc7c9deceb629c7da2a8`
  (node v22.23.3), Linux, `--network none`. Recorded in `index.json`.
- `.gitattributes` marks this tree `-text`, so bytes are stable on every OS.

## Regenerate

```sh
# POSIX / Git Bash
scripts/parity/gen-init-golden.sh
# Windows PowerShell
pwsh scripts/parity/gen-init-golden.ps1
```

Optional: `--only slug1,slug2` (`-Only` in PowerShell) for a subset, and
`DARE_NPM_PKG=<path to @dewtech/dare-cli>` when the package is not under
`npm root -g`. The driver refuses any version other than 3.18.1 unless you pass
`--allow-version`. A full run takes about 15-30 minutes on Docker Desktop for
Windows. Most of that time is spent loading `node_modules` over the bind mount.
Two full runs produce byte-identical output. This was checked with `diff -r`
when the fixtures were created.

How it works (`scripts/parity/gen-init-golden.mjs`):

1. The script stages the package inside the container. It copies `package.json`,
   `dist/`, `templates/` and `skills/` with modes 0755 for dirs and 0644 for files,
   which matches an npm tarball install. `node_modules` is a symlink to the
   read-only mount.
2. It puts `scripts/parity/shims` first on `PATH`. The shims are `npx`, `npm`,
   `cargo`, `rails`, `docker`, `gem`, `composer`, `python` and `go`.
   - A shim never runs the real tool.
   - `<cmd> --version` exits 0 and is **not logged**. This is the `hasCommand`
     probe in `stack-bootstrap.ts`, and the Rust port replaces it with a
     PATH/PATHEXT lookup (B-04).
   - Any other call appends `{"cwd","argv"}` to `commands.jsonl` and exits 0.
   - The shims do not create files. No tool output is faked.
3. For each case it runs the real `dist/bin/dare.js init <name> [flags]` in a
   fresh process, with cwd set to an empty *case root* and `HOME` set to an empty
   dir. The run fails if init writes anything outside `<case-root>/<name>` or
   into `HOME`. Nothing was written there.
   - `mode: "cli"`: non-interactive flags.
   - `mode: "interactive"`: `inquirer.prompt` is replaced by a stub. The stub
     walks the exact question array from `init.ts`, evaluates `when`, `default`
     and `validate`, records the result to `prompts.json` and returns the
     scripted `answers`. The rest is the unmodified npm path:
     `generateProjectStructure`, spinner, next steps and tips.
     `case.json.generator_config` is the config that `init.ts` lines 252-268
     pass to `generateProjectStructure`.

## Files per case (`<slug>/`)

| File | Content |
|---|---|
| `case.json` | Inputs: `mode`, `command` (argv after the binary), `answers` (interactive), `generator_config`, `toolchain`, `env`, `project_dir`, `covers` (B-xx visible in the case), `stored_files` |
| `tree.txt` | Sorted list (byte order) of posix paths relative to the project dir. Every file is listed, and every **empty** dir is listed with a trailing `/` |
| `hashes.txt` | `<sha256>  <path>` for every file, hashed after CRLF→LF normalization. Files that contain a NUL byte are hashed raw |
| `executables.txt` | Files with an exec bit. Only the Rails scaffold `chmod 0755`s files (`bin/*`) |
| `commands.jsonl` | One `{"cwd": "<dir relative to case root>", "argv": [...]}` per external command, in order |
| `stdout.txt` | Terminal output, normalized (see below) |
| `prompts.json` | Interactive cases only. Each question with `type`, `name`, `message`, `choices`, `default` (evaluated), `asked`, `answer` |
| `files/<path>.golden` | Full contents of files that **code** generates (list below). Template-copied files are covered only by `hashes.txt`. The `.golden` suffix keeps stored `.gitignore`/`Cargo.toml` from acting on this repo |

Stored contents (`shouldStore` in the driver):
- `dare.config.json`, `.cursorrules`, `.antigravityrules`, `AGENTS.md`,
  `.claude/settings.json`, `DARE/README.md`, `dare-graph.yml`
- every `.gitignore`, every `Cargo.toml`, every `.cargo/config.toml`
- `.cursor/rules/skill-*.mdc` when no template has that name or the content
  differs from the template. These are the generated stack skills, such as
  `skill-node-nestjs.mdc`, `skill-mcp-server.mdc` and `skill-none.mdc`.
- every file in a Leptos crate dir, and every file in the `crates/<prefix>-core`
  and `crates/<prefix>-cli` crates

## Normalization rules (the Rust test must apply the same)

- **Case root token.** The absolute case root is replaced by `<case-root>` in
  argv and stdout. Command `cwd` is relative to the case root, for example
  `api-nest-validations/frontend`.
- **`--user <uid>:<gid>`** in docker argv and stdout becomes `<uid>:<gid>`. npm
  only adds this flag on POSIX (`currentUserFlags`, `stack-bootstrap.ts`
  ~1458). On Windows npm emits no `--user` and converts the `-v` path with
  `dockerizePath`.
- **stdout.txt** is stdout and stderr **interleaved**: both fds point to the
  same file. ora writes the spinner to stderr, and `console.log` writes to
  stdout.
  - The process is not a TTY, so there is no banner, no colors, and ora prints
    `- <text>` at start and `✔ <text>` on success.
  - The driver still strips ANSI escapes, keeps only the text after the last
    bare `\r` on each line, and converts CRLF to LF.
- **`dare.config.json.version`** is stored raw (`"3.18.1"`). The Rust test
  patches it to the binary version.
- **Text content** is hashed after CRLF→LF.

## Cases

All cases use `toolchain=native` unless noted. All tools are shimmed, so `auto`
would behave the same.

| Slug | Mode | Inputs |
|---|---|---|
| `backend-ruby-rails-8` | cli | `--stack ruby-rails-8`. The native `rails new . … --api` is shimmed, and the overlay is written without runtime files |
| `backend-rust-axum`, `backend-node-nestjs`, `backend-python-fastapi`, `backend-php-laravel`, `backend-go-gin`, `backend-go-stdlib` | cli | `--stack <id>` |
| `backend-ruby-rails-8-offline` | cli | `--stack ruby-rails-8`, `DARE_RAILS_OFFLINE=1` |
| `mvc-ruby-rails-8-offline` | cli | `--stack ruby-rails-8 --fullstack`, `DARE_RAILS_OFFLINE=1` |
| `mvc-ruby-rails-8-docker` | cli | `--stack ruby-rails-8 --fullstack --toolchain docker` (`ruby:3.3` `sh -c "gem install rails … && rails new …"`) |
| `mvc-php-laravel` | cli | `--stack php-laravel --fullstack` |
| `mcp-{node-ts,python,rust,go}-stdio`, `mcp-node-ts-sse`, `mcp-node-ts-http` | cli | `--mcp <lang> --transport <t>` |
| `monorepo-node-nestjs-vue-claude-code` | interactive | **User-reported case** (`api-nest-validations`) |
| `monorepo-python-fastapi-react-codex` | interactive | monorepo, codex |
| `frontend-vue-antigravity-docker` | interactive | frontend vue, antigravity, **toolchain docker** (`node:20-alpine`) |
| `frontend-react-hybrid` | interactive | frontend react, hybrid |
| `monorepo-rust-axum-leptos-single-claude-hybrid` | interactive | rust-axum + rust-leptos, layout single, claude-hybrid |
| `monorepo-rust-axum-leptos-csr-multi-ars-cursor` | interactive | rust-axum + rust-leptos-csr, multi, prefix `ars`, name `arsenal-web` |
| `backend-node-nestjs-graphrag-json` / `-neo4j` | interactive | backend nestjs, cursor, graphrag json/neo4j, mcp=false |
| `monorepo-go-gin-none-cursor` | interactive | monorepo go-gin + frontend `none` (extra, for B-05) |
| `mcp-rust-http-claude-code` | interactive | MCP rust, `http-stream`, tools+resources, claude-code (extra, for B-02/B-06) |

Shim side effects the Rust test must reproduce:
- External scaffolders create nothing, so some directories stay empty and are
  listed in `tree.txt` with a trailing `/`:
  - Vite (`degit` + `npm install`) leaves `frontend/` (monorepo) or nothing (frontend
    only) behind. `tryRenameNpmProject` skips the missing `package.json`
    (`stack-bootstrap.ts` ~1503).
  - Rails native and docker plans leave `bin/`, `public/`, `tmp/`,
    `config/environments/` and similar dirs.
- `cargo fetch` (Leptos) produces no `Cargo.lock`. npm then deletes the member
  lock only if it exists.

## Expected deviations (B-xx) the Rust test applies as explicit patches

| # | What the golden shows (npm bug) | Rust expectation | Cases |
|---|---|---|---|
| B-01 | `claude-hybrid` installs only Claude files: there is no `.cursorrules`, `.cursor/**` or `skill-*.mdc` (`project-generator.ts` cursor branch checks only `cursor`/`hybrid`) | Also install the Cursor set: `.cursorrules`, `.cursor/commands/*`, `.cursor/rules/*` and the generated `skill-rust-axum.mdc`/`skill-rust-leptos.mdc` | `monorepo-rust-axum-leptos-single-claude-hybrid` |
| B-02 | `.claude/settings.json` for MCP always uses `python -m py_compile main.py` / `pytest` / `ruff check .` (`templates.ts` `generateClaudeSettings` ~589-630, `structure === 'mcp-server'` → python) | Use the commands of the chosen MCP language (rust: cargo …) | `mcp-rust-http-claude-code` |
| B-03 | Mojibake: `âœ…` in `.claude/settings.json` hook echo, and `â†’`/`â€”` in `.cursorrules` (Ralph Loop line, MCP transport line) | Correct UTF-8 (`✅`, `→`, `—`), so the content hash changes | every case with `.claude/settings.json` or `.cursorrules` |
| B-04 | npm probes `<cmd> --version` (not logged here) and fails to find `.cmd` shims on Windows | PATH+PATHEXT resolution. Argv is unchanged | all cases with commands (Windows only) |
| B-05 | Frontend `none` writes `.cursor/rules/skill-none.mdc` | File absent | `monorepo-go-gin-none-cursor` |
| B-06 | MCP rust/go next steps print `npm install` / `npm run inspect` / `npm run dev` | Language-specific commands (cargo / go) | `mcp-rust-http-claude-code` (stdout) |
| B-07 | Multi-crate next steps print the name slug (`crates/arsenal-web-core …`) instead of the prefix | `crates/ars-core \| crates/ars-server \| crates/ars-web \| crates/ars-cli` | `monorepo-rust-axum-leptos-csr-multi-ars-cursor` (stdout) |
| B-08 | Leptos crate in a monorepo leaves an empty `.cargo/` (`crates/web/.cargo/`, `crates/ars-web/.cargo/`) | Dir not created (remove the `…/.cargo/` line from `tree.txt`) | both Rust workspace cases |
| B-09 | (rollback behaviour, not visible in a successful init) | n/a | none |
| B-10 | `Short crate prefix` default is `app` when the name is positional (`prompts.json` `default`) | Initials of the project name (`ars`-style) | `monorepo-rust-axum-leptos-csr-multi-ars-cursor` (prompt default) |

Other accepted deviations (DESIGN):
- `dare.config.json.version` is the Rust binary version.
- No `Dockerfile` or `docker-compose.yml` is written. None of these goldens
  contain them, because npm writes them only in the `skipBootstrap` legacy path.
