param([Parameter(Mandatory=$true)][string]$Bin)
if (-not (Test-Path $Bin)) { throw "missing $Bin" }
$v = & $Bin --version
if ($v -notmatch '^dare \d+\.\d+\.\d+(\.\d+)?(-[a-zA-Z0-9._-]+)?\s*$') { throw "bad version: $v" }
$h = & $Bin --help | Out-String
if ($h -notmatch 'Usage:|--version') { throw "bad help" }
