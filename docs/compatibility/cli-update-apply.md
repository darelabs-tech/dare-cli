# CLI: `dare update` — aplicação (Ciclo 022)

Aplicação do `UpdatePlan` (021) com políticas keep/replace/ask, session backup versionado, migrations de `dare.config.json`, writes atómicos e rollback em falha. Complementa [DEC-023](../DECISION-LOG.md) e o Blueprint 022 §5.12. Planeamento / `--dry-run` → [`cli-update-plan.md`](cli-update-plan.md) (021 / DEC-022).

## Flags

| Flag | Efeito |
|------|--------|
| `--dry-run` | Só planeamento (021): emite `UpdatePlan`, **zero writes**; ignora `--force` |
| `-y` / `--yes` | Non-interactive apply; **não** sobrescreve `customized` (ver matriz) |
| `--force` | Sobrescreve `customized` (com session backup); só efectivo no apply |
| `--target <harness>` | Filtra assets por harness id (021); **não** aceita semver |
| `-d` / `--dir <path>` | Diretório inicial para walk-up do project root (default: cwd) |
| `--json` | Envelope JSON (004); `data` = `UpdateApplyReport` schema 1 (apply) ou `UpdatePlan` (dry-run) |
| `--no-color` | Sem ANSI (global 004) |

```text
dare update --dry-run [--target <harness>] [-d <dir>]
dare update [-y|--yes] [--force] [--target <harness>] [-d <dir>]
```

**`-y` ≠ `--force`:** `-y` confirma apply sem prompts e **mantém** ficheiros `customized`; só `--force` (com ou sem `-y`) faz replace de customizações.

## Matriz §5.1 (`resolve_action`)

| status | force | yes | interactive | batch_replace | → action |
|--------|-------|-----|-------------|---------------|----------|
| identical | * | * | * | * | Keep |
| missing | * | * | * | * | Replace |
| apply | * | * | * | * | Replace |
| customized | true | * | * | * | Replace |
| customized | false | true | * | * | Keep |
| customized | false | false | false | * | Keep |
| customized | false | false | true | true | Replace |
| customized | false | false | true | false | Keep |

Resumo de produto (espelha Design §4.1):

| Classificação | Flags | Ação |
|---------------|-------|------|
| `identical` | qualquer | **keep** |
| `missing` | qualquer apply | **replace** (create; sem backup de destino) |
| `apply` | default / `-y` / `--force` | **replace** (+ backup se destino existe) |
| `customized` | TTY, sem `-y`/`--force` | **ask** batch → Y replace / N keep (default N) |
| `customized` | non-TTY, sem `--force` | **keep** |
| `customized` | `-y` only | **keep** (+ warning `kept customized: …`) |
| `customized` | `--force` (± `-y`) | **replace** (+ backup) |

`interactive` = `stdout` e `stdin` são TTY **e** `!yes` **e** `!force`. Prompt (stderr): `Replace all N customized files? [y/N]:` — `y`/`Y`/`yes` → replace todos; resto (incl. EOF) → keep.

## Backup: `.dare/backup-<ver>/` vs `.dare/backups/` (005) — Classe B

| Layout | Uso | Ciclo |
|--------|-----|-------|
| `.dare/backup-<cliVersion>/` | **Session backup** de `dare update` apply; espelha paths relativos do project | 022 (compat TS Mestre §21) |
| `.dare/backup-<cliVersion>-<utc>/` | Colisão se o dir da versão já existe (`YYYYMMDDThhmmssZ`) | 022 |
| `.dare/backups/<utc>-<sha8>/…` | Backup genérico de `dare-core` (migrate avulso / outros) | 005 |

**Classe B:** session root versionado alinhado ao TS (`.dare/backup-<ver>/`), **não** reutilizar `.dare/backups/` como root de sessão. Primitives 005 (`atomic_write`, path jail) continuam a escrever bytes sob o `backupRoot` da sessão. Migrate de config no apply usa o **mesmo** journal/session backup (evita dual layout).

Exemplo:

```text
.dare/backup-0.1.0-alpha.0/
  CLAUDE.md
  .claude/commands/dare-discover.md
  dare.config.json
```

`backupRoot` no report = path relativo POSIX (ou `null` se zero writes).

## Rollback

Journal de sessão (`SessionJournal`): `backup_root`, pares `(dest, backupRel)` em ordem cronológica, `created` (ficheiros novos), `created_dirs`.

Em **qualquer** erro durante apply/migrate:

1. Restaurar cada `(dest, bak)` em ordem **inversa**.
2. Remover ficheiros em `created` (rev); ignorar NotFound.
3. `rmdir` dirs em `created_dirs` (rev) se vazios.
4. **Não** apagar o dir `backupRoot` inteiro (auditoria).
5. Retornar `Err` original (se rollback falhar → Internal wrapping ambos).

Pós-falha: tree de assets == pré-apply (excepto o dir de backup que pode permanecer). Aplicação parcial **não** persiste. Em `Ok(report)`, `rolledBack` é sempre `false`.

## Migrate config (session journal)

Se o plan indicar migrate **ou** houver item `dare.config.json` com status ∈ `{missing, apply}` **ou** (`customized` **e** action = Replace):

1. `need_backup_root()`; se o ficheiro existe → `session_backup_file`.
2. `plan_migrate` com `write_schema_version: true` + apply in-memory + `save_dare_config`.
3. Registar em `migrated` (ex. `["dare.config.json"]`); created/replaced conforme existência prévia.

Se `customized` + `-y` (keep) → **não** migrar. Não chamar `apply_migrate` nested (um único journal; evita `.dare/backups/` paralelo).

## Schema `UpdateApplyReport` 1

| Campo JSON | Tipo | Nullable | Semântica |
|------------|------|----------|-----------|
| `schemaVersion` | `u32` | não | sempre `1` |
| `mode` | `string` | não | `"update"` |
| `cliVersion` | `string` | não | versão CLI |
| `projectRoot` | `string` | não | abs display |
| `backupRoot` | `string \| null` | sim | rel POSIX; `null` se nenhum write |
| `target` | `string \| null` | sim | harness id ou null |
| `force` | `bool` | não | |
| `yes` | `bool` | não | |
| `kept` | `string[]` | não | sorted |
| `created` | `string[]` | não | sorted |
| `replaced` | `string[]` | não | sorted |
| `skipped` | `string[]` | não | reserved; v1 pode ser `[]` |
| `backedUp` | `string[]` | não | dest paths backed up; sorted |
| `migrated` | `string[]` | não | ex. `["dare.config.json"]` |
| `warnings` | `string[]` | não | ex. kept customized |
| `rolledBack` | `bool` | não | `false` em sucesso |

Paths relativos POSIX ao project root; lists sorted lexico. Campos extras → bump `schemaVersion` + ADR.

Envelope 004: `{ "correlation_id", "data": <UpdateApplyReport>, "ok": true }`.

### Exemplo — sucesso (`-y`, um create)

```json
{
  "schemaVersion": 1,
  "mode": "update",
  "cliVersion": "0.1.0-alpha.0",
  "projectRoot": "C:/tmp/proj",
  "backupRoot": null,
  "target": null,
  "force": false,
  "yes": true,
  "kept": [],
  "created": ["AGENTS.md"],
  "replaced": [],
  "skipped": [],
  "backedUp": [],
  "migrated": [],
  "warnings": [],
  "rolledBack": false
}
```

### Exemplo — customized kept com `-y`

```json
{
  "schemaVersion": 1,
  "mode": "update",
  "cliVersion": "0.1.0-alpha.0",
  "projectRoot": "/tmp/customized-assets",
  "backupRoot": null,
  "target": null,
  "force": false,
  "yes": true,
  "kept": ["CLAUDE.md"],
  "created": [],
  "replaced": [],
  "skipped": [],
  "backedUp": [],
  "migrated": [],
  "warnings": ["kept customized: CLAUDE.md"],
  "rolledBack": false
}
```

### Exemplo — schema completo (Apêndice C Design)

```json
{
  "schemaVersion": 1,
  "mode": "update",
  "cliVersion": "0.1.0-alpha.0",
  "projectRoot": "/abs",
  "backupRoot": ".dare/backup-0.1.0-alpha.0",
  "target": null,
  "force": false,
  "yes": true,
  "kept": ["path/a"],
  "created": ["path/b"],
  "replaced": ["path/c"],
  "skipped": [],
  "backedUp": ["path/c"],
  "migrated": ["dare.config.json"],
  "warnings": [],
  "rolledBack": false
}
```

Human (en-US): mode, cliVersion, projectRoot, backupRoot, counts (kept/created/replaced/backedUp/migrated), warnings, linha final `mode: update`.

## Exit codes (004)

| Code | `ErrorKind` | Quando |
|------|-------------|--------|
| 0 | — | Apply (ou dry-run 021) OK |
| 1 | Internal | Bug / estado inconsistente pós-rollback falho |
| 2 | Usage | Args inválidos (clap) |
| 3 | NotFound | Project root / path base ausente |
| 4 | InvalidInput / Config | Path safety; `--target` inválido; config migrate inválida; read cap |
| 5 | Io | I/O / falha de write (após tentativa de rollback) |

Alinhados a microplano 004 / DEC-005 — sem mapa paralelo.

## Diff vs TypeScript 3.18.1

| Item | Classe |
|------|--------|
| Políticas keep / replace / ask + `-y` ≠ `--force` | A |
| Session backup `.dare/backup-<ver>/` (layout TS-compat) | A |
| Exit codes / JSON envelope 004 | B |
| Session backup ≠ `.dare/backups/` (005) como root | B |
| Ask batch único (não N prompts) | B |
| `UpdateApplyReport` schema 1 camelCase | B |
| Releases sem buraco 3.9+ / Codex em plan (021) | C |
| Self-update do binário | fora de escopo (041+) |

## Local verify

```bash
docker compose -f docker-compose.ci.yml config
cargo test -p dare-update
cargo test -p dare-cli --test cli_smoke -- update
```

Compose CI reutilizado (sem imagem nova) — **verificado exit 0** em **mp022-001**. Sem waiver.

## DEC-023

Decisão de produto e contratos: [DEC-023](../DECISION-LOG.md) — apply policies, session backup, migrate no journal, `UpdateApplyReport` schema 1.

## Skill IDE

Slash `/dare-update` e skill `dare-update`: documentar `-y` / `--force` e que apply está implementado no microplano 022. Ver `.claude/commands/dare-update.md` e `.claude/skills/dare-update/SKILL.md`.
