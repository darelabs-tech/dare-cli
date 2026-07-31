<#
.SYNOPSIS
  Shadow pilot runner (055): copy source to temp, fingerprint source, run allowlisted dare argv.

.DESCRIPTION
  Usage:
    pilot-shadow.ps1 --pilot-id <id> --source <path> --dare-bin <path> [--cycle N] [--skip-commands]

  Exit codes: 0 ok · 2 usage · 3 path · 4 policy · 5 IO · 6 compare fail
  Compatible with Windows PowerShell 5.1+ and PowerShell 7+.
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$MSG_WRITE_ORIGINAL = 'shadow must not write to the original pilot tree'
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Exit-WithCode {
    param([int]$Code, [string]$Message)
    if ($Message) { [Console]::Error.WriteLine($Message) }
    exit $Code
}

function Show-Usage {
    Write-Host @'
Usage:
  pilot-shadow.ps1 --pilot-id <id> --source <path> --dare-bin <path> [--cycle N] [--skip-commands]

Exit codes: 2 usage · 3 path · 4 policy · 5 IO · 6 compare fail
'@
}

function Test-AllowlistedArgs {
    param([string[]]$ArgsAfterBin)
    if ($null -eq $ArgsAfterBin -or $ArgsAfterBin.Count -eq 0) { return $false }
    $joined = ($ArgsAfterBin -join ' ').Trim()
    $exact = @(
        '--version',
        '--help',
        'welcome',
        'info',
        'discover',
        'discover --check',
        'validate',
        'update --dry-run',
        'self --help',
        'mcp --help',
        'capabilities'
    )
    if ($exact -contains $joined) { return $true }
    if ($ArgsAfterBin[0] -eq 'harness' -and ($ArgsAfterBin -contains '--help')) { return $true }
    return $false
}

function Get-RelativeFileList {
    param([string]$Root)
    $rootNorm = $Root.TrimEnd('\', '/')
    Get-ChildItem -LiteralPath $Root -Recurse -File -Force |
        ForEach-Object {
            $full = $_.FullName
            $rel = $full.Substring($rootNorm.Length).TrimStart('\', '/')
            $rel.Replace('\', '/')
        } |
        Sort-Object
}

function Get-FingerprintSample {
    param([string]$Root, [int]$MinCount = 3)
    $rels = @(Get-RelativeFileList -Root $Root)
    if ($rels.Count -lt $MinCount) {
        Exit-WithCode 3 "path: source must contain at least $MinCount files for fingerprint (found $($rels.Count))"
    }
    $take = [Math]::Max($MinCount, [Math]::Min(8, $rels.Count))
    $sample = @($rels | Select-Object -First $take)
    $map = [ordered]@{}
    foreach ($rel in $sample) {
        $abs = Join-Path $Root ($rel -replace '/', [IO.Path]::DirectorySeparatorChar)
        try {
            $hash = (Get-FileHash -LiteralPath $abs -Algorithm SHA256).Hash.ToLowerInvariant()
        } catch {
            Exit-WithCode 5 "IO: failed to hash source file: $rel"
        }
        $map[$rel] = $hash
    }
    return $map
}

function Assert-Fingerprints {
    param([string]$Root, $Before)
    foreach ($rel in $Before.Keys) {
        $abs = Join-Path $Root ($rel -replace '/', [IO.Path]::DirectorySeparatorChar)
        if (-not (Test-Path -LiteralPath $abs -PathType Leaf)) {
            [Console]::Error.WriteLine($MSG_WRITE_ORIGINAL)
            exit 4
        }
        try {
            $hash = (Get-FileHash -LiteralPath $abs -Algorithm SHA256).Hash.ToLowerInvariant()
        } catch {
            Exit-WithCode 5 "IO: failed to re-hash source file: $rel"
        }
        if ($hash -ne $Before[$rel]) {
            [Console]::Error.WriteLine($MSG_WRITE_ORIGINAL)
            exit 4
        }
    }
}

function Protect-Redact {
    param([string]$Text)
    if ([string]::IsNullOrEmpty($Text)) { return $Text }
    $out = $Text
    $userHome = [Environment]::GetFolderPath('UserProfile')
    if ($userHome) {
        $out = $out.Replace($userHome, '$HOME')
        $out = $out.Replace($userHome.Replace('\', '/'), '$HOME')
    }
    if ($env:TEMP) {
        $out = $out.Replace($env:TEMP, '$TMP')
        $out = $out.Replace($env:TEMP.Replace('\', '/'), '$TMP')
    }
    $out = [regex]::Replace($out, '(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*\S+', '$1=***')
    return $out
}

function Resolve-NextCycle {
    param([string]$ResultsDir, $Requested = $null)
    if ($null -ne $Requested) { return [int]$Requested }
    $n = 1
    while (Test-Path -LiteralPath (Join-Path $ResultsDir "cycle-$n.md")) { $n++ }
    return $n
}

function Write-Utf8NoBom {
    param([string]$Path, [string]$Content)
    $enc = New-Object System.Text.UTF8Encoding $false
    [IO.File]::WriteAllText($Path, $Content, $enc)
}

# --- parse argv (named flags; no positional shell concat) ---
$PilotId = $null
$Source = $null
$DareBin = $null
$CycleOpt = $null
$SkipCommands = $false

$i = 0
$argv = @($args)
while ($i -lt $argv.Count) {
    switch ($argv[$i]) {
        '--pilot-id' {
            if ($i + 1 -ge $argv.Count) { Show-Usage; Exit-WithCode 2 'usage: --pilot-id requires a value' }
            $PilotId = $argv[$i + 1]; $i += 2
        }
        '--source' {
            if ($i + 1 -ge $argv.Count) { Show-Usage; Exit-WithCode 2 'usage: --source requires a value' }
            $Source = $argv[$i + 1]; $i += 2
        }
        '--dare-bin' {
            if ($i + 1 -ge $argv.Count) { Show-Usage; Exit-WithCode 2 'usage: --dare-bin requires a value' }
            $DareBin = $argv[$i + 1]; $i += 2
        }
        '--cycle' {
            if ($i + 1 -ge $argv.Count) { Show-Usage; Exit-WithCode 2 'usage: --cycle requires a value' }
            $CycleOpt = [int]$argv[$i + 1]; $i += 2
        }
        '--skip-commands' {
            $SkipCommands = $true; $i += 1
        }
        '--help' {
            Show-Usage; exit 0
        }
        default {
            Show-Usage
            Exit-WithCode 2 "usage: unknown argument $($argv[$i])"
        }
    }
}

if (-not $PilotId -or -not $Source -or -not $DareBin) {
    Show-Usage
    Exit-WithCode 2 'usage: --pilot-id, --source, and --dare-bin are required'
}

if ($PilotId -notmatch '^[a-z0-9]+(-[a-z0-9]+)*$') {
    Exit-WithCode 2 'usage: --pilot-id must match ^[a-z0-9]+(-[a-z0-9]+)*$'
}

function Resolve-FullPath {
    param([string]$PathValue)
    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        throw 'empty path'
    }
    if ([IO.Path]::IsPathRooted($PathValue)) {
        return [IO.Path]::GetFullPath($PathValue)
    }
    return [IO.Path]::GetFullPath((Join-Path (Get-Location).Path $PathValue))
}

$SourceFull = Resolve-FullPath -PathValue $Source
if (-not (Test-Path -LiteralPath $SourceFull -PathType Container)) {
    Exit-WithCode 3 "path: source is not a directory: $Source"
}

$DareBinFull = Resolve-FullPath -PathValue $DareBin
if (-not (Test-Path -LiteralPath $DareBinFull -PathType Leaf)) {
    Exit-WithCode 3 "path: dare-bin not found: $DareBin"
}

$uuid = [guid]::NewGuid().ToString('N')
$ShadowRoot = Join-Path $env:TEMP "dare-pilot-$PilotId-$uuid"

# Default allowlist smoke commands (each entry is a string[] argv after binary)
$DefaultCommandSets = @(
    [string[]]@('--version'),
    [string[]]@('--help'),
    [string[]]@('info')
)

$before = Get-FingerprintSample -Root $SourceFull -MinCount 3

try {
    New-Item -ItemType Directory -Path $ShadowRoot -Force | Out-Null
    Copy-Item -Path (Join-Path $SourceFull '*') -Destination $ShadowRoot -Recurse -Force
} catch {
    Exit-WithCode 5 "IO: failed to copy source to shadow root: $($_.Exception.Message)"
}

$commandRows = New-Object System.Collections.Generic.List[string]
$verdict = 'pass'
$compareFailed = $false

if (-not $SkipCommands) {
    foreach ($cmdArgs in $DefaultCommandSets) {
        $argArr = [string[]]$cmdArgs
        if (-not (Test-AllowlistedArgs -ArgsAfterBin $argArr)) {
            Exit-WithCode 4 "policy: command not on allowlist: $($argArr -join ' ')"
        }
        $argvLine = ($argArr -join ' ')
        $exitCode = 0
        $stdout = ''
        $stderr = ''
        try {
            # Argv-only: call operator + splat (no Invoke-Expression / shell string concat)
            Push-Location -LiteralPath $ShadowRoot
            try {
                $prevEap = $ErrorActionPreference
                $ErrorActionPreference = 'Continue'
                $combined = & $DareBinFull @argArr 2>&1
                $exitCode = $LASTEXITCODE
                if ($null -eq $exitCode) { $exitCode = 0 }
                $ErrorActionPreference = $prevEap
                $stdout = [string](($combined | Where-Object { $_ -isnot [System.Management.Automation.ErrorRecord] }) -join "`n")
                $stderr = [string](($combined | Where-Object { $_ -is [System.Management.Automation.ErrorRecord] }) -join "`n")
            } finally {
                Pop-Location
            }
        } catch {
            $exitCode = 1
            $stderr = $_.Exception.Message
        }
        if ($null -eq $stdout) { $stdout = '' }
        if ($null -eq $stderr) { $stderr = '' }
        $note = Protect-Redact ("stdout_len={0}; stderr_len={1}" -f $stdout.Length, $stderr.Length)
        [void]$commandRows.Add("| ``dare $argvLine`` | $exitCode | $note |")
        if ($exitCode -ne 0) {
            $compareFailed = $true
            $verdict = 'fail'
        }
    }
} else {
    [void]$commandRows.Add('| _(skipped)_ | 0 | --skip-commands |')
}

Assert-Fingerprints -Root $SourceFull -Before $before

$resultsDir = Join-Path $RepoRoot "docs\pilot\results\$PilotId"
try {
    New-Item -ItemType Directory -Path $resultsDir -Force | Out-Null
} catch {
    Exit-WithCode 5 "IO: cannot create results dir: $resultsDir"
}

if ($null -ne $CycleOpt) {
    $cycle = [int]$CycleOpt
} else {
    $cycle = Resolve-NextCycle -ResultsDir $resultsDir -Requested $null
}
$reportPath = Join-Path $resultsDir "cycle-$cycle.md"
$fpLines = @($before.GetEnumerator() | ForEach-Object { "- ``$($_.Key)``: ``$($_.Value.Substring(0,12))...``" }) -join "`n"
$shadowHint = Protect-Redact $ShadowRoot
$cmdTable = ($commandRows -join "`n")
$integrity = 'pass'
$body = @"
# Shadow cycle $cycle - $PilotId

| Field | Value |
|-------|-------|
| pilot_id | ``$PilotId`` |
| cycle | $cycle |
| shadow_root | ``$shadowHint`` (redacted) |
| source_integrity | ``$integrity`` |
| verdict | ``$verdict`` |

## Commands

| argv | exit | notes |
|------|------|-------|
$cmdTable

## Source fingerprint sample (>=3)

$fpLines

## Notes

- Copy-only shadow; original source verified unchanged (``MSG_WRITE_ORIGINAL`` gate).
- Allowlist argv only; no shell string concatenation.
"@

try {
    Write-Utf8NoBom -Path $reportPath -Content $body
} catch {
    Exit-WithCode 5 "IO: failed to write report: $reportPath"
}

Write-Host "shadow OK: pilot=$PilotId cycle=$cycle shadow=$ShadowRoot report=$reportPath integrity=pass"

if ($compareFailed) {
    exit 6
}
exit 0
