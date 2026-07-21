# Antigravity harness adapter

Microplano **014**. Crate `dare-harness`. Decisão: [DEC-015](../DECISION-LOG.md) · [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md).

## CLI

```bash
dare harness antigravity detect [--root <path>]
dare harness antigravity install [--root <path>] [--force]
dare harness antigravity validate [--root <path>]
```

| Subcomando | Side effects | Mensagem (en-US) |
|------------|--------------|------------------|
| `detect` | nenhum | `harness antigravity detect: rules={} dir={} skills={} workflows={}` |
| `install` | rules + workflows + commands + skills | `harness antigravity install: wrote {n} commands + skills/rules` |
| `validate` | nenhum | `harness antigravity validate: ok ({n} commands)` |

`--force`: sobrescreve ficheiros **unmanaged**. Default: preserve.

Ordem install: `generate_antigravityrules` → `ensure_workflows_dir` → `install_antigravity`.

## Artefactos

| Path | Papel |
|------|-------|
| `.antigravityrules` | Rules managed |
| `.antigravity/commands/*.md` | Commands (matrix `outputs.antigravity`) |
| `.agents/skills/<id>/SKILL.md` | Skills partilhadas com Codex |
| `.agents/workflows/.gitkeep` | Marcador de dir (workflows vazios / paridade TS) |

## Frontmatter

Skills devem ter bloco `---` com `name:` e `description:` non-empty (`validate_skill_frontmatter`).

## Contagens

| Item | Valor |
|------|-------|
| Commands SoT | **49** (`outputs.antigravity`) |
| Package skills “48” | Exception Classe C `agent-skills-full-parity` |

## Coexistência Codex

Mesmo corpo em `.agents/skills` via `render_agent_skill`. Install Codex após Antigravity com `!force` não deve invalidar validate Antigravity.

## Trade-offs (resumo)

SoT 49; workflows = `.gitkeep`; share Codex; frontmatter MUST; release = microplano 015.

## Segurança (RS)

Path jail; sem secrets; frontmatter parse-only; help `--force`; atomic_write; validate não apaga.

## DEC-015

Adapter Antigravity: rules + commands + shared `.agents/skills`; frontmatter validate.
