# Regenerates crates/dare-cli/tests/fixtures/golden/init-npm/ from the published
# npm @dewtech/dare-cli (3.18.1) inside a pinned Linux container. No network.
#
#   pwsh scripts/parity/gen-init-golden.ps1 [-Only slug1,slug2] [-AllowVersion]
#
# Env overrides: DARE_NPM_PKG, DARE_PARITY_IMAGE, DARE_GOLDEN_OUT (see gen-init-golden.sh).
param(
  [string]$Only = '',
  [switch]$AllowVersion
)
$ErrorActionPreference = 'Stop'

$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

$pkg = $env:DARE_NPM_PKG
if (-not $pkg) {
  $npmRoot = (& npm root -g).Trim()
  $pkg = Join-Path $npmRoot '@dewtech\dare-cli'
}
if (-not (Test-Path (Join-Path $pkg 'package.json'))) {
  Write-Error "@dewtech/dare-cli not found at $pkg (set DARE_NPM_PKG)"
  exit 2
}

$image = $env:DARE_PARITY_IMAGE
if (-not $image) {
  $image = 'node:22-bookworm@sha256:0e5f906573693feaa1e21057ebdcfdb5bd5021f050b2dc7c9deceb629c7da2a8'
}

$out = $env:DARE_GOLDEN_OUT
if (-not $out) {
  $out = Join-Path $repo 'crates\dare-cli\tests\fixtures\golden\init-npm'
}
New-Item -ItemType Directory -Force $out | Out-Null

$extra = @()
if ($Only) { $extra += @('--only', $Only) }
if ($AllowVersion) { $extra += '--allow-version' }

$dockerArgs = @(
  'run', '--rm', '--network', 'none',
  '-e', 'HOME=/tmp/home', '-e', "DARE_PARITY_IMAGE=$image",
  '-v', "${pkg}:/opt/dare/node_modules/@dewtech/dare-cli:ro",
  '-v', "$(Join-Path $repo 'scripts\parity'):/parity:ro",
  '-v', "${out}:/out",
  $image,
  'node', '/parity/gen-init-golden.mjs',
  '--pkg', '/opt/dare/node_modules/@dewtech/dare-cli',
  '--out', '/out',
  '--work', '/tmp/dare-init-golden'
) + $extra

& docker @dockerArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
