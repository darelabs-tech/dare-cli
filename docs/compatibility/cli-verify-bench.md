# CLI verify advanced + bench (`dare bench` / execute flags)

> **DEC-050** · Microplano 049 · Library: `crates/dare-verify` · CLI: `commands/bench.rs`, execute flags in `main.rs`

## Purpose

Extend post-Ralph verification with advanced aspects (fail-to-pass, anti-tamper, mutation, formal) and a deterministic Fix·Rate harness via `dare bench`. Ralph gates (`Build` / `Test` / `Lint`) stay unchanged; advanced aspects use `AdvancedAspect` / `AspectResult`.

| Surface | Role |
|---------|------|
| `dare bench` | Run fixture suite → `BenchReport` (Fix·Rate + solve-rate) |
| `dare execute --complete` / `--agent` | After Ralph OK → `run_advanced_verify` → `LoopVerdict` |
| `--best-of N` | Candidate worktrees + Pareto winner |
| `--policy decay` | Decay actions after repeated failure signatures |

## Advanced aspects

| Aspect | Report string | Default |
|--------|---------------|---------|
| Fail-to-pass | `fail-to-pass` | On when verify enabled; skipped if no FTP list |
| Anti-tamper | `anti-tamper` | On when verify enabled |
| Mutation | `mutation` | Incremental; threshold **0.70**; tool missing → `skipped` unless `--full-mutation` |
| Formal | `formal` | Off; opt-in `--formal` or `verify.formal.enabled` |

`AspectStatus`: `pass` / `fail` / `skipped`. Only `fail` blocks DONE (except explicit policies above).

Constants: `MUTATION_THRESHOLD=0.70`, `REPAIR_MAX=5`, `BEST_OF_MAX=8`, `DECAY_WINDOW=3`, `DECAY_MAX_ATTEMPTS=5`, `ADVANCED_TIMEOUT_SECS=600`.

## Fix·Rate (frozen)

Per fixture `F` with `fail_to_pass.txt` / `pass_to_pass.txt` (one id per line; `#` and blanks ignored):

1. Any `pass_to_pass` failure → `fixtureFixRate = 0.0`, `fixtureOk = false`.
2. Else let `A = |fail_to_pass|`, `B` = how many now pass.  
   - `A == 0` → `fixtureFixRate = 1.0` if ok else `0.0`.  
   - Else → `fixtureFixRate = B / A`.
3. `suiteFixRate = mean(fixtureFixRate)` (JSON rounds to **4** decimals half-up).
4. `solveRate = (count fixtureOk) / (count fixtures)`.

Regression with `--baseline` + `--fail-on-regression N` (`N` ∈ 0..=100):

```
drop_pp = (baseline.solveRate - current.solveRate) * 100.0
if drop_pp > N → exit 1
```

Without `--fail-on-regression`: report drop, exit 0 (unless suite invalid).  
`--fail-on-regression` without `--baseline` → Usage exit **2**.

## `dare bench` flags

```text
dare bench
  [--suite <DIR>]                 # default fixtures/bench (relative to -d/cwd)
  [--json]
  [--baseline <FILE>]
  [--fail-on-regression <N>]      # 0..=100 percentage points
  [--filter <GLOB>]               # glob on case id
  [-d|--dir <PATH>]
```

Suite: `suite.json` schemaVersion **1** + case dirs with `patch.diff`, `fail_to_pass.txt`, `pass_to_pass.txt`, `repo/`. Work under `.dare/bench-work/` (suite tree not mutated).

### Exit codes (`dare bench`)

| Code | When |
|------|------|
| 0 | Suite ran; no regression failure |
| 1 | Regression failed (`--fail-on-regression`) |
| 2 | Invalid suite/baseline, usage (e.g. regression without baseline) |
| 4 | InvalidInput (`N` outside 0..=100) |
| 5 | Io |
| 124 | Timeout |

## Execute flags (additive)

| Flag | Default | Effect |
|------|---------|--------|
| `--verify` / `--no-verify` | verify on (`verify.enabled`) | Skip advanced path when off |
| `--full-mutation` | false | Full mutation; tool missing → fail |
| `--formal` / `--no-formal` | false | Opt-in formal (`dafny`\|`verus`\|`lean`) |
| `--formal-backend` | `dafny` | Backend when formal on |
| `--best-of <n>` | none | `1..=8`; worktrees `.dare/worktrees/cand-{n}/` |
| `--policy` | `fixed` | `fixed`\|`decay` (decay requires `--agent`) |
| `--verdict-json` | false | Print `LoopVerdict` after complete |
| `--prerank` | false | Soft order candidates (no-op without `--best-of`) |

Unknown `--policy` → Usage **2**. `--best-of` out of range → **2**/`4` with `--best-of must be between 1 and 8`.

## Reports

- `LoopVerdict` / `BenchReport`: **schemaVersion 1** (camelCase JSON).
- Verification artifacts under `.dare/verification/`.

## Capability

`dare-bench` → `cli_commands: ["bench"]` in `assets/capability-matrix.yml`.

## Examples

```bash
dare bench --json
dare bench --suite fixtures/bench --filter 'sample-*'
dare bench --baseline bench-baseline.json --fail-on-regression 5
dare execute --complete mp049-001 --full-mutation --formal --verdict-json
dare execute --agent --policy decay --best-of 3 --driver mock
```

## Out of scope (049)

`dare ai` (**050**), dashboard/MCP (**051/052**), rewriting Ralph build/test/lint, Cargo deps for Dafny/Stryker.
