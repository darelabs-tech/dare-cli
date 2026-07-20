import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Fixed list from DARE/BLUEPRINT.md §5.3 + task-001 fixtures-inventory.md */
export const REQUIRED_PATHS = [
  "docs/adr/README.md",
  "docs/adr/ADR-001-compatibilidade-bugs-legados.md",
  "docs/adr/ADR-002-contrato-saida-json.md",
  "docs/adr/ADR-004-rest-compativel-e-mcp-real.md",
  "docs/adr/ADR-006-compatibilidade-migracao-graph-db.md",
  "docs/adr/ADR-007-formato-canonico-capabilities.md",
  "docs/compatibility/README.md",
  "docs/compatibility/baseline-3.18.1.md",
  "docs/compatibility/baseline-manifest.json",
  "docs/compatibility/classification-matrix.md",
  "docs/compatibility/language-policy.md",
  "docs/compatibility/disk-and-json-policy.md",
  "docs/compatibility/breaking-change-process.md",
  "docs/compatibility/fixtures-inventory.md",
  "docs/DECISION-LOG.md",
  "scripts/governance/verify-all.mjs",
];

/**
 * Resolve repository root by walking up from this module until both
 * `docs/` and `scripts/` exist, or fall back to process.cwd().
 *
 * @returns {string}
 */
export function resolveRepoRoot() {
  let current = dirname(fileURLToPath(import.meta.url));

  while (true) {
    const hasDocs = existsSync(join(current, "docs"));
    const hasScripts = existsSync(join(current, "scripts"));

    if (hasDocs && hasScripts) {
      return current;
    }

    const parent = dirname(current);
    if (parent === current) {
      return resolve(process.cwd());
    }

    current = parent;
  }
}

/**
 * Verify that all required governance paths exist under repoRoot.
 *
 * @param {string} [repoRoot]
 * @returns {{ ok: boolean; missing: string[]; checked: number }}
 */
export function verifyStructure(repoRoot = resolveRepoRoot()) {
  const root = resolve(repoRoot);
  const missing = [];

  for (const relativePath of REQUIRED_PATHS) {
    const absolutePath = join(root, relativePath);
    if (!existsSync(absolutePath)) {
      missing.push(relativePath);
    }
  }

  return {
    ok: missing.length === 0,
    missing,
    checked: REQUIRED_PATHS.length,
  };
}

function runCli() {
  const result = verifyStructure();

  if (!result.ok) {
    for (const path of result.missing) {
      process.stderr.write(`missing required path: ${path}\n`);
    }
    process.exit(1);
  }

  process.stdout.write(
    `${JSON.stringify({ ok: true, checked: result.checked })}\n`,
  );
  process.exit(0);
}

const isMain =
  process.argv[1] &&
  resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));

if (isMain) {
  runCli();
}
