<#
.SYNOPSIS
  Runs the full publish pipeline. Version defaults to today (vYYYY-MM-DD).
.EXAMPLE
  pwsh -File .\scripts\refresh.ps1
.EXAMPLE
  pwsh -File .\scripts\refresh.ps1 -Version v2026-07-30
#>
param(
  [string]$Version = "v$(Get-Date -Format 'yyyy-MM-dd')"
)

$ErrorActionPreference = 'Stop'

if ($Version -notmatch '^v\d{4}-\d{2}-\d{2}$') {
  Write-Host "[refresh] ERROR: version must look like vYYYY-MM-DD, got: $Version"
  exit 1
}

Set-Location (Join-Path $PSScriptRoot '..')
$manifest = 'iron_insights_pipeline/Cargo.toml'

function Invoke-Stage([string]$Label, [string[]]$CargoArgs) {
  Write-Host "[refresh] $Label"
  cargo run --release --manifest-path $manifest --bin @CargoArgs
  if ($LASTEXITCODE -ne 0) {
    Write-Host "[refresh] ERROR: $Label failed (exit $LASTEXITCODE)"
    exit $LASTEXITCODE
  }
}

Invoke-Stage "01_download ($Version)" @('01_download', '--', '--dataset-version', $Version)
Invoke-Stage '02_build_aggregates'    @('02_build_aggregates')
Invoke-Stage '03_publish_data'        @('03_publish_data', '--', '--data-dir', 'data', '--version', $Version, '--keep-versions', '2')
Invoke-Stage '04_seo_geo'             @('04_seo_geo', '--', '--data-dir', 'data', '--web-dir', 'iron_insights_web')

Write-Host "[refresh] published $Version - verify with pwsh -File .\scripts\qa.ps1 -DataDir data -SiteDir docs"
