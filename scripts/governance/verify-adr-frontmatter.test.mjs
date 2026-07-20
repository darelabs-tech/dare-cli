import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

import {
  REQUIRED_ADR_IDS,
  verifyAdrFile,
  verifyAdrFrontmatter,
} from "./verify-adr-frontmatter.mjs";
import { resolveRepoRoot } from "./verify-structure.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const validFixture = join(here, "fixtures", "adr-valid.md");
const proposedFixture = join(here, "fixtures", "adr-proposed.md");
const repoRoot = resolveRepoRoot();
const repoAdrDir = join(repoRoot, "docs", "adr");

/**
 * @param {string} tempAdrDir
 */
function seedRequiredAdrsFromRepo(tempAdrDir) {
  mkdirSync(tempAdrDir, { recursive: true });

  for (const name of readdirSync(repoAdrDir)) {
    const id = name.match(/^(ADR-\d{3})-/)?.[1];
    if (id && REQUIRED_ADR_IDS.includes(id)) {
      copyFileSync(join(repoAdrDir, name), join(tempAdrDir, name));
    }
  }
}

describe("verify-adr-frontmatter", () => {
  it("should_fail_STATUS_ACCEPTED_on_proposed_fixture", () => {
    const errors = verifyAdrFile(proposedFixture);
    assert.ok(errors.some((error) => error.rule === "STATUS_ACCEPTED"));
  });

  it("should_pass_on_valid_fixture", () => {
    const tempDir = mkdtempSync(join(tmpdir(), `dare-adr-valid-${randomUUID()}-`));
    const target = join(tempDir, "ADR-001-fixture-valid.md");
    copyFileSync(validFixture, target);

    const errors = verifyAdrFile(target);
    assert.deepEqual(errors, []);
  });

  it("should_pass_on_repo_adrs_when_accepted", () => {
    const result = verifyAdrFrontmatter();
    assert.equal(result.ok, true, JSON.stringify(result.errors));
    assert.equal(result.checked.length, REQUIRED_ADR_IDS.length);
  });

  it("should_not_fail_if_adr_003_extra_exists", () => {
    const tempAdrDir = mkdtempSync(join(tmpdir(), `dare-adr-003-${randomUUID()}-`));
    seedRequiredAdrsFromRepo(tempAdrDir);

    writeFileSync(
      join(tempAdrDir, "ADR-003-idioma-misto-futuro.md"),
      `---
id: ADR-003
title: "Extra ADR ignored by verifier"
status: Proposed
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance"]
---

## Contexto
Ignored.

## Decisão
Ignored.

## Consequências
Ignored.

## Critérios de aceite
Ignored.

## Referências
Ignored.
`,
    );

    const result = verifyAdrFrontmatter(tempAdrDir);
    assert.equal(result.ok, true, JSON.stringify(result.errors));
  });

  it("should_fail_ADR_FILE_REQUIRED_when_adr_001_missing", () => {
    const tempAdrDir = mkdtempSync(
      join(tmpdir(), `dare-adr-missing-${randomUUID()}-`),
    );
    seedRequiredAdrsFromRepo(tempAdrDir);

    unlinkSync(join(tempAdrDir, "ADR-001-compatibilidade-bugs-legados.md"));

    const result = verifyAdrFrontmatter(tempAdrDir);
    assert.equal(result.ok, false);
    assert.ok(
      result.errors.some(
        (error) =>
          error.rule === "ADR_FILE_REQUIRED" && error.file.includes("ADR-001"),
      ),
    );
  });
});
