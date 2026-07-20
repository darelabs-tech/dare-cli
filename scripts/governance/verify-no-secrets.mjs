import { existsSync, readFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  indexAdrFiles,
  REQUIRED_ADR_IDS,
} from "./verify-adr-frontmatter.mjs";
import { scanForSecrets } from "./verify-baseline.mjs";
import { resolveRepoRoot } from "./verify-structure.mjs";

const DEFAULT_MANIFEST = "docs/compatibility/baseline-manifest.json";

/** @typedef {{ file: string; rule: "NO_SECRETS"; detail: string }} NoSecretsError */

/**
 * @param {string} text
 * @param {string} fileLabel
 * @returns {NoSecretsError[]}
 */
function scanFileForSecrets(text, fileLabel) {
  const scan = scanForSecrets(text);
  if (scan.ok) {
    return [];
  }

  return [
    {
      file: fileLabel,
      rule: "NO_SECRETS",
      detail: `forbidden substring: ${scan.substring}`,
    },
  ];
}

/**
 * @param {string} repoRoot
 * @returns {{ ok: boolean; scanned: string[]; errors: NoSecretsError[] }}
 */
export function verifyNoSecrets(repoRoot) {
  /** @type {NoSecretsError[]} */
  const errors = [];
  /** @type {string[]} */
  const scanned = [];

  const manifestPath = join(repoRoot, DEFAULT_MANIFEST);
  if (existsSync(manifestPath)) {
    scanned.push(DEFAULT_MANIFEST);
    errors.push(
      ...scanFileForSecrets(readFileSync(manifestPath, "utf8"), DEFAULT_MANIFEST),
    );
  }

  const adrDir = join(repoRoot, "docs", "adr");
  const indexed = indexAdrFiles(adrDir);

  for (const requiredId of REQUIRED_ADR_IDS) {
    const filePath = indexed.get(requiredId);
    if (!filePath) {
      continue;
    }

    const fileLabel = `docs/adr/${basename(filePath)}`;
    scanned.push(fileLabel);
    errors.push(
      ...scanFileForSecrets(readFileSync(filePath, "utf8"), fileLabel),
    );
  }

  return {
    ok: errors.length === 0,
    scanned,
    errors,
  };
}

function runCli() {
  const result = verifyNoSecrets(resolveRepoRoot());

  if (!result.ok) {
    for (const error of result.errors) {
      process.stderr.write(`${error.file}: [${error.rule}] ${error.detail}\n`);
    }
    process.exit(1);
  }

  process.stdout.write(
    `${JSON.stringify({ ok: true, scanned: result.scanned })}\n`,
  );
  process.exit(0);
}

const isMain =
  process.argv[1] &&
  resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));

if (isMain) {
  runCli();
}
