# Sync legacy root templates/ from canonical assets/templates/ (DEC-010 / T-02).
# Run from repo root after editing assets/templates:
#   pwsh scripts/sync-templates-from-assets.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$src = Join-Path $root "assets\templates"
$dst = Join-Path $root "templates"
if (-not (Test-Path $src)) { throw "missing $src" }
New-Item -ItemType Directory -Force -Path $dst | Out-Null
Copy-Item -Path (Join-Path $src "*") -Destination $dst -Force
Write-Host "synced $src -> $dst"
