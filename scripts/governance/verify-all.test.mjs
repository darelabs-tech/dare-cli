import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { randomUUID } from "node:crypto";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

import { runVerifyAll } from "./verify-all.mjs";
import { REQUIRED_PATHS, resolveRepoRoot } from "./verify-structure.mjs";

const repoRoot = resolveRepoRoot();
const here = dirname(fileURLToPath(import.meta.url));

/**
 * @param {string} tempRoot
 */
function seedRequiredPaths(tempRoot) {
  for (const relative of REQUIRED_PATHS) {
    const source = join(repoRoot, relative);
    const target = join(tempRoot, relative);
    mkdirSync(dirname(target), { recursive: true });
    copyFileSync(source, target);
  }

  for (const relative of [
    "scripts/governance/verify-baseline.mjs",
    "scripts/governance/verify-adr-frontmatter.mjs",
    "scripts/governance/verify-no-secrets.mjs",
    "scripts/governance/run-tests.mjs",
    "scripts/governance/package.json",
  ]) {
    const source = join(here, "..", "..", relative);
    const target = join(tempRoot, relative);
    mkdirSync(dirname(target), { recursive: true });
    copyFileSync(source, target);
  }
}

describe("verify-all", () => {
  it("should_exit_0_on_repo", async () => {
    const exitCode = await runVerifyAll();
    assert.equal(exitCode, 0);
  });

  it("should_fail_NO_SECRETS_when_adr_fixture_injected", async () => {
    const tempRoot = mkdtempSync(
      join(tmpdir(), `dare-verify-all-secret-${randomUUID()}-`),
    );
    seedRequiredPaths(tempRoot);

    const adrPath = join(
      tempRoot,
      "docs",
      "adr",
      "ADR-002-contrato-saida-json.md",
    );
    writeFileSync(adrPath, `${readFileSync(adrPath, "utf8")}\nghp_test\n`);

    const exitCode = await runVerifyAll(tempRoot);
    assert.equal(exitCode, 1);
  });
});
