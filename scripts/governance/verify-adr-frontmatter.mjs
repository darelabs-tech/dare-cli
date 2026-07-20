import { existsSync, readdirSync, readFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { scanForSecrets } from "./verify-baseline.mjs";
import { resolveRepoRoot } from "./verify-structure.mjs";

/** @typedef {{ file: string; rule: string; detail: string }} AdrError */

export const REQUIRED_ADR_IDS = [
  "ADR-001",
  "ADR-002",
  "ADR-004",
  "ADR-006",
  "ADR-007",
];

export const IGNORED_ADR_IDS = ["ADR-003"];

export const REQUIRED_SECTIONS = [
  "## Contexto",
  "## Decisão",
  "## Consequências",
  "## Critérios de aceite",
  "## Referências",
];

const ADR_FILENAME_REGEX = /^ADR-\d{3}-.+\.md$/;

/**
 * @param {string} filename
 * @returns {string | null}
 */
export function idFromFilename(filename) {
  const match = filename.match(/^(ADR-\d{3})-/);
  return match ? match[1] : null;
}

/**
 * @param {string} content
 * @returns {{ frontmatter: string; body: string } | null}
 */
export function splitFrontmatter(content) {
  if (!content.startsWith("---")) {
    return null;
  }

  const closing = content.indexOf("\n---", 3);
  if (closing === -1) {
    return null;
  }

  const frontmatter = content.slice(4, closing);
  const body = content.slice(closing + 4).replace(/^\r?\n/, "");
  return { frontmatter, body };
}

/**
 * @param {string} frontmatter
 * @param {string} field
 * @returns {string | null}
 */
export function readFrontmatterField(frontmatter, field) {
  const match = frontmatter.match(new RegExp(`^${field}:\\s*(.+)$`, "m"));
  if (!match) {
    return null;
  }

  let value = match[1].trim();
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    value = value.slice(1, -1);
  }

  return value;
}

/**
 * @param {string} body
 * @returns {{ ok: true } | { ok: false; detail: string }}
 */
export function validateSectionsOrder(body) {
  let searchFrom = 0;

  for (const heading of REQUIRED_SECTIONS) {
    const index = body.indexOf(heading, searchFrom);
    if (index === -1) {
      return {
        ok: false,
        detail: `missing or out-of-order heading: ${heading}`,
      };
    }
    searchFrom = index + heading.length;
  }

  return { ok: true };
}

/**
 * @param {string} adrDir
 * @returns {Map<string, string>}
 */
export function indexAdrFiles(adrDir) {
  /** @type {Map<string, string>} */
  const byId = new Map();

  if (!existsSync(adrDir)) {
    return byId;
  }

  for (const name of readdirSync(adrDir)) {
    if (!ADR_FILENAME_REGEX.test(name)) {
      continue;
    }

    const id = idFromFilename(name);
    if (!id || IGNORED_ADR_IDS.includes(id)) {
      continue;
    }

    byId.set(id, join(adrDir, name));
  }

  return byId;
}

/**
 * @param {string} filePath
 * @returns {AdrError[]}
 */
export function verifyAdrFile(filePath) {
  /** @type {AdrError[]} */
  const errors = [];
  const file = basename(filePath);

  let content;
  try {
    content = readFileSync(filePath, "utf8");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    errors.push({
      file,
      rule: "ADR_FILE_REQUIRED",
      detail: `cannot read file: ${message}`,
    });
    return errors;
  }

  const secretScan = scanForSecrets(content);
  if (!secretScan.ok) {
    errors.push({
      file,
      rule: "NO_SECRETS",
      detail: `forbidden substring: ${secretScan.substring}`,
    });
  }

  const split = splitFrontmatter(content);
  if (!split) {
    errors.push({
      file,
      rule: "FRONTMATTER_PRESENT",
      detail: "YAML frontmatter block (--- ... ---) missing at top",
    });
    return errors;
  }

  const id = readFrontmatterField(split.frontmatter, "id");
  const status = readFrontmatterField(split.frontmatter, "status");
  const expectedId = idFromFilename(file);

  if (!id || !expectedId || id !== expectedId) {
    errors.push({
      file,
      rule: "ID_MATCH_FILENAME",
      detail: `expected id ${expectedId ?? "unknown"}, got ${id ?? "missing"}`,
    });
  }

  if (status !== "Accepted") {
    errors.push({
      file,
      rule: "STATUS_ACCEPTED",
      detail: `expected status Accepted, got ${status ?? "missing"}`,
    });
  }

  const sections = validateSectionsOrder(split.body);
  if (!sections.ok) {
    errors.push({
      file,
      rule: "SECTIONS_ORDER",
      detail: sections.detail,
    });
  }

  return errors;
}

/**
 * @param {string} [adrGlob]
 * @returns {{ ok: boolean; checked: string[]; errors: AdrError[] }}
 */
export function verifyAdrFrontmatter(adrGlob) {
  const repoRoot = resolveRepoRoot();
  const adrDir = adrGlob ? resolve(adrGlob) : join(repoRoot, "docs", "adr");
  /** @type {AdrError[]} */
  const errors = [];
  /** @type {string[]} */
  const checked = [];

  const indexed = indexAdrFiles(adrDir);

  for (const requiredId of REQUIRED_ADR_IDS) {
    const filePath = indexed.get(requiredId);
    if (!filePath) {
      errors.push({
        file: `${requiredId}-*.md`,
        rule: "ADR_FILE_REQUIRED",
        detail: `required ADR file for ${requiredId} not found`,
      });
      continue;
    }

    checked.push(basename(filePath));
    errors.push(...verifyAdrFile(filePath));
  }

  return {
    ok: errors.length === 0,
    checked,
    errors,
  };
}

function runCli() {
  const result = verifyAdrFrontmatter();

  if (!result.ok) {
    for (const error of result.errors) {
      process.stderr.write(`${error.file}: [${error.rule}] ${error.detail}\n`);
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
