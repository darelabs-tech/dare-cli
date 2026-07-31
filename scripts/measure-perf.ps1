# Measure dare-cli release startup / RSS / binary size for MP-054.
# CI gate (Fase F/G): measured <= baseline * (1 + PERF_REGRESSION_MAX)
# where PERF_REGRESSION_MAX=0.15 (fail if >15% above committed baseline).
# This script WRITES docs/perf/baseline-054.md; CI compare jobs must not rewrite.
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$BaselinePath = Join-Path $Root 'docs\perf\baseline-054.md'
$StartUpSamples = 5
# PERF_REGRESSION_MAX=0.15 — documented for consumers of this baseline

Write-Host 'Building dare-cli (release)...'
cargo build -p dare-cli --release
if ($LASTEXITCODE -ne 0) { throw 'cargo build -p dare-cli --release failed' }

$meta = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$targetDir = $meta.target_directory
$hostTriple = (rustc -vV | Select-String '^host:').ToString().Split(':')[1].Trim()
$binPath = Join-Path $targetDir 'release\dare.exe'
if (-not (Test-Path $binPath)) {
    $binPath = Join-Path $targetDir 'release\dare'
}
if (-not (Test-Path $binPath)) { throw "binary missing under $targetDir/release" }

function Get-MedianMs([double[]]$Values) {
    $sorted = @($Values | Sort-Object)
    $n = $sorted.Count
    if ($n -eq 0) { throw 'no samples for median' }
    if ($n % 2 -eq 1) {
        return [math]::Round($sorted[[math]::Floor($n / 2)], 0)
    }
    $a = $sorted[($n / 2) - 1]
    $b = $sorted[$n / 2]
    return [math]::Round(($a + $b) / 2.0, 0)
}

$samples = New-Object 'System.Collections.Generic.List[double]'
for ($i = 0; $i -lt $StartUpSamples; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $binPath -ArgumentList '--version' -PassThru -NoNewWindow -Wait
    $sw.Stop()
    if ($p.ExitCode -ne 0) { throw "dare --version exited $($p.ExitCode)" }
    [void]$samples.Add([double]$sw.Elapsed.TotalMilliseconds)
}

# Discard first (cold); median of remaining 4
$warm = @($samples | Select-Object -Skip 1)
$startupMedianMs = [int](Get-MedianMs $warm)

# RSS: sample WorkingSet64 while process is alive
$rssPeakKiB = 0
$measure = Start-Process -FilePath $binPath -ArgumentList '--version' -PassThru -NoNewWindow
$maxWs = [int64]0
while (-not $measure.HasExited) {
    try {
        $gp = Get-Process -Id $measure.Id -ErrorAction SilentlyContinue
        if ($null -ne $gp -and $gp.WorkingSet64 -gt $maxWs) {
            $maxWs = $gp.WorkingSet64
        }
    } catch { }
    Start-Sleep -Milliseconds 5
}
$measure.WaitForExit()
# ExitCode can be $null briefly on some hosts after WaitForExit; coerce safely.
$rssExit = $measure.ExitCode
if ($null -eq $rssExit) { $rssExit = 0 }
if ($rssExit -ne 0) { throw "dare --version (rss sample) exited $rssExit" }
if ($maxWs -le 0) {
    $measure.Refresh()
    $maxWs = [int64]$measure.WorkingSet64
}
$rssPeakKiB = [int][math]::Round($maxWs / 1024.0)

$binInfo = Get-Item -LiteralPath $binPath
$binarySizeBytes = [int64]$binInfo.Length
$binarySha256 = (Get-FileHash -Algorithm SHA256 -Path $binPath).Hash.ToLowerInvariant()
$measuredAt = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
$gitSha = (git -C $Root rev-parse HEAD).Trim()

$body = @"
---
schemaVersion: 1
targetTriple: "$hostTriple"
startupMedianMs: $startupMedianMs
rssPeakKiB: $rssPeakKiB
binarySizeBytes: $binarySizeBytes
binarySha256: "$binarySha256"
measuredAt: "$measuredAt"
gitSha: "$gitSha"
---

# Perf baseline — MP-054

Committed baseline for CI regression gate
(``measured <= baseline * (1 + PERF_REGRESSION_MAX)`` with **PERF_REGRESSION_MAX=0.15**).

Values above are filled by ``scripts/measure-perf.ps1`` / ``scripts/measure-perf.sh``.
Humans commit the first baseline for each ``targetTriple``; CI compare jobs must **not** rewrite this file.

Gate rule: ``measured <= baseline * (1 + PERF_REGRESSION_MAX)`` per present metric
(``startupMedianMs``, ``rssPeakKiB``, ``binarySizeBytes``).

## How to regenerate

From the repo root (Windows):

``````powershell
.\scripts\measure-perf.ps1
``````

On Unix/macOS (CI):

``````bash
bash scripts/measure-perf.sh
``````

Both scripts:

1. ``cargo build -p dare-cli --release``
2. Run ``dare --version`` five times; discard the first cold sample; take the median ms of the remaining four
3. Capture RSS (Unix: ``ps``; Windows: ``WorkingSet64``)
4. Record release binary size and SHA-256
5. Rewrite the YAML front-matter in this file
"@

$dir = Split-Path -Parent $BaselinePath
if (-not (Test-Path $dir)) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
}
$text = ($body -replace "`r`n", "`n")
[System.IO.File]::WriteAllText($BaselinePath, $text)

Write-Host "Wrote $BaselinePath"
Write-Host "targetTriple=$hostTriple startupMedianMs=$startupMedianMs rssPeakKiB=$rssPeakKiB binarySizeBytes=$binarySizeBytes"
Write-Host "binarySha256=$binarySha256"
Write-Host 'Note: PERF_REGRESSION_MAX=0.15 applies in CI gate vs this baseline.'
