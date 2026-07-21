# Claude Code harness adapter

Microplano **011**. Crate `dare-harness`.

## CLI

```bash
dare harness claude detect
dare harness claude install [--force]
dare harness claude validate
```

## Preserve

Commands with first line `<!-- dare:managed … -->` are managed. Unmanaged files are not overwritten unless `--force`.
Settings: skip if present without `"_dare_managed": true`.

## DEC-012

Adapter Claude: install from capability-matrix via `render_claude_command`; path jail + atomic_write.
