# CLI: `dare skill` (list / info)

Microplano **044**. Crate `dare-skills`. Decisão: [DEC-032](../DECISION-LOG.md).

## Superfície

| Comando | Disco | Saída |
|---------|-------|-------|
| `dare skill list` | lê `.dare/skills.yml` se existir (não falha se ausente) | lista merged |
| `dare skill info <name>` | idem | detalhe ou NotFound **3** |

Globais: `--json`, `--no-color`.

**Fora deste ciclo:** `add` / `remove` / `update` / `publish` (→ **045**).

## Prioridade de registries

`remote > local > mock` (mesmo `name` → fonte de maior prioridade).

| Fonte | Root / URL | Soft-fail |
|-------|------------|-----------|
| Remote | `https://dare-registry.vercel.app` (override `DARE_REMOTE_REGISTRY`) | timeout **3 s**; erro → vazio |
| Local | `~/.dare/registry` ou `DARE_LOCAL_REGISTRY` | entries inválidas skipped |
| Mock | embutido `registry-mock.json` (**7** skills) | sempre disponível |

`DARE_REMOTE_REGISTRY=off` (ou vazio) desliga o remoto (útil em CI).

## Modelo

- `RegistrySkill` — entrada de list/info (`kind`: generic|stack, `source`: mock|local|remote)
- `SkillManifest` — schema de `skill.yml` de pacote
- Projeto: `dare-contracts::SkillsManifest` ← `.dare/skills.yml`
- Seis genéricas: `dare-ax`, `dare-frontend-design`, `dare-layered-design`, `dare-llm-integration`, `dare-quality-telemetry`, `dare-realtime`
- Demais (ex. `skill-*`) → `stack`
- `resolve_dependencies` — ordem topológica; ciclo → InvalidInput **4**

## Lockfile

**Não** implementado (paridade Mestre §4.1 / DEC-032).

## Exit codes

| Code | Quando |
|------|--------|
| 0 | OK |
| 2 | Usage |
| 3 | `info` skill ausente |
| 4 | InvalidInput / Config |
| 5 | Io |

## Compat

| Diff vs TS 3.18.1 | Classe | Nota |
|-------------------|--------|------|
| `info` usa merge remote>local>mock (TS: mock-only p/ info/deps) | B | DEC-032 |
| Sem lockfile | A | Preservar |
| Soft-fail remoto | A | Paridade |
