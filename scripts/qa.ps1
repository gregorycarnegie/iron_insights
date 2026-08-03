param(
  [string]$DataDir = "data",
  [string]$SiteDir = "docs",
  [string]$BaseUrl = "",
  [string]$SliceKey = ""
)

$ErrorActionPreference = 'Stop'

function Fail([string]$Message) {
  Write-Error "[qa] ERROR: $Message"
  exit 1
}

function Format-Bytes([Int64]$Bytes) {
  if ($Bytes -ge 1GB) { return "{0:N2} GB" -f ($Bytes / 1GB) }
  if ($Bytes -ge 1MB) { return "{0:N2} MB" -f ($Bytes / 1MB) }
  if ($Bytes -ge 1KB) { return "{0:N2} KB" -f ($Bytes / 1KB) }
  return "$Bytes B"
}

function Join-Url([string]$Left, [string]$Right) {
  $l = $Left.TrimEnd('/')
  $r = $Right.TrimStart('./').TrimStart('/')
  return "$l/$r"
}

if (-not (Test-Path $DataDir)) { Fail "data directory not found: $DataDir" }

$latestPath = Join-Path $DataDir 'latest.json'
if (-not (Test-Path $latestPath)) { Fail "missing latest.json: $latestPath" }

$latest = Get-Content $latestPath -Raw | ConvertFrom-Json
if (-not $latest.version) { Fail "latest.json missing .version" }

$version = $latest.version
$versionDir = Join-Path $DataDir $version
if (-not (Test-Path $versionDir)) { Fail "version directory missing: $versionDir" }

$indexPath = Join-Path $versionDir 'index.json'
if (-not (Test-Path $indexPath)) { Fail "missing index.json: $indexPath" }

$index = Get-Content $indexPath -Raw | ConvertFrom-Json
if (-not $index.slices -and -not $index.shards) { Fail "index.json missing .slices or .shards" }

$isSharded = [bool]$index.shards
$indexRootBytes = (Get-Item $indexPath).Length
$sliceEntries = [System.Collections.Generic.List[object]]::new()
$shardSizeByRel = @{}

$appendSlices = {
  param(
    [object]$sliceNode,
    [string]$shardRel
  )

  if ($null -eq $sliceNode) { return }

  foreach ($p in @($sliceNode.PSObject.Properties)) {
    $sliceEntries.Add([PSCustomObject]@{
      Key = [string]$p.Name
      # Empty when the payload is inlined into the shard as base64.
      Bin = [string]$p.Value.bin
      ShardRel = $shardRel
      SummaryTotal = [int64]$p.Value.summary.total
    })
  }
}

if ($index.slices) {
  & $appendSlices $index.slices "index.json"
} else {
  foreach ($sp in @($index.shards.PSObject.Properties)) {
    $rel = [string]$sp.Value
    if ([string]::IsNullOrWhiteSpace($rel)) { Fail "empty shard path for $($sp.Name)" }
    if ($rel.StartsWith('/')) { Fail "invalid absolute shard path for $($sp.Name): $rel" }
    $shardPath = Join-Path $versionDir $rel
    if (-not (Test-Path $shardPath)) { Fail "missing shard file for $($sp.Name): $rel" }
    $shardSizeByRel[$rel] = (Get-Item $shardPath).Length
    $shard = Get-Content $shardPath -Raw | ConvertFrom-Json
    & $appendSlices $shard.slices $rel
  }
}

if ($sliceEntries.Count -eq 0) { Fail "index has no slice entries" }

Write-Host "[qa] Version: $version"
Write-Host "[qa] Slice entries: $($sliceEntries.Count)"

$missing = 0
$invalid = 0
$summaryTotalSum = [int64]0

foreach ($entry in $sliceEntries) {
  $key = $entry.Key
  $rel = $entry.Bin

  if (-not [string]::IsNullOrWhiteSpace($rel)) {
    if ($rel.StartsWith('/')) {
      Write-Host "[qa] invalid absolute path in index ($key): $rel" -ForegroundColor Yellow
      $invalid++
    } else {
      $full = Join-Path $versionDir $rel
      if (-not (Test-Path $full)) {
        Write-Host "[qa] missing file for ${key}: $rel" -ForegroundColor Yellow
        $missing++
      } elseif ((Get-Item $full).Length -le 0) {
        Write-Host "[qa] empty file for ${key}: $rel" -ForegroundColor Yellow
        $missing++
      }
    }
  }

  $summaryTotalSum += [int64]$entry.SummaryTotal
}

if ($missing -gt 0) { Fail "found $missing missing/empty referenced files" }
if ($invalid -gt 0) { Fail "found $invalid invalid slice entries" }
if ($summaryTotalSum -le 0) { Fail "aggregate summary.total is zero" }

$allFiles = Get-ChildItem $versionDir -Recurse -File | Where-Object { $_.Name -match '\.(bin|json)$' }
$binBytes = ($allFiles | Where-Object { $_.Extension -eq '.bin' } | Measure-Object -Property Length -Sum).Sum
$jsonBytes = ($allFiles | Where-Object { $_.Extension -eq '.json' } | Measure-Object -Property Length -Sum).Sum
if (-not $binBytes) { $binBytes = 0 }
if (-not $jsonBytes) { $jsonBytes = 0 }
$totalBytes = $binBytes + $jsonBytes

Write-Host "[qa] Aggregate summary.total sum: $summaryTotalSum"
Write-Host "[qa] Files checked: $($allFiles.Count)"
Write-Host "[qa] Data payload: total=$(Format-Bytes $totalBytes) (bin=$(Format-Bytes $binBytes), json=$(Format-Bytes $jsonBytes))"

$selectedEntry = $null
if ($SliceKey) {
  $selectedEntry = $sliceEntries | Where-Object { $_.Key -eq $SliceKey } | Select-Object -First 1
  if (-not $selectedEntry) {
    Write-Host "[qa] warning: requested SliceKey not found, using first slice."
  }
}
if (-not $selectedEntry) {
  $selectedEntry = $sliceEntries | Where-Object { $_.Key -like 'sex=F|equip=All|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
}
if (-not $selectedEntry) {
  $selectedEntry = $sliceEntries | Where-Object { $_.Key -like 'sex=F|equip=Raw|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
}
if (-not $selectedEntry) {
  $selectedEntry = $sliceEntries | Select-Object -First 1
}
$selectedName = $selectedEntry.Key

$sampleIndexBytes = $indexRootBytes
$sampleShardRel = $null
if ($isSharded) {
  $sampleShardRel = [string]$selectedEntry.ShardRel
  if ([string]::IsNullOrWhiteSpace($sampleShardRel)) {
    Fail "failed to resolve shard index for sample slice: $selectedName"
  }
  $sampleIndexBytes += [int64]($shardSizeByRel[$sampleShardRel])
}

# Inlined slice: the payload is already counted inside the index shard.
$sampleBinBytes = 0
if (-not [string]::IsNullOrWhiteSpace($selectedEntry.Bin)) {
  $sampleBinPath = Join-Path $versionDir $selectedEntry.Bin
  if (Test-Path $sampleBinPath) { $sampleBinBytes = (Get-Item $sampleBinPath).Length }
}
$latestBytes = (Get-Item $latestPath).Length
$sampleDataBytes = $latestBytes + $sampleIndexBytes + $sampleBinBytes

$maleProbe = $sliceEntries | Where-Object { $_.Key -like 'sex=M|equip=All|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
if (-not $maleProbe) {
  $maleProbe = $sliceEntries | Where-Object { $_.Key -like 'sex=M|equip=Raw|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
}
if (-not $maleProbe) {
  $maleProbe = $sliceEntries | Where-Object { $_.Key -like 'sex=M|equip=All|*' } | Select-Object -First 1
}
if (-not $maleProbe) {
  $maleProbe = $sliceEntries | Where-Object { $_.Key -like 'sex=M|equip=Raw|*' } | Select-Object -First 1
}

Write-Host "[qa] Sample slice: $selectedName"
if ($isSharded) {
  Write-Host "[qa] Sample data request budget: $(Format-Bytes $sampleDataBytes) (latest+index_root+index_shard+bin)"
} else {
  Write-Host "[qa] Sample data request budget: $(Format-Bytes $sampleDataBytes) (latest+index+bin)"
}

$siteBudgetBytes = 0
if (Test-Path $SiteDir) {
  $siteFiles = Get-ChildItem $SiteDir -File -Recurse | Where-Object {
    $_.Extension -in @('.html', '.css', '.js', '.wasm')
  }
  $siteBudgetBytes = ($siteFiles | Measure-Object -Property Length -Sum).Sum
  if (-not $siteBudgetBytes) { $siteBudgetBytes = 0 }
  Write-Host "[qa] Site static payload (.html/.css/.js/.wasm): $(Format-Bytes $siteBudgetBytes)"
} else {
  Write-Host "[qa] SiteDir not found ($SiteDir), skipping static payload summary."
}

$firstViewBudget = $siteBudgetBytes + $sampleDataBytes
if ($firstViewBudget -gt 0) {
  Write-Host "[qa] Estimated first-view payload: $(Format-Bytes $firstViewBudget)"
}

if (-not [string]::IsNullOrWhiteSpace($BaseUrl)) {
  Write-Host "[qa] URL timing probe:"
  $probeItems = [System.Collections.Generic.List[object]]::new()
  $probeItems.Add([PSCustomObject]@{ Label = "base"; Url = (Join-Url $BaseUrl "data/latest.json") })
  $probeItems.Add([PSCustomObject]@{ Label = "base"; Url = (Join-Url $BaseUrl ("data/$version/index.json")) })
  if ($isSharded -and -not [string]::IsNullOrWhiteSpace($sampleShardRel)) {
    $probeItems.Add([PSCustomObject]@{ Label = "f_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $sampleShardRel.Replace('\', '/'))) })
  }
  if (-not [string]::IsNullOrWhiteSpace($selectedEntry.Bin)) {
    $probeItems.Add([PSCustomObject]@{ Label = "f_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $selectedEntry.Bin.Replace('\', '/'))) })
  }

  if ($maleProbe -and ($maleProbe.Key -ne $selectedEntry.Key)) {
    Write-Host "[qa] Probe sample (M/All): $($maleProbe.Key)"
    if ($isSharded -and -not [string]::IsNullOrWhiteSpace([string]$maleProbe.ShardRel)) {
      $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.ShardRel.Replace('\', '/'))) })
    }
    if (-not [string]::IsNullOrWhiteSpace([string]$maleProbe.Bin)) {
      $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.Bin.Replace('\', '/'))) })
    }
  }

  foreach ($item in $probeItems) {
    $u = [string]$item.Url
    $label = [string]$item.Label
    try {
      $sw = [System.Diagnostics.Stopwatch]::StartNew()
      $resp = Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 30
      $sw.Stop()
      $len = if ($resp.RawContentLength -gt 0) { $resp.RawContentLength } else { 0 }
      Write-Host ("[qa]  [{0}] {1,4}  {2,6} ms  {3,10}  {4}" -f $label, $resp.StatusCode, [int]$sw.Elapsed.TotalMilliseconds, (Format-Bytes $len), $u)
    } catch {
      Write-Host "[qa]  [$label] FAIL        --       --  $u" -ForegroundColor Yellow
      Write-Host "[qa]    $($_.Exception.Message)" -ForegroundColor Yellow
    }
  }
}

Write-Host "[qa] OK"
