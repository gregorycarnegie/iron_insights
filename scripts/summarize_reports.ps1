Param(
    [string]$ReportsDir = "tests",
    [string]$OutCsv = "tests/report_summary.csv",
    [string]$OutMd = "tests/report_summary.md"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (!(Test-Path $ReportsDir)) {
    Write-Error "Reports directory not found: $ReportsDir"
}

$files = Get-ChildItem -Path $ReportsDir -Filter *.html -File | Sort-Object Name
if ($files.Count -eq 0) {
    Write-Error "No HTML reports in $ReportsDir"
}

function Extract-Metric {
    Param([string]$content, [string]$id)
    $regex = '"'+$id+'":\{\"id\":\"'+$id+'\".{0,500}?\"numericValue\":([0-9eE\.-]+).{0,200}?\"displayValue\":\"([^\"]+)\"'
    if ($content -match $regex) { ,@($matches[1],$matches[2]) } else { ,@('','') }
}

$rows = @()
foreach ($f in $files) {
    $h = Get-Content -Raw $f.FullName
    $perf = ''
    if ($h -match '"id":"performance".{0,200}?"score":([0-9\.]+)') { $perf = $matches[1] }
    $url = ''; if ($h -match '"finalUrl":"([^"]+)"') { $url = $matches[1] }
    $fetch=''; if ($h -match '"fetchTime":"([^"]+)"') { $fetch = $matches[1] }

    $fcp = Extract-Metric $h 'first-contentful-paint'
    $lcp = Extract-Metric $h 'largest-contentful-paint'
    $si  = Extract-Metric $h 'speed-index'
    $tbt = Extract-Metric $h 'total-blocking-time'
    $cls = Extract-Metric $h 'cumulative-layout-shift'
    $tti = Extract-Metric $h 'interactive'

    $rows += [PSCustomObject]@{
        File = $f.Name
        URL  = $url
        Time = $fetch
        Perf = $perf
        FCP  = $fcp[1]
        LCP  = $lcp[1]
        SI   = $si[1]
        TBT  = $tbt[1]
        CLS  = $cls[1]
        TTI  = $tti[1]
    }
}

# Write CSV
$rows | Export-Csv -Path $OutCsv -NoTypeInformation -Encoding UTF8

# Write Markdown
$md = @()
$md += "| File | URL | Time | Perf | FCP | LCP | SI | TBT | CLS | TTI |"
$md += "|------|-----|------|------|-----|-----|----|-----|-----|-----|"
foreach ($r in $rows) {
    $md += "| $($r.File) | $($r.URL) | $($r.Time) | $($r.Perf) | $($r.FCP) | $($r.LCP) | $($r.SI) | $($r.TBT) | $($r.CLS) | $($r.TTI) |"
}
Set-Content -Path $OutMd -Value ($md -join "`n") -Encoding UTF8

Write-Host "Wrote:" $OutCsv
Write-Host "Wrote:" $OutMd

