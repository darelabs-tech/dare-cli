# Assets inventory & embed (`dare-assets`)

Microplano **009**.

## Layout

- `assets/manifest.yml` — versão 1, entries com `id`, `path`, `sha256`, `kind`
- `assets/templates/*` — templates DARE canónicos (cópias de `templates/`)

## API

- `verify_embedded_assets()` — falha se missing/hash mismatch
- `materialize_to(root, dest)` — escreve sob ProjectRoot via `atomic_write`
- Embed: `rust-embed` folder `../../assets`

## DEC-010

Single source sob `assets/`; `.claude/commands` permanece `external`/editável (não apagado neste ciclo).
