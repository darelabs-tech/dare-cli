import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { randomUUID } from "node:crypto";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

import {
  exitCodeForFailure,
  scanForSecrets,
  validateManifestFields,
  verifyBaseline,
} from "./verify-baseline.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const scriptPath = join(here, "verify-baseline.mjs");
const validFixture = join(here, "fixtures", "manifest-valid.json");
const badHashFixture = join(here, "fixtures", "manifest-bad-hash.json");
const realManifest = join(
  repoRoot,
  "docs",
  "compatibility",
  "baseline-manifest.json",
);

/**
 * @returns {string | null}
 */
function packBaselineTarball() {
  const packDir = mkdtempSync(join(tmpdir(), "dare-baseline-pack-"));
  const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";
  const pack = spawnSync(
    npmCmd,
    ["pack", "@dewtech/dare-cli@3.18.1", "--pack-destination", packDir],
    { encoding: "utf8" },
  );

  if (pack.status !== 0) {
    return null;
  }

  const packedName = (pack.stdout || "").trim().split(/\r?\n/).at(-1)?.trim();
  if (!packedName) {
    const files = readdirSync(packDir).filter((name) => name.endsWith(".tgz"));
    if (files.length === 1) {
      return join(packDir, files[0]);
    }
    return null;
  }

  return join(packDir, packedName);
}

function runCli(env = {}) {
  return spawnSync(process.execPath, [scriptPath], {
    encoding: "utf8",
    cwd: repoRoot,
    env: { ...process.env, ...env },
  });
}

describe("verify-baseline", () => {
  it("should_exit_1_on_invalid_schema", async () => {
    const tempDir = mkdtempSync(join(tmpdir(), "dare-baseline-invalid-"));
    const manifestPath = join(tempDir, "manifest.json");
    writeFileSync(
      manifestPath,
      JSON.stringify({
        schema_version: "2.0",
        package_name: "@dewtech/dare-cli",
        package_version: "3.18.1",
        source: "npm",
        resolved_url:
          "https://registry.npmjs.org/@dewtech/dare-cli/-/dare-cli-3.18.1.tgz",
        content_hash_alg: "sha256",
        content_hash:
          "991121297f89c8360f865e90baba7586eb71c93eb2f3216b63453d16c76ce5af",
      }),
    );

    const result = await verifyBaseline({ manifestPath });
    assert.equal(result.ok, false);
    assert.equal(result.code, "SCHEMA_INVALID");
    assert.equal(exitCodeForFailure(result), 1);
  });

  it("should_exit_2_on_hash_mismatch", async () => {
    const tarballPath = packBaselineTarball();
    if (!tarballPath) {
      return;
    }

    const previous = process.env.GOVERNANCE_TARBALL_PATH;
    process.env.GOVERNANCE_TARBALL_PATH = tarballPath;

    try {
      const result = await verifyBaseline({ manifestPath: badHashFixture });
      assert.equal(result.ok, false);
      assert.equal(result.code, "HASH_MISMATCH");
      assert.equal(exitCodeForFailure(result), 2);
    } finally {
      if (previous === undefined) {
        delete process.env.GOVERNANCE_TARBALL_PATH;
      } else {
        process.env.GOVERNANCE_TARBALL_PATH = previous;
      }
    }
  });

  it("should_exit_0_on_real_tarball", async () => {
    const result = await verifyBaseline({ manifestPath: realManifest });
    if (!result.ok && result.code === "DOWNLOAD_FAILED") {
      return;
    }

    assert.equal(result.ok, true);
    assert.equal(result.package_version, "3.18.1");
    assert.equal(result.matched, true);
    assert.match(result.content_hash, /^[a-f0-9]{64}$/);
    assert.notEqual(
      result.content_hash,
      "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    const cli = runCli();
    if (cli.status === 2) {
      const parsed = JSON.parse((cli.stderr || cli.stdout).trim());
      if (parsed.code === "DOWNLOAD_FAILED") {
        return;
      }
    }

    assert.equal(cli.status, 0);
    const output = JSON.parse((cli.stdout || "").trim());
    assert.equal(output.ok, true);
    assert.equal(output.matched, true);
  });

  it("should_exit_1_when_manifest_missing", async () => {
    const missingPath = join(
      tmpdir(),
      `dare-missing-manifest-${randomUUID()}.json`,
    );

    const result = await verifyBaseline({ manifestPath: missingPath });
    assert.equal(result.ok, false);
    assert.equal(result.code, "SCHEMA_INVALID");
    assert.equal(exitCodeForFailure(result), 1);
  });

  it("should_exit_2_when_registry_offline_without_tarball", async () => {
    const tempDir = mkdtempSync(join(tmpdir(), "dare-baseline-offline-"));
    const manifestPath = join(tempDir, "manifest-offline.json");
    writeFileSync(manifestPath, JSON.stringify({
      schema_version: "1.0",
      package_name: "@dewtech/dare-cli",
      package_version: "3.18.1",
      source: "npm",
      resolved_url: "https://127.0.0.1:9/unreachable/dare-cli-3.18.1.tgz",
      content_hash_alg: "sha256",
      content_hash:
        "991121297f89c8360f865e90baba7586eb71c93eb2f3216b63453d16c76ce5af",
    }));

    const previous = process.env.GOVERNANCE_TARBALL_PATH;
    delete process.env.GOVERNANCE_TARBALL_PATH;

    const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";
    const blockedPack = spawnSync(
      npmCmd,
      ["pack", "@dewtech/dare-cli@3.18.1", "--pack-destination", tempDir],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          GOVERNANCE_TARBALL_PATH: "",
          npm_config_registry: "http://127.0.0.1:9",
        },
      },
    );

    try {
      if (blockedPack.status === 0) {
        return;
      }

      const result = await verifyBaseline({ manifestPath });
      assert.equal(result.ok, false);
      assert.equal(result.code, "DOWNLOAD_FAILED");
      assert.equal(exitCodeForFailure(result), 2);
    } finally {
      if (previous === undefined) {
        delete process.env.GOVERNANCE_TARBALL_PATH;
      } else {
        process.env.GOVERNANCE_TARBALL_PATH = previous;
      }
    }
  });

  it("should_reject_secret_like_substrings", async () => {
    const tempDir = mkdtempSync(join(tmpdir(), "dare-baseline-secret-"));
    const manifestPath = join(tempDir, "manifest-secret.json");
    writeFileSync(
      manifestPath,
      '{"schema_version":"1.0","token=":"leak","package_name":"@dewtech/dare-cli"}',
    );

    const secretScan = scanForSecrets('{"url":"https://x?token=abc"}');
    assert.equal(secretScan.ok, false);

    const result = await verifyBaseline({ manifestPath });
    assert.equal(result.ok, false);
    assert.equal(result.code, "SCHEMA_INVALID");
    assert.equal(exitCodeForFailure(result), 1);

    const fields = validateManifestFields({
      schema_version: "1.0",
      package_name: "@dewtech/dare-cli",
      package_version: "3.18.1",
      source: "npm",
      resolved_url:
        "https://registry.npmjs.org/@dewtech/dare-cli/-/dare-cli-3.18.1.tgz",
      content_hash_alg: "sha256",
      content_hash:
        "991121297f89c8360f865e90baba7586eb71c93eb2f3216b63453d16c76ce5af",
    });
    assert.equal(fields.ok, true);
  });
});
