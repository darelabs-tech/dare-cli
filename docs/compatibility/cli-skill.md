# CLI: `dare skill` (list / info / add / remove / update / publish)

Microplanos **044** + **045**. Crate `dare-skills`. Decisões: [DEC-033](../DECISION-LOG.md) (registry), [DEC-043](../DECISION-LOG.md) (lifecycle).

## Superfície

| Comando | Disco | Saída |
|---------|-------|-------|
| `dare skill list` | lê `.dare/skills.yml` se existir | lista merged |
| `dare skill info <name>` | idem | detalhe ou NotFound **3** |
| `dare skill add <name> [--version] [--from <archive>]` | escreve `packages/skills/<name>/` + upsert `.dare/skills.yml` | InstallReport |
| `dare skill remove <name>` | apaga dir + remove manifest (bloqueia reverse-deps) | RemoveReport |
| `dare skill update <name> [--from]` | recopia conteúdo + atualiza manifest | InstallReport |
| `dare skill publish <name> [--out <dir>]` | `*.tar.gz` + `.sha256` (+ `.minisig` se chave) | PublishReport |

Globais: `--json`, `--no-color`.

## Prioridade de registries

`remote > local > mock` (mesmo `name` → fonte de maior prioridade).

| Fonte | Root / URL | Soft-fail |
|-------|------------|-----------|
| Remote | `https://dare-registry.vercel.app` (override `DARE_REMOTE_REGISTRY`) | timeout **3 s**; erro → vazio |
| Local | `~/.dare/registry` ou `DARE_LOCAL_REGISTRY` | entries inválidas skipped |
| Mock | embutido `registry-mock.json` (**7** skills) | sempre disponível |

`DARE_REMOTE_REGISTRY=off` (ou vazio) desliga o remoto (útil em CI).

## Lifecycle (045)

- **Install atômico:** staging `packages/skills/.staging-<name>/` → rename para `packages/skills/<name>/`.
- **Deps:** `resolve_dependencies` instala dependências ausentes antes do alvo.
- **Mock/remote sem ficheiros:** materializa `skill.yml` + `SKILL.md` stub.
- **Local:** copia `~/.dare/registry/<name>/<version>/` quando presente.
- **`--from`:** extrai `.tar` / `.tar.gz` / `.tgz` / `.zip` com **path jail** (bloqueia `..`, absolutos, escape).
- **Remove:** apaga a árvore no disco (correção vs TS). Bloqueia se outra skill instalada lista o alvo em `depends_on`.
- **Update:** recopia conteúdo (correção vs TS que só tocava o manifest).
- **Publish:** exige `license: MIT` e `dare_version` em `skill.yml`; escreve artefato + SHA-256; assinatura Ed25519 se `DARE_SKILL_PRIVATE_KEY` (64 hex chars).

## Modelo

- `RegistrySkill` — entrada de list/info (`kind`: generic|stack, `source`: mock|local|remote)
- `SkillManifest` — schema de `skill.yml` de pacote
- Projeto: `dare-contracts::SkillsManifest` ← `.dare/skills.yml` (`SkillEntry.id` = nome)
- Seis genéricas: `dare-ax`, `dare-frontend-design`, `dare-layered-design`, `dare-llm-integration`, `dare-quality-telemetry`, `dare-realtime`
- Demais (ex. `skill-*`) → `stack`
- `resolve_dependencies` — ordem topológica; ciclo → InvalidInput **4**

## Lockfile

**Não** implementado (paridade Mestre §4.1 / DEC-033).

## Exit codes

| Code | Quando |
|------|--------|
| 0 | OK |
| 2 | Usage |
| 3 | `info` / install alvo / remove alvo ausente |
| 4 | InvalidInput (ciclo, traversal, MIT, reverse-deps, name path-unsafe) |
| 5 | Io |

## Compat

| Diff vs TS 3.18.1 | Classe | Nota |
|-------------------|--------|------|
| `info` usa merge remote>local>mock (TS: mock-only p/ info/deps) | B | DEC-033 |
| Sem lockfile | A | Preservar |
| Soft-fail remoto | A | Paridade |
| `remove` apaga ficheiros (TS não apagava) | C | DEC-043 |
| `update` recopia conteúdo (TS só manifest) | C | DEC-043 |
| `publish` envia tarball+hash+sig (TS só metadados) | C | DEC-043 |
| Path traversal em zip/tar bloqueado | C (hardening) | DEC-043 |
