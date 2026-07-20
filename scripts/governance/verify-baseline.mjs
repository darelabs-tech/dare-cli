import { createHash } from "node:crypto";
import {
  createWriteStream,
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";

import { resolveRepoRoot } from "./verify-structure.mjs";

const DEFAULT_MANIFEST = "docs/compatibility/baseline-manifest.json";
const EXPECTED_PACKAGE_NAME = "@dewtech/dare-cli";
const EXPECTED_PACKAGE_VERSION = "3.18.1";
const SECRET_SUBSTRINGS = ["token=", "Bearer ", "npm_", "ghp_", "AKIA"];
const HASH_REGEX = /^[a-f0-9]{64}$/;

/**
 * @param {string} text
 * @returns {{ ok: true } | { ok: false; substring: string }}
 */
export function scanForSecrets(text) {
  for (const substring of SECRET_SUBSTRINGS) {
    if (text.includes(substring)) {
      return { ok: false, substring };
    }
  }
  return { ok: true };
}

/**
 * @param {unknown} manifest
 * @returns {{ ok: true; manifest: Record<string, unknown> } | { ok: false; code: "SCHEMA_INVALID" | "VERSION_MISMATCH"; message: string }}
 */
export function validateManifestFields(manifest) {
  if (typeof manifest !== "object" || manifest === null || Array.isArray(manifest)) {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: "manifest must be a JSON object",
    };
  }

  const record = /** @type {Record<string, unknown>} */ (manifest);

  if (record.schema_version !== "1.0") {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: 'schema_version must be "1.0"',
    };
  }

  if (record.package_name !== EXPECTED_PACKAGE_NAME) {
    return {
      ok: false,
      code: "VERSION_MISMATCH",
      message: `package_name must be "${EXPECTED_PACKAGE_NAME}"`,
    };
  }

  if (record.package_version !== EXPECTED_PACKAGE_VERSION) {
    return {
      ok: false,
      code: "VERSION_MISMATCH",
      message: `package_version must be "${EXPECTED_PACKAGE_VERSION}"`,
    };
  }

  if (record.source !== "npm") {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: 'source must be "npm"',
    };
  }

  if (record.content_hash_alg !== "sha256") {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: 'content_hash_alg must be "sha256"',
    };
  }

  if (
    typeof record.content_hash !== "string" ||
    !HASH_REGEX.test(record.content_hash)
  ) {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: "content_hash must be a 64-character lowercase hex string",
    };
  }

  if (
    typeof record.resolved_url !== "string" ||
    !record.resolved_url.startsWith("https://")
  ) {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: "resolved_url must be an https URL string",
    };
  }

  return { ok: true, manifest: record };
}

/**
 * @param {string} filePath
 * @returns {string}
 */
export function sha256File(filePath) {
  const bytes = readFileSync(filePath);
  return createHash("sha256").update(bytes).digest("hex");
}

/**
 * @param {string} version
 * @param {string} destDir
 * @returns {string}
 */
function npmPackTarball(version, destDir) {
  const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";
  const result = spawnSync(
    npmCmd,
    [
      "pack",
      `${EXPECTED_PACKAGE_NAME}@${version}`,
      "--pack-destination",
      destDir,
    ],
    { encoding: "utf8", cwd: destDir },
  );

  if (result.status !== 0) {
    throw new Error(
      `npm pack failed: ${(result.stderr || result.stdout || "").trim()}`,
    );
  }

  const stdout = (result.stdout || "").trim();
  const lastLine = stdout.split(/\r?\n/).at(-1)?.trim() ?? "";
  const candidate = join(destDir, lastLine);

  if (lastLine.endsWith(".tgz") && existsSync(candidate)) {
    return candidate;
  }

  const packed = readdirSync(destDir).filter((name) => name.endsWith(".tgz"));
  if (packed.length === 1) {
    return join(destDir, packed[0]);
  }

  throw new Error("npm pack did not produce a tarball");
}

/**
 * @param {string} url
 * @param {string} destPath
 * @returns {Promise<void>}
 */
async function downloadTarball(url, destPath) {
  const response = await fetch(url, { redirect: "follow" });
  if (!response.ok || !response.body) {
    throw new Error(`download failed: HTTP ${response.status}`);
  }

  await pipeline(response.body, createWriteStream(destPath));
}

/**
 * @param {{ package_version: string; resolved_url: string }} manifest
 * @returns {Promise<{ path: string; cleanup: () => void }>}
 */
async function acquireTarball(manifest) {
  const envPath = process.env.GOVERNANCE_TARBALL_PATH;
  if (envPath && existsSync(envPath)) {
    return {
      path: resolve(envPath),
      cleanup: () => {},
    };
  }

  const tempDir = mkdtempSync(join(tmpdir(), "dare-baseline-"));
  let tarballPath;

  try {
    tarballPath = npmPackTarball(manifest.package_version, tempDir);
    return {
      path: tarballPath,
      cleanup: () => {
        try {
          rmSync(tempDir, { recursive: true, force: true });
        } catch {
          // best-effort temp cleanup
        }
      },
    };
  } catch (packError) {
    tarballPath = join(tempDir, "downloaded.tgz");
    try {
      await downloadTarball(manifest.resolved_url, tarballPath);
      return {
        path: tarballPath,
        cleanup: () => {
          try {
            rmSync(tempDir, { recursive: true, force: true });
          } catch {
            // best-effort temp cleanup
          }
        },
      };
    } catch (downloadError) {
      try {
        rmSync(tempDir, { recursive: true, force: true });
      } catch {
        // best-effort temp cleanup
      }

      const packMessage =
        packError instanceof Error ? packError.message : String(packError);
      const downloadMessage =
        downloadError instanceof Error
          ? downloadError.message
          : String(downloadError);
      throw new Error(
        `tarball acquisition failed (npm pack: ${packMessage}; download: ${downloadMessage})`,
      );
    }
  }
}

/**
 * @param {{
 *   manifestPath?: string;
 *   skipDownload?: boolean;
 *   expectedHashEnv?: string;
 * }} [opts]
 * @returns {Promise<
 *   | { ok: true; package_version: "3.18.1"; content_hash: string; matched: true }
 *   | { ok: false; code: "SCHEMA_INVALID" | "HASH_MISMATCH" | "DOWNLOAD_FAILED" | "VERSION_MISMATCH"; message: string }
 * >}
 */
export async function verifyBaseline(opts = {}) {
  const manifestPath = resolve(
    opts.manifestPath ?? join(resolveRepoRoot(), DEFAULT_MANIFEST),
  );

  if (!existsSync(manifestPath)) {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: `manifest not found: ${manifestPath}`,
    };
  }

  let rawText;
  try {
    rawText = readFileSync(manifestPath, "utf8");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: `cannot read manifest: ${message}`,
    };
  }

  const secretScan = scanForSecrets(rawText);
  if (!secretScan.ok) {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: `manifest contains forbidden substring: ${secretScan.substring}`,
    };
  }

  let parsed;
  try {
    parsed = JSON.parse(rawText);
  } catch {
    return {
      ok: false,
      code: "SCHEMA_INVALID",
      message: "manifest is not valid JSON",
    };
  }

  const schemaResult = validateManifestFields(parsed);
  if (!schemaResult.ok) {
    return schemaResult;
  }

  const manifest = schemaResult.manifest;
  const contentHash = /** @type {string} */ (manifest.content_hash);

  if (opts.skipDownload) {
    return {
      ok: true,
      package_version: "3.18.1",
      content_hash: contentHash,
      matched: true,
    };
  }

  if (opts.expectedHashEnv) {
    const expected = process.env[opts.expectedHashEnv];
    if (typeof expected !== "string" || expected !== contentHash) {
      return {
        ok: false,
        code: "HASH_MISMATCH",
        message: "expected hash from env does not match manifest content_hash",
      };
    }

    return {
      ok: true,
      package_version: "3.18.1",
      content_hash: contentHash,
      matched: true,
    };
  }

  let tarball;
  try {
    tarball = await acquireTarball({
      package_version: EXPECTED_PACKAGE_VERSION,
      resolved_url: /** @type {string} */ (manifest.resolved_url),
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      code: "DOWNLOAD_FAILED",
      message,
    };
  }

  try {
    const computedHash = sha256File(tarball.path);
    if (computedHash !== contentHash) {
      return {
        ok: false,
        code: "HASH_MISMATCH",
        message: "computed tarball hash does not match manifest content_hash",
      };
    }

    return {
      ok: true,
      package_version: "3.18.1",
      content_hash: computedHash,
      matched: true,
    };
  } finally {
    tarball.cleanup();
  }
}

/**
 * @param {{ ok: false; code: string }} result
 * @returns {1 | 2}
 */
export function exitCodeForFailure(result) {
  switch (result.code) {
    case "DOWNLOAD_FAILED":
    case "HASH_MISMATCH":
      return 2;
    default:
      return 1;
  }
}

async function runCli() {
  const result = await verifyBaseline();

  if (result.ok) {
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(0);
  }

  process.stderr.write(`${JSON.stringify(result)}\n`);
  process.exit(exitCodeForFailure(result));
}

const isMain =
  process.argv[1] &&
  resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));

if (isMain) {
  runCli().catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(
      `${JSON.stringify({ ok: false, code: "DOWNLOAD_FAILED", message })}\n`,
    );
    process.exit(2);
  });
}
