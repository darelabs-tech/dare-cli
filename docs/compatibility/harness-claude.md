# Claude Code harness adapter

Microplano **011**. Crate `dare-harness`. Decisão: [DEC-012](../DECISION-LOG.md) · capabilities: [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md).

## CLI

```bash
dare harness claude detect [--root <path>]
dare harness claude install [--root <path>] [--force]
dare harness claude validate [--root <path>]
```

| Subcomando | Side effects | Mensagem (en-US) |
|------------|--------------|------------------|
| `detect` | nenhum | `harness claude detect: claude_md={bool} claude_dir={bool}` |
| `install` | `CLAUDE.md` + `.claude/commands/*.md` + `.claude/settings.json` | `harness claude install: wrote {n} commands` |
| `validate` | nenhum | `harness claude validate: ok ({n} commands)` |

`--force`: sobrescreve ficheiros **unmanaged** (CLAUDE.md, commands, settings). Default: preserve.

Ordem do install: `generate_claude_md` → `install_commands` → `write_settings_json`.

## Preserve

| Artefacto | Managed se… | Sem `--force` |
|-----------|-------------|---------------|
| `CLAUDE.md` / commands | 1ª linha começa com `<!-- dare:managed` | skip overwrite |
| `.claude/settings.json` | contém `"_dare_managed"` | skip se ausente |

## Settings schema (mínimo gerado)

- `permissions.allow` — inclui `Read(DARE/**)`, `Write(DARE/**)`
- `hooks.PostToolUse` — matcher `"Write"`, `type: command`, comando fixo (Ralph Loop reminder; sem interpolação de input do user)
- `_dare_managed: true`

## Paths (49)

Os paths Claude instalados = `outputs.claude` de cada entry em `assets/capability-matrix.yml` (**49**). Fonte única; não há lista duplicada no adapter.

**SHOULD / baseline TS 3.18.1:** nomes/paths alinhados à matrix ADR-007. Drift intencional vs npm legado = Classe B/C no decision log / classification-matrix — não bloqueia validate se a matrix for a SoT.

## Trade-offs (T-01…T-14 resumo)

| ID | Escolha |
|----|---------|
| T-01 | Adapter em `dare-harness/src/claude.rs` |
| T-02 | Conteúdo via matrix + `render_claude_command` |
| T-03/T-04 | Marcadores managed md / `_dare_managed` |
| T-05/T-06 | Preserve default; `--force` overwrite |
| T-07 | Settings: skip ou replace (sem merge field-level) |
| T-08 | PostToolUse command constante |
| T-09/T-10 | Contagem 49; validate = existência de ficheiro |
| T-11 | CLAUDE.md stub (sem LLM) |
| T-12 | `SafeRelativePath` + `atomic_write` |
| T-13/T-14 | Outros adapters fora; golden paths SHOULD |

## Segurança (RS → adapter)

| RS | Controlo |
|----|----------|
| RS-01 | Matrix validate + SafeRelativePath antes de write |
| RS-02 | Stub/hook/commands sem secrets |
| RS-03 | Jail `ProjectRoot` |
| RS-04 | `cargo audit` + `cargo deny` (Ralph) |
| RS-05 | Settings gerados sem secrets |
| RS-06 | PostToolUse command fixo |
| RS-07 | Help `--force` explícito |
| RS-08 | `atomic_write` por ficheiro; validate não apaga |

## DEC-012

Adapter Claude: install from capability-matrix via `render_claude_command`; path jail + atomic_write; preserve unmanaged; CLI `dare harness claude`.
