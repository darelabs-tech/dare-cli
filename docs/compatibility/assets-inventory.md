# Assets inventory & embed (`dare-assets`)

Microplano **009**. Decisão: [DEC-010](../DECISION-LOG.md).

## Layout (SoT)

- `assets/manifest.yml` — versão 1; entries `id`, `path`, `sha256`, `kind`
- `assets/templates/*` — templates DARE **canónicos**
- `assets/capability-matrix.yml` — canonical (validação profunda = microplano 010)
- `templates/` (raiz) — espelho legado Class B; ver `templates/README.md`

## Trade-offs (T-01…T-12)

| # | Escolha |
|---|--------|
| T-01 | SoT = `assets/`; raiz `templates/` = espelho |
| T-03 | `rust-embed` **=8.7.2** |
| T-04 | `external` skip em verify/materialize |
| T-05 | Paths POSIX relativos; sem `..` / `\` |
| T-06 | SHA-256 hex lowercase |
| T-10 | Teste freshness FS vs manifest |
| T-11 | CLI só `dare assets verify` |

## API

```text
assert_safe_asset_path(path) -> CoreResult<()>
sha256_hex / load_manifest_from_str
verify_embedded_assets() -> CoreResult<()>
materialize_to(root, dest_rel) -> CoreResult<usize>
EmbeddedAssets (rust-embed folder ../../assets)
```

- Verify: missing / hash mismatch → `CoreError::Config`
- Materialize: verify first; `atomic_write` sob `ProjectRoot`; skip `external`
- Path invalid → erro **antes** de escrever

## Inventário mínimo

| id | path | kind |
|----|------|------|
| template-design | templates/DESIGN-template.md | canonical |
| template-blueprint | templates/BLUEPRINT-template.md | canonical |
| template-tasks | templates/TASKS-template.md | canonical |
| template-task-spec | templates/TASK-SPEC-template.md | canonical |
| template-telemetry | templates/TELEMETRY-template.md | canonical |
| template-hooks-adapter | templates/HOOKS-ADAPTER.md | canonical |
| capability-matrix | capability-matrix.yml | canonical |

## Paridade TS 3.18.1

| Área | Classe | Nota |
|------|--------|------|
| Templates DARE | A | Em `assets/templates` |
| `.claude/commands` | B | external — não apagar |
| `implementations/**` | C | Fora do embed neste ciclo |
| Harness IDE | C | 011–014 |

## Scripts

```bash
python scripts/regen-assets-manifest.py   # recalcula sha256
pwsh scripts/sync-templates-from-assets.ps1  # espelho → templates/
```

## Segurança (RS-01…RS-10)

| RS | Controlo |
|----|----------|
| RS-01 | `assert_safe_asset_path` |
| RS-02 | Sem secrets em assets |
| RS-03 | `ProjectRoot` + `atomic_write` |
| RS-04 | audit + deny |
| RS-07 | hash mismatch = hard fail |
| RS-08 | external não materializado |
| RS-09 | hash in-process (`sha2`) |

## Ver também

- [`disk-and-json-policy.md`](disk-and-json-policy.md)
- [`path-safety.md`](path-safety.md)
