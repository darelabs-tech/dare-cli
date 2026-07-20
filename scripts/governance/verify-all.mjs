import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { verifyAdrFrontmatter } from "./verify-adr-frontmatter.mjs";
import { exitCodeForFailure, verifyBaseline } from "./verify-baseline.mjs";
import { verifyNoSecrets } from "./verify-no-secrets.mjs";
import { resolveRepoRoot, verifyStructure } from "./verify-structure.mjs";

const DEFAULT_MANIFEST = "docs/compatibility/baseline-manifest.json";

/**
 * @returns {Promise<number>}
 */
/**
 * @param {string} [repoRoot]
 * @returns {Promise<number>}
 */
export async function runVerifyAll(repoRoot = resolveRepoRoot()) {
  let worstExit = 0;

  const structure = verifyStructure(repoRoot);
  if (!structure.ok) {
    for (const path of structure.missing) {
      process.stderr.write(`missing required path: ${path}\n`);
    }
    worstExit = Math.max(worstExit, 1);
  }

  const secrets = verifyNoSecrets(repoRoot);
  if (!secrets.ok) {
    for (const error of secrets.errors) {
      process.stderr.write(`${error.file}: [${error.rule}] ${error.detail}\n`);
    }
    worstExit = Math.max(worstExit, 1);
  }

  const adr = verifyAdrFrontmatter(join(repoRoot, "docs", "adr"));
  if (!adr.ok) {
    for (const error of adr.errors) {
      process.stderr.write(`${error.file}: [${error.rule}] ${error.detail}\n`);
    }
    worstExit = Math.max(worstExit, 1);
  }

  const manifestPath = join(repoRoot, DEFAULT_MANIFEST);
  if (existsSync(manifestPath)) {
    const baseline = await verifyBaseline({ manifestPath });
    if (!baseline.ok) {
      process.stderr.write(`${JSON.stringify(baseline)}\n`);
      worstExit = Math.max(worstExit, exitCodeForFailure(baseline));
    }
  }

  return worstExit;
}

async function runCli() {
  const exitCode = await runVerifyAll();

  if (exitCode === 0) {
    process.stdout.write(`${JSON.stringify({ ok: true })}\n`);
  }

  process.exit(exitCode);
}

const isMain =
  process.argv[1] &&
  resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));

if (isMain) {
  runCli().catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${JSON.stringify({ ok: false, message })}\n`);
    process.exit(1);
  });
}
