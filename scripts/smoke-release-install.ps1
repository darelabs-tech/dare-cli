# Local smoke (Windows): package release binary + checksums + SBOM + install.ps1
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
$Version = if ($env:DARE_SMOKE_VERSION) { $env:DARE_SMOKE_VERSION } else { 'v0.1.0-alpha.smoke' }
$Out = Join-Path $Root 'dist\smoke'
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force -Path $Out | Out-Null

cargo build -p dare-cli --release
$meta = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$targetDir = $meta.target_directory
$hostTriple = (rustc -vV | Select-String '^host:').ToString().Split(':')[1].Trim()
$stage = "dare-$Version-$hostTriple"
$stageDir = Join-Path $Out $stage
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
$binSrc = Join-Path $targetDir 'release\dare.exe'
if (-not (Test-Path $binSrc)) { throw "binary missing: $binSrc" }
Copy-Item $binSrc (Join-Path $stageDir 'dare.exe')
$artifact = "$stage.zip"
Compress-Archive -Path $stageDir -DestinationPath (Join-Path $Out $artifact) -Force

$hash = (Get-FileHash -Algorithm SHA256 -Path (Join-Path $Out $artifact)).Hash.ToLowerInvariant()
Set-Content -Path (Join-Path $Out 'SHA256SUMS') -Value "$hash  $artifact" -Encoding ascii
$created = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
$sbom = @"
{"spdxVersion":"SPDX-2.3","dataLicense":"CC0-1.0","SPDXID":"SPDXRef-DOCUMENT","name":"dare-cli-$Version","documentNamespace":"https://local/spdx/$Version","creationInfo":{"created":"$created","creators":["Tool: dare-smoke"]},"packages":[{"name":"dare","SPDXID":"SPDXRef-Package-dare","downloadLocation":"NOASSERTION","filesAnalyzed":false,"versionInfo":"$Version"}]}
"@
Set-Content -Path (Join-Path $Out 'SBOM.spdx.json') -Value $sbom.Trim() -Encoding utf8
Set-Content -Path (Join-Path $Out 'SHA256SUMS.sig') -Value 'signing skipped — local smoke' -Encoding ascii
Copy-Item (Join-Path $Root 'installers\install.ps1') (Join-Path $Out 'install.ps1')

$prefix = Join-Path $Out 'prefix'
$env:DARE_LOCAL_ARCHIVE = Join-Path $Out $artifact
$env:DARE_PREFIX = $prefix
& (Join-Path $Root 'installers\install.ps1')
$ver = & (Join-Path $prefix 'bin\dare.exe') --version
if ($LASTEXITCODE -ne 0) { throw "dare --version failed" }
if ($ver -notmatch '^dare ') { throw "unexpected version output: $ver" }

$wf = Join-Path $Root '.github\workflows\release.yml'
$txt = Get-Content -Raw $wf
foreach ($t in @(
    'x86_64-unknown-linux-gnu',
    'aarch64-unknown-linux-gnu',
    'x86_64-apple-darwin',
    'aarch64-apple-darwin',
    'x86_64-pc-windows-msvc'
)) {
    if ($txt -notmatch [regex]::Escape($t)) { throw "missing target in release.yml: $t" }
}
if ($txt -notmatch 'macos-13') { throw 'macos-13 missing' }
if ($txt -notmatch 'macos-14') { throw 'macos-14 missing' }

Write-Host "smoke-install OK: $artifact"
if (-not (Test-Path (Join-Path $Out 'SHA256SUMS'))) { throw 'SHA256SUMS missing' }
if (-not (Test-Path (Join-Path $Out 'SBOM.spdx.json'))) { throw 'SBOM missing' }
Write-Host 'five-target matrix OK in release.yml'
