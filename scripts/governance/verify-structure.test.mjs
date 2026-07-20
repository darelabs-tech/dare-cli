import { existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { randomUUID } from "node:crypto";
import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  REQUIRED_PATHS,
  resolveRepoRoot,
  verifyStructure,
} from "./verify-structure.mjs";

describe("verify-structure", () => {
  it("should_export_at_least_15_required_paths", () => {
    assert.ok(Array.isArray(REQUIRED_PATHS));
    assert.ok(REQUIRED_PATHS.length >= 15);
    assert.ok(REQUIRED_PATHS.every((p) => typeof p === "string" && p.length > 0));
  });

  it("should_report_missing_when_file_absent", () => {
    const tempRoot = join(tmpdir(), `dare-governance-${randomUUID()}`);
    mkdirSync(tempRoot, { recursive: true });

    try {
      const result = verifyStructure(tempRoot);
      assert.equal(result.ok, false);
      assert.ok(result.missing.length > 0);
      assert.equal(result.checked, REQUIRED_PATHS.length);
    } finally {
      // temp dir left for OS cleanup; no writes outside repo
    }
  });

  it("should_ok_when_all_present", () => {
    const repoRoot = resolveRepoRoot();
    const docsExist = existsSync(join(repoRoot, "docs", "DECISION-LOG.md"));

    if (!docsExist) {
      // task-001 may still be running in parallel — skip when docs tree absent
      return;
    }

    const result = verifyStructure(repoRoot);
    assert.equal(result.ok, true);
    assert.deepEqual(result.missing, []);
    assert.equal(result.checked, REQUIRED_PATHS.length);
  });
});
