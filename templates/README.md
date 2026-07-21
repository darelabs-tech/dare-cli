# Templates (legacy mirror)

**Source of truth:** [`assets/templates/`](../assets/templates/)

This `templates/` directory at the repo root is a **Class B legacy mirror** of the
canonical files under `assets/templates/`. Edit only `assets/templates/`, then
run:

```powershell
pwsh scripts/sync-templates-from-assets.ps1
python scripts/regen-assets-manifest.py
```

Do not treat root `templates/` as the official SoT for the Rust CLI embed.
