# DARE native installer (alpha) — microplano 015
# Usage:
#   $env:DARE_VERSION='v0.1.0-alpha.1'; .\installers\install.ps1
#   $env:DARE_LOCAL_ARCHIVE='C:\path\dare-....zip'; $env:DARE_PREFIX='C:\tmp\p'; .\installers\install.ps1
# Env: DARE_VERSION | DARE_LOCAL_ARCHIVE (one required);
#      DARE_REPO, DARE_INSTALL_BASE, DARE_PREFIX (optional)

$ErrorActionPreference = 'Stop'
$Repo = if ($env:DARE_REPO) { $env:DARE_REPO } else { 'dewtech/dare-cli' }
$BaseUrl = if ($env:DARE_INSTALL_BASE) { $env:DARE_INSTALL_BASE } else { "https://github.com/$Repo/releases/latest/download" }
$Prefix = if ($env:DARE_PREFIX) { $env:DARE_PREFIX } else { Join-Path $env:LOCALAPPDATA 'dare' }
$BinDir = Join-Path $Prefix 'bin'
$Version = $env:DARE_VERSION

function Get-DareTarget {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch ($arch) {
        'x64' { return 'x86_64-pc-windows-msvc' }
        'arm64' { throw 'arm64 Windows target not in alpha matrix yet' }
        default { throw "unsupported arch: $arch" }
    }
}

$target = Get-DareTarget
$tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetTempPath() + [guid]::NewGuid().ToString())
$archivePath = $null

try {
    if ($env:DARE_LOCAL_ARCHIVE) {
        $archivePath = $env:DARE_LOCAL_ARCHIVE
        if (-not (Test-Path -LiteralPath $archivePath)) { throw "local archive not found: $archivePath" }
        $archiveName = Split-Path $archivePath -Leaf
        Copy-Item -LiteralPath $archivePath -Destination (Join-Path $tmp $archiveName)
        $archivePath = Join-Path $tmp $archiveName
    }
    elseif ($Version) {
        $archiveName = "dare-$Version-$target.zip"
        $url = "$BaseUrl/$archiveName"
        $archivePath = Join-Path $tmp $archiveName
        Invoke-WebRequest -Uri $url -OutFile $archivePath
        try {
            $sums = Join-Path $tmp 'SHA256SUMS'
            Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS" -OutFile $sums
            $expected = (Get-Content $sums | Where-Object { $_ -match [regex]::Escape($archiveName) } | ForEach-Object { ($_ -split '\s+', 2)[0] })
            if ($expected) {
                $actual = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash.ToLowerInvariant()
                if ($actual -ne $expected.ToLowerInvariant()) { throw "SHA256 mismatch for $archiveName" }
            }
        }
        catch {
            Write-Warning "checksum verification skipped: $_"
        }
    }
    else {
        throw 'Set DARE_VERSION=vX.Y.Z-alpha.N or DARE_LOCAL_ARCHIVE=/path/to/archive.zip'
    }

    Expand-Archive -Path $archivePath -DestinationPath $tmp -Force
    $bin = Get-ChildItem -Path $tmp -Recurse -Filter 'dare.exe' | Select-Object -First 1
    if (-not $bin) { throw 'dare.exe not found in archive' }
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item $bin.FullName (Join-Path $BinDir 'dare.exe') -Force
    Write-Host "Installed: $(Join-Path $BinDir 'dare.exe')"
    & (Join-Path $BinDir 'dare.exe') --version
    if ($LASTEXITCODE -ne 0) { throw "dare --version failed with exit $LASTEXITCODE" }
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
