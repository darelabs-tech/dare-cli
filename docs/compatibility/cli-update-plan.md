# CLI: `dare update` — planejamento (Ciclo 021)

Planeamento **read-only** de sincronização de artefatos IDE/templates com o estado desejado embutido no CLI. Complementa [DEC-022](../DECISION-LOG.md) e o Blueprint 021. Apply real, backup e migrations → microplano **022**.

## Flags

| Flag | Efeito |
|------|--------|
| `--dry-run` | **Obrigatório** neste ciclo: classifica assets, emite `UpdatePlan`, **zero writes** |
| `--target <harness>` | Filtra por harness id (ver abaixo); **não** aceita semver |
| `-d` / `--dir <path>` | Diretório inicial para walk-up do project root (default: cwd) |
| `--json` | Envelope JSON (004); `data` = `UpdatePlan` schema 1 |
| `--no-color` | Sem ANSI (global 004) |

Sem `--dry-run`: stub Internal exit **1** — apply não implementado até 022.

## `--target` = harness (não versão)

`--target` recebe um **id de harness**, não a versão do CLI/npm.

| Valor CLI | `expanded_ids` (filter `appliesTo`) |
|-----------|-------------------------------------|
| `claude-code` | `["claude-code"]` |
| `cursor` | `["cursor"]` |
| `codex` | `["codex"]` |
| `antigravity` | `["antigravity"]` |
| `hybrid` | `["cursor", "antigravity"]` |
| `claude-hybrid` | `["claude-code", "cursor"]` |

Sem `--target`: todos os assets do manifest V2 entram no plano (incluindo entradas harness-specific).

Valores inválidos (ex.: `3.2.0`, semver npm) → `InvalidInput` exit **4**; mensagem contém `invalid --target harness:`.

## Classificação (`AssetUpdateStatus`)

Algoritmo por path (SHA + managed marker):

| Status JSON | Quando |
|-------------|--------|
| `missing` | Path não existe ou não é ficheiro regular |
| `identical` | SHA-256 actual == `expectedSha256` do V2 (hex lower, 64 chars) |
| `apply` | SHA difere **e** `content_is_managed(bytes)` (1ª linha: `<!-- dare:managed` ou `---`) |
| `customized` | SHA difere **e** conteúdo **não** managed |

`content_is_managed` vive em `dare-harness` (partilhado pelos adapters).

Leitura de ficheiros: `read_limited` (cap 2 MiB, política 007). Directórios nunca contam como presentes.

## `UpdateManifestV2` (schema 2 — embed)

Ficheiro embed: `assets/update-manifest.v2.json` (`UPDATE_MANIFEST_V2_EMBED`).

| Campo | Tipo | Obrigatório | Semântica |
|-------|------|-------------|-----------|
| `schemaVersion` | `u32` | sim | sempre `2` |
| `cliVersion` | `string` | sim | versão do desired state (ex. `0.1.0-alpha.0`) |
| `releases` | `ReleaseEntry[]` | sim | ordenado; série **sem buracos** |
| `assets` | `DesiredAsset[]` | sim | inventário fechado |

`ReleaseEntry`: `{ "version": string, "notes": string }` (`notes` pode ser `""`).

`DesiredAsset`:

| Campo | Tipo | Obrigatório | Semântica |
|-------|------|-------------|-----------|
| `path` | `string` | sim | relativo POSIX; passa path jail |
| `sha256` | `string` | sim | hex lowercase 64 chars |
| `appliesTo` | `string[]` | sim | não vazio; `"*"` e/ou harness ids |
| `kind` | `string` | não | hint: `canonical` \| `harness` \| `template` |
| `source` | `string` | não | path embed para verificação SHA em CI |

**Validação ao load:**

- `schemaVersion != 2` → Config: `unsupported update manifest schemaVersion`
- `assets` vazio, path inválido, sha malformado, `appliesTo` vazio → Config
- **MUST:** ≥1 asset com `appliesTo` contendo `"codex"` e `path == "AGENTS.md"`

Leitor `UpdateManifestV1` (schema 1) permanece em `dare-contracts` para compat/testes; `plan_update` consome **só V2**.

### Releases sem buraco (Classe C)

O manifest TS 3.18.1 omitia entradas `3.9+` na série de releases. O V2 nativo **não** reproduz esse buraco: `releases[]` deve cobrir a série declarada de forma contínua a partir de `0.1.0-alpha.0` (alpha Rust ≠ npm `3.18.1`). Classe **C** — bugfix consciente vs baseline TS.

### Codex (Classe C)

O TS omitia Codex de `appliesTo` em várias policies. O V2 **MUST** incluir paths Codex (`AGENTS.md` com `appliesTo` contendo `"codex"`). Classe **C** — bugfix consciente (DEC-014 / RF-13).

## `UpdatePlan` (schema 1)

| Campo | Tipo | Semântica |
|-------|------|-----------|
| `schemaVersion` | `u32` | sempre `1` |
| `mode` | `string` | sempre `"dry-run"` |
| `projectRoot` | `string` | path absoluto normalizado (`\` → `/`) |
| `target` | `string \| null` | harness id CLI ou `null` (= all) |
| `cliVersion` | `string` | `CARGO_PKG_VERSION` do build `dare-cli` |
| `counts` | `UpdateCounts` | `{ identical, missing, apply, customized }` — coerente com `items` |
| `items` | `UpdateItem[]` | ordenados por `path` asc |

`UpdateItem`:

| Campo | Tipo | Semântica |
|-------|------|-----------|
| `path` | `string` | relativo POSIX |
| `status` | `AssetUpdateStatus` | lowercase JSON |
| `expectedSha256` | `string` | do V2 |
| `actualSha256` | `string \| null` | `null` se `missing` |
| `appliesTo` | `string[]` | cópia do V2 |

Exemplo concreto:

```json
{
  "schemaVersion": 1,
  "mode": "dry-run",
  "projectRoot": "/tmp/proj",
  "target": "codex",
  "cliVersion": "0.1.0-alpha.0",
  "counts": {
    "identical": 0,
    "missing": 1,
    "apply": 0,
    "customized": 0
  },
  "items": [
    {
      "path": "AGENTS.md",
      "status": "missing",
      "expectedSha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "actualSha256": null,
      "appliesTo": ["codex"]
    }
  ]
}
```

JSON envelope (004): `{ "correlation_id", "data": <UpdatePlan>, "ok": true }`. Dry-run OK → `ok: true` mesmo com `customized > 0`.

Saída human (en-US): contagens + paths; secção `customized:` só se `counts.customized > 0`; **sem** corpo de ficheiro nem unified diff.

## Exit codes

| Code | Quando |
|------|--------|
| 0 | `--dry-run` OK (inclui `customized > 0`) |
| 1 | Apply stub (sem `--dry-run`) **ou** Internal |
| 2 | Usage (clap) |
| 3 | NotFound (embed manifest ausente — raro) |
| 4 | InvalidInput (root, `--target`, path jail) **ou** Config (V2 inválido) |
| 5 | Io ao ler ficheiro do project |

## Stub apply → 022

Sem `--dry-run`:

```
dare update apply is not implemented; use --dry-run (see microplano 022)
```

Exit **1** (Internal). Microplano 022 substitui por apply com backup + atomic write.

## Zero writes (dry-run)

`plan_update` e o fluxo CLI com `--dry-run` **não** executam `atomic_write`, backup, nem tocam `.dare/`. Apenas leitura limitada + classificação. Listing de diretórios antes/depois permanece idêntico.

## Mensagens canónicas (en-US)

| Situação | Substring MUST |
|----------|----------------|
| Stub apply | `dare update apply is not implemented; use --dry-run (see microplano 022)` |
| Root missing | `project root not found` |
| Bad target | `invalid --target harness:` |
| Bad V2 schema | `unsupported update manifest schemaVersion` |

## Diff vs TypeScript 3.18.1

| Item | Classe |
|------|--------|
| Status identical/missing/apply/customized | A |
| SHA-256 | A |
| Manifest desired V2 | B |
| Exit codes / JSON 004 | B |
| Apply stub até 022 | B |
| Releases sem buraco 3.9+ | C |
| Codex em `appliesTo` / plano | C |

## Skill IDE

Slash command `/dare-update` e skill `dare-update` devem exemplificar `--target codex` (ou outro harness válido), **nunca** semver tipo `3.2.0`. Ver `.claude/commands/dare-update.md`.

## Local verify

```bash
docker compose -f docker-compose.ci.yml config
cargo test -p dare-update
cargo test -p dare-cli --test cli_smoke -- update
```

Compose CI reutilizado (sem imagem nova) — **verificado exit 0** (mp021-001).
