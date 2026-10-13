// Command shim used by the `dare init` parity goldens (cycle rust-init-npm-parity-011).
//
// Invoked by the per-command wrappers in this directory (`npx`, `npm`, `cargo`,
// `rails`, `docker`, `gem`, `composer`, `python`, `go`, plus `.cmd` twins for
// Windows). It never runs the real tool and never touches the network:
//
//   - `<cmd> --version` (the npm `hasCommand` probe) → prints a fake version and
//     exits 0 WITHOUT logging (probes are an npm implementation detail; the Rust
//     port resolves tools via PATH/PATHEXT instead, see DESIGN B-04).
//   - anything else → appends one JSON line to $DARE_SHIM_LOG and exits 0:
//       {"cwd": "<cwd relative to $DARE_SHIM_ROOT, posix, '.' for root>",
//        "argv": ["<cmd>", ...args]}
//     Inside argv, every occurrence of $DARE_SHIM_ROOT is replaced by the token
//     `<case-root>` and a `--user <uid>:<gid>` value by `<uid>:<gid>`, so the
//     log is independent of the machine, temp dir and user.
//
// Usage: node _shim.mjs <cmd-name> [args...]
import fs from 'node:fs';
import path from 'node:path';

const [name, ...args] = process.argv.slice(2);

if (args.length === 1 && args[0] === '--version') {
  process.stdout.write(`${name} 0.0.0-dare-shim\n`);
  process.exit(0);
}

const logFile = process.env.DARE_SHIM_LOG;
const root = process.env.DARE_SHIM_ROOT ? path.resolve(process.env.DARE_SHIM_ROOT) : '';

function toPosix(p) {
  return p.split(path.sep).join('/');
}

function normalizeArg(arg, prev) {
  let out = arg;
  if (root) {
    out = out.split(root).join('<case-root>');
    const posixRoot = toPosix(root);
    if (posixRoot !== root) out = out.split(posixRoot).join('<case-root>');
    out = out.replace(/<case-root>\\/g, '<case-root>/');
  }
  if (prev === '--user' && /^\d+:\d+$/.test(out)) out = '<uid>:<gid>';
  return out;
}

let cwd = process.cwd();
if (root) {
  const rel = path.relative(root, cwd);
  cwd = rel === '' ? '.' : toPosix(rel);
} else {
  cwd = toPosix(cwd);
}

const argv = [name];
args.forEach((a, i) => argv.push(normalizeArg(a, i > 0 ? args[i - 1] : undefined)));

if (logFile) {
  fs.appendFileSync(logFile, JSON.stringify({ cwd, argv }) + '\n');
}
process.exit(0);
