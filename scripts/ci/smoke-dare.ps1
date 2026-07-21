param([Parameter(Mandatory=$true)][string]$Bin)
if (-not (Test-Path $Bin)) { throw "missing $Bin" }
$v = & $Bin --version
if ($v -notmatch '^dare 0\.1\.0-alpha\.0\s*$') { throw "bad version: $v" }
$h = & $Bin --help | Out-String
if ($h -notmatch 'Usage:|--version') { throw "bad help" }
