# Cursor harness adapter

Microplano **012**. Crate `dare-harness`. Decisão: [DEC-013](../DECISION-LOG.md) · [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md).

## CLI

```bash
dare harness cursor detect [--root <path>]
dare harness cursor install [--root <path>] [--force]
dare harness cursor validate [--root <path>]
```

| Subcomando | Side effects | Mensagem (en-US) |
|------------|--------------|------------------|
| `detect` | nenhum | `harness cursor detect: cursor_dir={bool} cursorrules={bool}` |
| `install` | `.cursorrules` + `.cursor/commands/*.md` | `harness cursor install: wrote {n} commands` |
| `validate` | nenhum | `harness cursor validate: ok ({n} commands)` |

`--force`: sobrescreve ficheiros **unmanaged**. Default: preserve.

Ordem install: `generate_cursorrules` → `install_cursor_commands` (**sem** rules `.mdc`).

## Preserve

Artefactos com 1ª linha `<!-- dare:managed … -->` são managed. Unmanaged não são sobrescritos sem `--force`.

## Contagens (SoT vs baseline TS)

| Item | Valor |
|------|-------|
| Paths `outputs.cursor` na matrix | **49** (SoT) |
| Baseline legado “33 commands” | Exception Classe C `cursor-commands-full-parity` |
| Baseline “25 rules” `.mdc` | Exception Classe C `cursor-rules-full-parity` — **deferred** neste ciclo |

Não reduzir a matrix a 33 sem ADR. Aceite do microplano = cobertura matrix **ou** exceptions (ambas presentes).

## Deferred / Class C

**Não** neste ciclo:

- `install_cursor_rules`
- `validate_mdc_frontmatter`
- rules condicionais de stack

Backlog: inventário `.mdc` em assets + remover/atualizar exception quando houver cobertura real.

## Trade-offs (T-01…T-14 resumo)

Adapter em `cursor.rs`; render via `render_claude_command`; SoT 49; rules deferidas; marcador managed igual ao 011; validate = existência de ficheiro.

## Segurança (RS)

Path jail (`ProjectRoot` / `SafeRelativePath` / `atomic_write`); sem secrets em stubs; help `--force` explícito; validate não apaga ficheiros.

## DEC-013

Adapter Cursor: commands from matrix; `.cursorrules` managed; preserve unmanaged; CLI `dare harness cursor`.
