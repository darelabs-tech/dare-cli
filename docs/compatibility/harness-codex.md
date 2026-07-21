# Codex harness adapter

Microplano **013**. Crate `dare-harness`. Decisão: [DEC-014](../DECISION-LOG.md) · [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md).

## CLI

```bash
dare harness codex detect [--root <path>]
dare harness codex install [--root <path>] [--force]
dare harness codex validate [--root <path>]
```

| Subcomando | Side effects | Mensagem (en-US) |
|------------|--------------|------------------|
| `detect` | nenhum | `harness codex detect: agents_md={} codex_dir={} agents_skills={}` |
| `install` | `AGENTS.md` + `.codex/skills/**` + `.agents/skills/**` | `harness codex install: wrote {n} skills + AGENTS.md` |
| `validate` | nenhum | `harness codex validate: ok ({n} skills)` |

`--force`: sobrescreve ficheiros **unmanaged**. Default: preserve.

Ordem install: `generate_agents_md` → `install_codex_skills`.

## `$skill-name`

`AGENTS.md` lista `- $<id>` por capability com `outputs.codex` (ex.: `$dare-design`). Invocar skills com `$skill-name`.

## Preserve / coexistência Antigravity

- Managed: 1ª linha `<!-- dare:managed` **ou** `---` (frontmatter).
- Unmanaged não são sobrescritos sem `--force`.
- Skills partilhadas em `.agents/skills/{id}/SKILL.md` (mesmo corpo que `.codex/skills/...`) — Antigravity reutiliza sem duplicar conteúdo divergente.

## Contagens

| Item | Valor |
|------|-------|
| Paths `outputs.codex` (SoT) | **49** |
| Package skills “48” | Exception Classe C `agent-skills-full-parity` (registry 044+) |

## Update policies

`UPDATE_HARNESS_IDES` inclui `"codex"` (`update_policies_include_codex() == true`). Wiring completo de `dare update` → microplano 021+.

## Trade-offs (resumo)

Dual write `.codex` + `.agents`; SoT 49; exception 48; constante update; sem adapter Antigravity neste ciclo.

## Segurança (RS)

Path jail; sem secrets em AGENTS/skills; help `--force`; atomic_write; validate não apaga; adapter **não** executa skills.

## DEC-014

Adapter Codex: AGENTS.md + skills; `.agents` share; `UPDATE_HARNESS_IDES` includes codex.
