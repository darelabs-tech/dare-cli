# dare-steering Skill

Expõe `dare steering list|show` na IDE Antigravity — inspeção determinística de steering files.

## Como usar

```bash
dare steering list [--json]
dare steering show <file> [--json]
```

- `list` — PROJECT-DNA + `.dare/steering/*.md` na ordem de precedência.
- `show` — blocos aplicáveis ao `<file>` (base → project → glob).

Sem LLM. Saída estável com `--json`.
