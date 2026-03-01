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

function Resolve-PathsFromSliceKey([string]$Key) {
  $parts = @{}
  foreach ($segment in ($Key -split '\|')) {
    $kv = $segment -split '=', 2
    if ($kv.Count -eq 2) {
      $parts[$kv[0]] = $kv[1]
    }
  }

  foreach ($required in @('sex', 'equip', 'wc', 'age', 'tested', 'lift')) {
    if (-not $parts.ContainsKey($required)) {
      return $null
    }
  }

  $lift = switch ($parts['lift']) {
    'S' { 'squat' }
    'B' { 'bench' }
    'D' { 'deadlift' }
    'T' { 'total' }
    default { $null }
  }
  if (-not $lift) { return $null }

  $slug = {
    param([string]$s)
    return (($s.ToLowerInvariant()) -replace '[^a-z0-9-]', '_')
  }

  $testedDir = if ($parts['tested'].Equals('Yes', [System.StringComparison]::OrdinalIgnoreCase)) { 'tested' } else { & $slug $parts['tested'] }
  $base = "{0}/{1}/{2}/{3}/{4}/{5}" -f (& $slug $parts['sex']), (& $slug $parts['equip']), (& $slug $parts['wc']), (& $slug $parts['age']), $testedDir, $lift

  return [PSCustomObject]@{
    Meta = "meta/$base.json"
    Hist = "hist/$base.bin"
    Heat = "heat/$base.bin"
  }
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
$indexBudgetBytes = $indexRootBytes
$sliceEntries = [System.Collections.Generic.List[object]]::new()
$shardSizeByRel = @{}

$appendSlices = {
  param(
    [object]$sliceNode,
    [string]$shardRel
  )

  if ($null -eq $sliceNode) { return }

  if ($sliceNode -is [System.Array]) {
    foreach ($k in @($sliceNode)) {
      $key = [string]$k
      $paths = Resolve-PathsFromSliceKey $key
      if ($null -eq $paths) { Fail "invalid compact slice key: $key" }
      $sliceEntries.Add([PSCustomObject]@{
        Key = $key
        Meta = [string]$paths.Meta
        Hist = [string]$paths.Hist
        Heat = [string]$paths.Heat
        ShardRel = $shardRel
      })
    }
    return
  }

  foreach ($p in @($sliceNode.PSObject.Properties)) {
    $sliceEntries.Add([PSCustomObject]@{
      Key = [string]$p.Name
      Meta = [string]$p.Value.meta
      Hist = [string]$p.Value.hist
      Heat = [string]$p.Value.heat
      ShardRel = $shardRel
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
    $shardSize = (Get-Item $shardPath).Length
    $indexBudgetBytes += $shardSize
    $shardSizeByRel[$rel] = $shardSize
    $shard = Get-Content $shardPath -Raw | ConvertFrom-Json
    & $appendSlices $shard.slices $rel
  }
}

if ($sliceEntries.Count -eq 0) { Fail "index has no slice entries" }

Write-Host "[qa] Version: $version"
Write-Host "[qa] Slice entries: $($sliceEntries.Count)"

$missing = 0
$invalid = 0
$histTotalSum = 0

foreach ($entry in $sliceEntries) {
  $key = $entry.Key

  foreach ($rel in @($entry.Meta, $entry.Hist, $entry.Heat)) {
    if ([string]::IsNullOrWhiteSpace($rel)) {
      Write-Host "[qa] invalid empty path in index ($key)" -ForegroundColor Yellow
      $invalid++
      continue
    }

    if ($rel.StartsWith('/')) {
      Write-Host "[qa] invalid absolute path in index ($key): $rel" -ForegroundColor Yellow
      $invalid++
      continue
    }

    $full = Join-Path $versionDir $rel
    if (-not (Test-Path $full)) {
      Write-Host "[qa] missing file for ${key}: $rel" -ForegroundColor Yellow
      $missing++
      continue
    }

    $len = (Get-Item $full).Length
    if ($len -le 0) {
      Write-Host "[qa] empty file for ${key}: $rel" -ForegroundColor Yellow
      $missing++
    }
  }

  $metaPath = Join-Path $versionDir $entry.Meta
  if (Test-Path $metaPath) {
    $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
    $bins = [int]($meta.hist.bins)
    $total = [int64]($meta.hist.total)
    $w = [int]($meta.heat.width)
    $h = [int]($meta.heat.height)

    if ($bins -lt 1) {
      Write-Host "[qa] bad histogram bins for ${key}: $bins" -ForegroundColor Yellow
      $invalid++
    }

    if ($total -lt 0) {
      Write-Host "[qa] bad hist.total for ${key}: $total" -ForegroundColor Yellow
      $invalid++
    } else {
      $histTotalSum += $total
    }

    if ($w -eq 0 -or $h -eq 0) {
      Write-Host "[qa] warning: zero-dimension heatmap for $key (${w}x${h})"
    }
  }
}

if ($missing -gt 0) { Fail "found $missing missing/empty referenced files" }
if ($invalid -gt 0) { Fail "found $invalid invalid slice entries" }
if ($histTotalSum -le 0) { Fail "aggregate hist.total is zero" }

$allFiles = Get-ChildItem $versionDir -Recurse -File | Where-Object { $_.Name -match '\.(bin|json)$' }
$binBytes = ($allFiles | Where-Object { $_.Extension -eq '.bin' } | Measure-Object -Property Length -Sum).Sum
$jsonBytes = ($allFiles | Where-Object { $_.Extension -eq '.json' } | Measure-Object -Property Length -Sum).Sum
if (-not $binBytes) { $binBytes = 0 }
if (-not $jsonBytes) { $jsonBytes = 0 }
$totalBytes = $binBytes + $jsonBytes

Write-Host "[qa] Aggregate hist.total sum: $histTotalSum"
Write-Host "[qa] Files checked: $($allFiles.Count)"
Write-Host "[qa] Data payload: total=$(Format-Bytes $totalBytes) (bin=$(Format-Bytes $binBytes), json=$(Format-Bytes $jsonBytes))"

$selectedProp = $null
if ($SliceKey) {
  $selectedProp = $sliceEntries | Where-Object { $_.Key -eq $SliceKey } | Select-Object -First 1
  if (-not $selectedProp) {
    Write-Host "[qa] warning: requested SliceKey not found, using first slice."
  }
}
if (-not $selectedProp) {
  $selectedProp = $sliceEntries | Where-Object { $_.Key -like 'sex=F|equip=All|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
}
if (-not $selectedProp) {
  $selectedProp = $sliceEntries | Where-Object { $_.Key -like 'sex=F|equip=Raw|wc=*|age=24-34|tested=All|lift=B' } | Select-Object -First 1
}
if (-not $selectedProp) {
  $selectedProp = $sliceEntries | Select-Object -First 1
}
$selectedEntry = $selectedProp
$selectedName = $selectedProp.Key

$sampleIndexBytes = $indexRootBytes
$sampleShardRel = $null
if ($isSharded) {
  $sampleShardRel = [string]$selectedEntry.ShardRel
  if ([string]::IsNullOrWhiteSpace($sampleShardRel)) {
    Fail "failed to resolve shard index for sample slice: $selectedName"
  }
  $sampleIndexBytes += [int64]($shardSizeByRel[$sampleShardRel])
}

$sampleMetaPath = Join-Path $versionDir $selectedEntry.Meta
$sampleHistPath = Join-Path $versionDir $selectedEntry.Hist
$sampleHeatPath = Join-Path $versionDir $selectedEntry.Heat
$sampleMetaBytes = if (Test-Path $sampleMetaPath) { (Get-Item $sampleMetaPath).Length } else { 0 }
$sampleHistBytes = if (Test-Path $sampleHistPath) { (Get-Item $sampleHistPath).Length } else { 0 }
$sampleHeatBytes = if (Test-Path $sampleHeatPath) { (Get-Item $sampleHeatPath).Length } else { 0 }
$latestBytes = (Get-Item $latestPath).Length
$sampleDataBytes = $latestBytes + $sampleIndexBytes + $sampleMetaBytes + $sampleHistBytes + $sampleHeatBytes

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
  Write-Host "[qa] Sample data request budget: $(Format-Bytes $sampleDataBytes) (latest+index_root+index_shard+meta+hist+heat)"
} else {
  Write-Host "[qa] Sample data request budget: $(Format-Bytes $sampleDataBytes) (latest+index+meta+hist+heat)"
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
  $probeItems.Add([PSCustomObject]@{ Label = "f_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $selectedEntry.Meta.Replace('\', '/'))) })
  $probeItems.Add([PSCustomObject]@{ Label = "f_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $selectedEntry.Hist.Replace('\', '/'))) })
  $probeItems.Add([PSCustomObject]@{ Label = "f_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $selectedEntry.Heat.Replace('\', '/'))) })

  if ($maleProbe -and ($maleProbe.Key -ne $selectedEntry.Key)) {
    Write-Host "[qa] Probe sample (M/All): $($maleProbe.Key)"
    if ($isSharded -and -not [string]::IsNullOrWhiteSpace([string]$maleProbe.ShardRel)) {
      $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.ShardRel.Replace('\', '/'))) })
    }
    $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.Meta.Replace('\', '/'))) })
    $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.Hist.Replace('\', '/'))) })
    $probeItems.Add([PSCustomObject]@{ Label = "m_all"; Url = (Join-Url $BaseUrl ("data/$version/" + $maleProbe.Heat.Replace('\', '/'))) })
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
