#!/usr/bin/env node
// Golden generator for `dare init` parity (DARE cycle rust-init-npm-parity-011, task-001).
//
// Runs the PUBLISHED npm `@dewtech/dare-cli` (3.18.1) `dare init` for every case of
// the parity matrix (BLUEPRINT §6) and records what it generated, so the Rust
// port can be compared against it (crates/dare-cli/tests/init_npm_parity.rs).
//
// Designed to run inside a Linux container (see gen-init-golden.sh / .ps1) so the
// behaviour matches a POSIX install. No network is needed: every external tool
// (npx, npm, cargo, rails, docker, gem, composer, python, go) is replaced by a
// shim from ./shims that records its argv to commands.jsonl and exits 0.
//
// Usage (normally via the wrappers):
//   node gen-init-golden.mjs --pkg <dir of @dewtech/dare-cli> --out <golden dir>
//                            [--work <scratch dir>] [--only slug1,slug2]
//                            [--allow-version]
//   node gen-init-golden.mjs --child <spec.json>   (internal: runs one case)
//
// Every case runs the real `dist/bin/dare.js init ...` in a fresh process:
//   - mode "cli":         non-interactive flags (`--stack`, `--mcp`, ...).
//   - mode "interactive": `dare init <name>` with `inquirer.prompt` replaced by a
//     stub that walks the exact question list from init.ts (evaluating `when`,
//     `default` and `validate`), records it to prompts.json and returns the
//     scripted answers. Everything after the prompt (generateProjectStructure,
//     spinner, next steps) is the unmodified npm code path.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SCRIPT = fileURLToPath(import.meta.url);
const SCRIPT_DIR = path.dirname(SCRIPT);
const EXPECTED_VERSION = '3.18.1';

// ─── Parity matrix ──────────────────────────────────────────────────────────
// `covers` lists the DESIGN deviations (B-xx) observable in the case.
const backendCli = (stack) => ({
  slug: `backend-${stack}`,
  mode: 'cli',
  name: 'demo-api',
  args: ['--stack', stack, '--toolchain', 'native'],
  description: `Backend only, ${stack}, non-interactive (ide=cursor, graphrag=sqlite, mcp=false).`,
  covers: [],
});

const mcpCli = (lang, transport) => ({
  slug: `mcp-${lang}-${transport}`,
  mode: 'cli',
  name: 'demo-mcp',
  args: ['--mcp', lang, '--transport', transport, '--toolchain', 'native'],
  description: `MCP server ${lang} over ${transport}, non-interactive.`,
  covers: [],
});

const CASES = [
  // 1. Backend only — the 7 backend stacks, non-interactive.
  ...['ruby-rails-8', 'rust-axum', 'node-nestjs', 'python-fastapi', 'php-laravel', 'go-gin', 'go-stdlib'].map(
    backendCli,
  ),
  {
    slug: 'backend-ruby-rails-8-offline',
    mode: 'cli',
    name: 'demo-api',
    args: ['--stack', 'ruby-rails-8', '--toolchain', 'native'],
    env: { DARE_RAILS_OFFLINE: '1' },
    description: 'Rails API-only with DARE_RAILS_OFFLINE=1 (offline template runtime, no rails new).',
    covers: [],
  },
  // 2. MVC (--fullstack).
  {
    slug: 'mvc-ruby-rails-8-offline',
    mode: 'cli',
    name: 'demo-mvc',
    args: ['--stack', 'ruby-rails-8', '--fullstack', '--toolchain', 'native'],
    env: { DARE_RAILS_OFFLINE: '1' },
    description: 'Rails full-stack MVC, offline templates (DARE_RAILS_OFFLINE=1).',
    covers: [],
  },
  {
    slug: 'mvc-ruby-rails-8-docker',
    mode: 'cli',
    name: 'demo-mvc',
    args: ['--stack', 'ruby-rails-8', '--fullstack', '--toolchain', 'docker'],
    description: 'Rails full-stack MVC via the ruby:3.3 Docker plan (records docker argv; overlay on top).',
    covers: [],
  },
  {
    slug: 'mvc-php-laravel',
    mode: 'cli',
    name: 'demo-mvc',
    args: ['--stack', 'php-laravel', '--fullstack', '--toolchain', 'native'],
    description: 'Laravel full-stack MVC, non-interactive.',
    covers: [],
  },
  // 3. MCP servers.
  mcpCli('node-ts', 'stdio'),
  mcpCli('python', 'stdio'),
  mcpCli('rust', 'stdio'),
  mcpCli('go', 'stdio'),
  mcpCli('node-ts', 'sse'),
  mcpCli('node-ts', 'http'),
  // 4. The case reported by the user.
  {
    slug: 'monorepo-node-nestjs-vue-claude-code',
    mode: 'interactive',
    name: 'api-nest-validations',
    answers: {
      structure: 'monorepo',
      backend: 'node-nestjs',
      frontend: 'vue',
      ide: 'claude-code',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'User-reported case: monorepo NestJS backend/ + Vue frontend/, Claude Code.',
    covers: ['B-03'],
  },
  // 5.
  {
    slug: 'monorepo-python-fastapi-react-codex',
    mode: 'interactive',
    name: 'shop-platform',
    answers: {
      structure: 'monorepo',
      backend: 'python-fastapi',
      frontend: 'react',
      ide: 'codex',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Monorepo FastAPI + React, Codex CLI.',
    covers: [],
  },
  // 6. Frontend only.
  {
    slug: 'frontend-vue-antigravity-docker',
    mode: 'interactive',
    name: 'web-portal',
    answers: {
      structure: 'frontend',
      frontend: 'vue',
      ide: 'antigravity',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'docker',
    },
    description: 'Frontend only, Vue at the root, Antigravity, toolchain=docker (records node:20-alpine argv).',
    covers: [],
  },
  {
    slug: 'frontend-react-hybrid',
    mode: 'interactive',
    name: 'web-portal',
    answers: {
      structure: 'frontend',
      frontend: 'react',
      ide: 'hybrid',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Frontend only, React at the root, Cursor + Antigravity hybrid.',
    covers: ['B-03'],
  },
  // 7. Rust fullstack workspaces.
  {
    slug: 'monorepo-rust-axum-leptos-single-claude-hybrid',
    mode: 'interactive',
    name: 'leptos-app',
    answers: {
      structure: 'monorepo',
      backend: 'rust-axum',
      frontend: 'rust-leptos',
      rustWorkspaceLayout: 'single',
      ide: 'claude-hybrid',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Rust workspace crates/server + crates/web (Leptos SSR), Claude Code + Cursor hybrid.',
    covers: ['B-01', 'B-03', 'B-08'],
  },
  {
    slug: 'monorepo-rust-axum-leptos-csr-multi-ars-cursor',
    mode: 'interactive',
    name: 'arsenal-web',
    answers: {
      structure: 'monorepo',
      backend: 'rust-axum',
      frontend: 'rust-leptos-csr',
      rustWorkspaceLayout: 'multi',
      cratePrefix: 'ars',
      ide: 'cursor',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Rust multi-crate workspace ars-{core,server,web,cli} (Leptos CSR), Cursor.',
    covers: ['B-03', 'B-07', 'B-08'],
  },
  // 8. GraphRAG backends (same shape as backend-node-nestjs, chosen interactively).
  {
    slug: 'backend-node-nestjs-graphrag-json',
    mode: 'interactive',
    name: 'demo-api',
    answers: {
      structure: 'backend',
      backend: 'node-nestjs',
      ide: 'cursor',
      graphrag: 'json',
      mcp: false,
      toolchain: 'native',
    },
    description: 'Backend NestJS with GraphRAG json (otherwise like backend-node-nestjs).',
    covers: [],
  },
  {
    slug: 'backend-node-nestjs-graphrag-neo4j',
    mode: 'interactive',
    name: 'demo-api',
    answers: {
      structure: 'backend',
      backend: 'node-nestjs',
      ide: 'cursor',
      graphrag: 'neo4j',
      mcp: false,
      toolchain: 'native',
    },
    description: 'Backend NestJS with GraphRAG neo4j (otherwise like backend-node-nestjs).',
    covers: [],
  },
  // Extra coverage for listed deviations.
  {
    slug: 'monorepo-go-gin-none-cursor',
    mode: 'interactive',
    name: 'demo-api',
    answers: {
      structure: 'monorepo',
      backend: 'go-gin',
      frontend: 'none',
      ide: 'cursor',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Monorepo with frontend "none" (npm writes .cursor/rules/skill-none.mdc).',
    covers: ['B-05'],
  },
  {
    slug: 'mcp-rust-http-claude-code',
    mode: 'interactive',
    name: 'tools-server',
    answers: {
      structure: 'mcp-server',
      mcpLanguage: 'rust',
      mcpTransport: 'http-stream',
      mcpFeatures: ['tools', 'resources'],
      ide: 'claude-code',
      graphrag: 'sqlite',
      mcp: true,
      toolchain: 'native',
    },
    description: 'Interactive MCP (Rust, HTTP stream, tools+resources), Claude Code.',
    covers: ['B-02', 'B-03', 'B-06'],
  },
];

// ─── Helpers ────────────────────────────────────────────────────────────────

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (!a.startsWith('--')) throw new Error(`unexpected argument: ${a}`);
    const key = a.slice(2);
    if (key === 'allow-version') {
      out[key] = true;
    } else {
      out[key] = argv[++i];
      if (out[key] === undefined) throw new Error(`missing value for ${a}`);
    }
  }
  return out;
}

const toPosix = (p) => p.split(path.sep).join('/');

function rmrf(p) {
  fs.rmSync(p, { recursive: true, force: true });
}

/** Mirrors init.ts: config passed to generateProjectStructure (outputDir tokenized). */
function generatorConfig(c) {
  const outputDir = `<case-root>/${c.name}`;
  if (c.mode === 'interactive') {
    const a = c.answers;
    // init.ts lines 252-268 (keys with `undefined` are dropped by JSON).
    return {
      name: c.name,
      structure: a.structure,
      backend: a.backend,
      frontend: a.frontend,
      mcpTransport: a.mcpTransport,
      mcpLanguage: a.mcpLanguage,
      mcpFeatures: a.mcpFeatures,
      ide: a.ide,
      graphrag: a.graphrag,
      mcp: a.mcp,
      toolchain: a.toolchain,
      rustWorkspaceLayout: a.rustWorkspaceLayout,
      cratePrefix: a.cratePrefix,
      outputDir,
    };
  }
  const opt = (flag) => {
    const i = c.args.indexOf(flag);
    return i >= 0 ? c.args[i + 1] : undefined;
  };
  const toolchain = opt('--toolchain') ?? 'auto';
  const mcpLang = opt('--mcp');
  if (mcpLang) {
    const t = opt('--transport') ?? 'stdio';
    // init.ts runNonInteractive: 'http' → 'http-stream'; no mcpFeatures.
    return {
      name: c.name,
      structure: 'mcp-server',
      mcpLanguage: mcpLang,
      mcpTransport: t === 'http' ? 'http-stream' : t,
      ide: 'cursor',
      graphrag: 'sqlite',
      mcp: false,
      toolchain,
      outputDir,
    };
  }
  return {
    name: c.name,
    structure: c.args.includes('--fullstack') ? 'mvc' : 'backend',
    backend: opt('--stack'),
    ide: 'cursor',
    graphrag: 'sqlite',
    mcp: false,
    toolchain,
    outputDir,
  };
}

/** Directory (relative to the project root) of the Leptos crate, if any. */
function leptosDir(cfg) {
  if (cfg.frontend !== 'rust-leptos' && cfg.frontend !== 'rust-leptos-csr') return null;
  if (cfg.structure === 'frontend') return '';
  if (cfg.structure !== 'monorepo') return null;
  if (cfg.backend === 'rust-axum') {
    if ((cfg.rustWorkspaceLayout ?? 'single') === 'multi') {
      const prefix =
        cfg.cratePrefix?.trim() ||
        cfg.name.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '') ||
        'app';
      return `crates/${prefix}-web`;
    }
    return 'crates/web';
  }
  return 'frontend';
}

const ALWAYS_STORE = new Set([
  'dare.config.json',
  '.cursorrules',
  '.antigravityrules',
  'AGENTS.md',
  '.claude/settings.json',
  'DARE/README.md',
  'dare-graph.yml',
]);

/**
 * Files whose CONTENT is stored (as `files/<path>.golden`): only files produced
 * by code in the npm generator (not copied verbatim from a static template).
 * Everything else is covered by hashes.txt only.
 */
function shouldStore(rel, cfg, pkg) {
  const base = path.posix.basename(rel);
  if (ALWAYS_STORE.has(rel)) return true;
  if (base === '.gitignore' || base === 'Cargo.toml') return true;
  if (rel.endsWith('.cargo/config.toml')) return true;
  if (/^\.cursor\/rules\/skill-[^/]+\.mdc$/.test(rel)) {
    const tpl = path.join(pkg, 'templates', 'ide', 'cursor', '.cursor', 'rules', base);
    if (!fs.existsSync(tpl)) return true;
    return !fs.readFileSync(tpl).equals(fs.readFileSync(path.join(cfg.__projectDir, rel)));
  }
  const ld = leptosDir(cfg);
  if (ld !== null) {
    if (ld === '' || rel.startsWith(`${ld}/`)) return true;
  }
  if (/^crates\/[^/]+-(core|cli)\//.test(rel)) return true;
  if (/(^|\/)package\.json$/.test(rel) && (cfg.frontend === 'react' || cfg.frontend === 'vue')) {
    const fd = cfg.structure === 'monorepo' ? 'frontend/' : '';
    if (rel === `${fd}package.json`) return true;
  }
  return false;
}

/** Walk dir → { files: [rel], emptyDirs: [rel] } with posix relative paths. */
function walk(root) {
  const files = [];
  const emptyDirs = [];
  const rec = (abs, rel) => {
    const entries = fs.readdirSync(abs, { withFileTypes: true });
    if (entries.length === 0 && rel) emptyDirs.push(rel);
    for (const e of entries) {
      const childAbs = path.join(abs, e.name);
      const childRel = rel ? `${rel}/${e.name}` : e.name;
      if (e.isDirectory()) rec(childAbs, childRel);
      else if (e.isFile()) files.push(childRel);
      else throw new Error(`unexpected non-regular entry: ${childAbs}`);
    }
  };
  rec(root, '');
  return { files, emptyDirs };
}

const byteSort = (a, b) => (Buffer.compare(Buffer.from(a), Buffer.from(b)));

function normalizedContent(buf) {
  // Text files: CRLF → LF. Files containing NUL are treated as binary (raw).
  if (buf.includes(0)) return buf;
  return Buffer.from(buf.toString('latin1').replace(/\r\n/g, '\n'), 'latin1');
}

function normalizeOutput(text, caseRoot) {
  let t = text.replace(/\r\n/g, '\n');
  // Spinner redraws: keep only what follows the last bare CR on each line.
  t = t
    .split('\n')
    .map((line) => (line.includes('\r') ? line.slice(line.lastIndexOf('\r') + 1) : line))
    .join('\n');
  // ANSI escape sequences (colors, cursor movement).
  // eslint-disable-next-line no-control-regex
  t = t.replace(/\x1b\[[0-9;?]*[ -/]*[@-~]/g, '');
  t = t.split(caseRoot).join('<case-root>');
  t = t.replace(/--user \d+:\d+/g, '--user <uid>:<gid>');
  return t;
}

// ─── Child: run one case in-process (cwd = case root) ───────────────────────

async function runChild(specFile) {
  const spec = JSON.parse(fs.readFileSync(specFile, 'utf8'));
  const inquirerUrl = pathToFileURL(path.join(spec.pkg, 'node_modules', 'inquirer', 'lib', 'index.js'));
  const inquirer = (await import(inquirerUrl.href)).default;

  if (spec.mode === 'interactive') {
    const provided = spec.answers;
    inquirer.prompt = async (questions) => {
      const answers = {};
      const record = [];
      const askedNames = new Set();
      for (const q of questions) {
        let asked;
        if (q.when === undefined) asked = true;
        else if (typeof q.when === 'function') asked = Boolean(await q.when({ ...answers }));
        else asked = Boolean(q.when);
        const entry = { type: q.type, name: q.name, message: q.message, asked };
        if (Array.isArray(q.choices)) {
          entry.choices = q.choices.map((ch) =>
            typeof ch === 'string'
              ? ch
              : { name: ch.name, value: ch.value, ...(ch.checked !== undefined ? { checked: ch.checked } : {}) },
          );
        }
        if (q.default !== undefined) {
          entry.default =
            typeof q.default === 'function' ? (asked ? q.default({ ...answers }) : '<function>') : q.default;
        }
        if (asked) {
          if (!(q.name in provided)) throw new Error(`no scripted answer for asked question '${q.name}'`);
          const value = provided[q.name];
          if (typeof q.validate === 'function') {
            const r = await q.validate(value, { ...answers });
            if (r !== true) throw new Error(`scripted answer for '${q.name}' failed validation: ${r}`);
          }
          answers[q.name] = value;
          askedNames.add(q.name);
          entry.answer = value;
        }
        record.push(entry);
      }
      for (const k of Object.keys(provided)) {
        if (!askedNames.has(k)) throw new Error(`scripted answer '${k}' was never asked`);
      }
      fs.writeFileSync(spec.promptsOut, JSON.stringify(record, null, 2) + '\n');
      return answers;
    };
  } else {
    inquirer.prompt = async () => {
      throw new Error('non-interactive case unexpectedly prompted');
    };
  }

  const bin = path.join(spec.pkg, 'dist', 'bin', 'dare.js');
  process.argv = [process.execPath, bin, 'init', spec.name, ...(spec.args ?? [])];
  await import(pathToFileURL(bin).href);
}

// ─── Parent ─────────────────────────────────────────────────────────────────

function prepareShims(work) {
  const dst = path.join(work, 'shims');
  fs.mkdirSync(dst, { recursive: true });
  for (const f of fs.readdirSync(path.join(SCRIPT_DIR, 'shims'))) {
    if (f.endsWith('.cmd')) continue; // Windows twins are not used on Linux.
    fs.copyFileSync(path.join(SCRIPT_DIR, 'shims', f), path.join(dst, f));
    if (!f.endsWith('.mjs')) fs.chmodSync(path.join(dst, f), 0o755);
  }
  return dst;
}

/**
 * Stages the package on the container's own filesystem: `package.json`, `dist/`,
 * `templates/` and `skills/` are copied with normalized modes (dirs 0755, files
 * 0644 — what an npm tarball install yields; Windows bind mounts report every
 * file as 0777, which fs.copy would propagate into the generated project), and
 * `node_modules` is symlinked to the original (read-only) location. Also makes
 * each case ~10x faster than importing dist/ over a Windows bind mount.
 */
function stagePackage(src, work) {
  const dst = path.join(work, 'pkg', 'node_modules', '@dewtech', 'dare-cli');
  fs.mkdirSync(dst, { recursive: true });
  const copyTree = (from, to) => {
    const st = fs.statSync(from);
    if (st.isDirectory()) {
      fs.mkdirSync(to, { recursive: true });
      fs.chmodSync(to, 0o755);
      for (const e of fs.readdirSync(from)) copyTree(path.join(from, e), path.join(to, e));
    } else {
      fs.copyFileSync(from, to);
      fs.chmodSync(to, 0o644);
    }
  };
  for (const entry of ['package.json', 'dist', 'templates', 'skills']) {
    if (fs.existsSync(path.join(src, entry))) copyTree(path.join(src, entry), path.join(dst, entry));
  }
  fs.symlinkSync(path.join(src, 'node_modules'), path.join(dst, 'node_modules'), 'dir');
  return dst;
}

function runCase(c, { pkg, out, work, shims }) {
  const caseRoot = path.join(work, 'cases', c.slug);
  const home = path.join(work, 'home', c.slug);
  rmrf(caseRoot);
  rmrf(home);
  fs.mkdirSync(caseRoot, { recursive: true });
  fs.mkdirSync(home, { recursive: true });
  const meta = path.join(work, 'meta', c.slug);
  rmrf(meta);
  fs.mkdirSync(meta, { recursive: true });

  const shimLog = path.join(meta, 'commands.jsonl');
  fs.writeFileSync(shimLog, '');
  const promptsOut = path.join(meta, 'prompts.json');
  const specFile = path.join(meta, 'spec.json');
  fs.writeFileSync(
    specFile,
    JSON.stringify({ pkg, mode: c.mode, name: c.name, args: c.args, answers: c.answers, promptsOut }),
  );

  const env = {
    PATH: [shims, path.dirname(process.execPath), '/usr/local/bin', '/usr/bin', '/bin'].join(':'),
    HOME: home,
    LANG: 'C.UTF-8',
    TERM: 'dumb',
    DARE_SHIM_LOG: shimLog,
    DARE_SHIM_ROOT: caseRoot,
    ...(c.env ?? {}),
  };
  // stdout and stderr share one file descriptor so the interleaving the user
  // sees in a terminal is preserved (ora writes to stderr, console.log to stdout).
  const outFile = path.join(meta, 'output.txt');
  const fd = fs.openSync(outFile, 'w');
  const res = spawnSync(process.execPath, [SCRIPT, '--child', specFile], {
    cwd: caseRoot,
    env,
    stdio: ['ignore', fd, fd],
  });
  fs.closeSync(fd);
  const output = fs.readFileSync(outFile, 'utf8');
  if (res.status !== 0) {
    throw new Error(`case ${c.slug} failed (exit ${res.status}):\n${output}`);
  }

  const projectDir = path.join(caseRoot, c.name);
  const cfg = { ...generatorConfig(c), __projectDir: projectDir };

  // Anything outside the project dir (besides nothing) is unexpected.
  const stray = fs.readdirSync(caseRoot).filter((e) => e !== c.name);
  if (stray.length) throw new Error(`case ${c.slug}: unexpected entries in case root: ${stray.join(', ')}`);
  const homeEntries = fs.readdirSync(home);
  if (homeEntries.length) throw new Error(`case ${c.slug}: init wrote into $HOME: ${homeEntries.join(', ')}`);

  const { files, emptyDirs } = walk(projectDir);
  files.sort(byteSort);
  const tree = [...files, ...emptyDirs.map((d) => `${d}/`)].sort(byteSort);

  const dst = path.join(out, c.slug);
  rmrf(dst);
  fs.mkdirSync(dst, { recursive: true });

  const hashes = [];
  const executables = [];
  const stored = [];
  for (const rel of files) {
    const abs = path.join(projectDir, rel);
    const buf = fs.readFileSync(abs);
    const sha = crypto.createHash('sha256').update(normalizedContent(buf)).digest('hex');
    hashes.push(`${sha}  ${rel}`);
    if (fs.statSync(abs).mode & 0o111) executables.push(rel);
    if (shouldStore(rel, cfg, pkg)) {
      const target = path.join(dst, 'files', ...`${rel}.golden`.split('/'));
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, normalizedContent(buf));
      stored.push(rel);
    }
  }

  const { __projectDir, ...cfgOut } = cfg;
  const caseJson = {
    slug: c.slug,
    description: c.description,
    mode: c.mode,
    command: ['dare', 'init', c.name, ...(c.args ?? [])],
    ...(c.mode === 'interactive' ? { answers: c.answers } : {}),
    generator_config: JSON.parse(JSON.stringify(cfgOut)),
    toolchain: cfgOut.toolchain,
    env: c.env ?? {},
    cwd: '<case-root>',
    project_dir: c.name,
    covers: c.covers ?? [],
    exit_code: res.status,
    stored_files: stored,
  };
  fs.writeFileSync(path.join(dst, 'case.json'), JSON.stringify(caseJson, null, 2) + '\n');
  fs.writeFileSync(path.join(dst, 'tree.txt'), tree.join('\n') + '\n');
  fs.writeFileSync(path.join(dst, 'hashes.txt'), hashes.join('\n') + '\n');
  fs.writeFileSync(path.join(dst, 'executables.txt'), executables.length ? executables.join('\n') + '\n' : '');
  fs.writeFileSync(path.join(dst, 'commands.jsonl'), fs.readFileSync(shimLog));
  fs.writeFileSync(path.join(dst, 'stdout.txt'), normalizeOutput(output, caseRoot));
  if (c.mode === 'interactive') {
    fs.copyFileSync(promptsOut, path.join(dst, 'prompts.json'));
  }
  return { slug: c.slug, files: files.length, stored: stored.length, commands: fs.readFileSync(shimLog, 'utf8').split('\n').filter(Boolean).length };
}

async function main() {
  const argv = process.argv.slice(2);
  if (argv[0] === '--child') {
    await runChild(argv[1]);
    return;
  }
  const opts = parseArgs(argv);
  if (!opts.pkg || !opts.out) {
    console.error('usage: gen-init-golden.mjs --pkg <dare-cli package dir> --out <golden dir> [--work <dir>] [--only a,b]');
    process.exit(2);
  }
  const srcPkg = path.resolve(opts.pkg);
  const out = path.resolve(opts.out);
  const work = path.resolve(opts.work ?? path.join(process.env.TMPDIR ?? '/tmp', 'dare-init-golden'));
  const pkgJson = JSON.parse(fs.readFileSync(path.join(srcPkg, 'package.json'), 'utf8'));
  if (pkgJson.version !== EXPECTED_VERSION && !opts['allow-version']) {
    console.error(`expected @dewtech/dare-cli ${EXPECTED_VERSION}, found ${pkgJson.version} (pass --allow-version)`);
    process.exit(2);
  }

  const only = opts.only ? new Set(opts.only.split(',')) : null;
  const cases = only ? CASES.filter((c) => only.has(c.slug)) : CASES;
  if (only && cases.length !== only.size) throw new Error('unknown slug in --only');

  rmrf(work);
  fs.mkdirSync(work, { recursive: true });
  const shims = prepareShims(work);
  const pkg = stagePackage(srcPkg, work);
  fs.mkdirSync(out, { recursive: true });

  if (!only) {
    // Drop stale case dirs (keeps README.md and other top-level files).
    const known = new Set(CASES.map((c) => c.slug));
    for (const e of fs.readdirSync(out, { withFileTypes: true })) {
      if (e.isDirectory() && !known.has(e.name)) rmrf(path.join(out, e.name));
    }
  }

  const summary = [];
  for (const c of cases) {
    const r = runCase(c, { pkg, out, work, shims });
    summary.push(r);
    console.log(`${r.slug}: ${r.files} files, ${r.stored} stored, ${r.commands} commands`);
  }

  if (!only) {
    const index = {
      source: `${pkgJson.name}@${pkgJson.version}`,
      node: process.version,
      image: process.env.DARE_PARITY_IMAGE ?? null,
      cases: CASES.map((c) => c.slug),
    };
    fs.writeFileSync(path.join(out, 'index.json'), JSON.stringify(index, null, 2) + '\n');
  }
  rmrf(work);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.stack ?? err.message : err);
  process.exit(1);
});
