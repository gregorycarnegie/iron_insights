$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
$src = Join-Path $repoRoot 'data'
$dst = Join-Path $PSScriptRoot 'data'

if (-not (Test-Path $src)) {
  throw "Source data directory not found: $src"
}

if (-not (Test-Path $dst)) {
  New-Item -ItemType Directory -Path $dst | Out-Null
}

robocopy $src $dst /MIR /NFL /NDL /NJH /NJS /NC /NS | Out-Null
Write-Host "Synced data -> app/data"
