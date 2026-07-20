#!/usr/bin/env node
/**
 * Wrapper so `npm test -- --passWithNoTests` (Ralph Loop nestjs gate) ignores extra flags.
 */
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '..', '..');
const tests = fs
  .readdirSync(here)
  .filter((f) => f.endsWith('.test.mjs'))
  .map((f) => path.join(here, f));

if (tests.length === 0) {
  console.error('No *.test.mjs files in scripts/governance');
  process.exit(1);
}

const result = spawnSync(process.execPath, ['--test', ...tests], {
  stdio: 'inherit',
  cwd: repoRoot,
});
process.exit(result.status === null ? 1 : result.status);
