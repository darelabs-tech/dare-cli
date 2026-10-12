# Operational Playbook (Draft)

## First checks
1. Run `dare info` or `dare --json info` in the project directory.
2. For `DARE_CONFIG_INVALID_JSON`, repair `dare.config.json` syntax; do not paste its contents into public tickets.
3. For `DARE_STATE_INVALID_JSON`, stop execution and restore a trusted `.dare/state.json` backup before resuming; do not manually erase state without understanding recovery semantics.
4. For `DARE_DAG_MISSING`, confirm the intended DAG path and run `dare validate` with the appropriate file.
5. Use `dare execute --status` for task state inspection.

## Security
Do not upload config, state, environment, logs or worktree contents without independent review and redaction. No automated support upload is implemented in this cycle yet.
