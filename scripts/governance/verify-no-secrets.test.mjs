import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { REQUIRED_ADR_IDS } from "./verify-adr-frontmatter.mjs";
import { verifyNoSecrets } from "./verify-no-secrets.mjs";
import { resolveRepoRoot } from "./verify-structure.mjs";

const repoRoot = resolveRepoRoot();
const repoAdrDir = join(repoRoot, "docs", "adr");

/**
 * @param {string} tempRoot
 */
function seedGovernanceTree(tempRoot) {
  mkdirSync(join(tempRoot, "docs", "compatibility"), { recursive: true });
  mkdirSync(join(tempRoot, "docs", "adr"), { recursive: true });

  copyFileSync(
    join(repoRoot, "docs", "compatibility", "baseline-manifest.json"),
    join(tempRoot, "docs", "compatibility", "baseline-manifest.json"),
  );

  for (const requiredId of REQUIRED_ADR_IDS) {
    const name = readdirSync(repoAdrDir).find((file) =>
      file.startsWith(`${requiredId}-`),
    );
    if (!name) {
      continue;
    }

    copyFileSync(
      join(repoAdrDir, name),
      join(tempRoot, "docs", "adr", name),
    );
  }
}

describe("verify-no-secrets", () => {
  it("should_pass_on_repo_manifest_and_adrs", () => {
    const result = verifyNoSecrets(repoRoot);
    assert.equal(result.ok, true, JSON.stringify(result.errors));
    assert.ok(result.scanned.includes("docs/compatibility/baseline-manifest.json"));
  });

  it("should_fail_when_manifest_contains_ghp_test", () => {
    const tempRoot = mkdtempSync(
      join(tmpdir(), `dare-no-secrets-manifest-${randomUUID()}-`),
    );
    seedGovernanceTree(tempRoot);

    const manifestPath = join(
      tempRoot,
      "docs",
      "compatibility",
      "baseline-manifest.json",
    );
    writeFileSync(
      manifestPath,
      `${readFileSync(manifestPath, "utf8")}\n# fixture leak ghp_test\n`,
    );

    const result = verifyNoSecrets(tempRoot);
    assert.equal(result.ok, false);
    assert.ok(
      result.errors.some(
        (error) =>
          error.rule === "NO_SECRETS" &&
          error.file.includes("baseline-manifest.json"),
      ),
    );
  });

  it("should_fail_when_adr_contains_ghp_test", () => {
    const tempRoot = mkdtempSync(
      join(tmpdir(), `dare-no-secrets-adr-${randomUUID()}-`),
    );
    seedGovernanceTree(tempRoot);

    const adrPath = join(
      tempRoot,
      "docs",
      "adr",
      "ADR-001-compatibilidade-bugs-legados.md",
    );
    writeFileSync(adrPath, `${readFileSync(adrPath, "utf8")}\nghp_test\n`);

    const result = verifyNoSecrets(tempRoot);
    assert.equal(result.ok, false);
    assert.ok(
      result.errors.some(
        (error) =>
          error.rule === "NO_SECRETS" && error.file.includes("ADR-001"),
      ),
    );
  });
});
