# todo.md — Leptos “How do I stack up?” (GitHub Pages, weekly data refresh)

## 0) Decide the “best lift” definition (based on OPL bulk CSV docs)

- [x] Use **sanctioned meets only** (`Sanctioned == "Yes"`) so we match how rankings/records treat unsanctioned meets (OPL docs: unsanctioned “do not count for rankings or records”).
- [x] Use **Best3 lift columns** as the canonical meet result for each lift:
  - [x] `Best3SquatKg`
  - [x] `Best3BenchKg`
  - [x] `Best3DeadliftKg`
  - [x] Use `TotalKg` for total comparisons (SBD only; blank/invalid totals excluded)
- [x] Define **per-lifter best (recommended)** to avoid prolific lifters overweighting distributions:
  - [x] Identify a lifter by `Name` (OPL disambiguates duplicates via `#` suffix, e.g. `John Doe #1`).
  - [x] For each lifter, compute `best_lift = max(Best3*Kg)` within a chosen population slice.
  - [x] Store the *meet context* for that best lift (at least `BodyweightKg`, `Date`, `Federation`, `MeetName`) so you can plot BW vs lift at time of the best.
- [x] Define population slices (MVP):
  - [x] `Sex` (M/F/Mx)
  - [x] `Equipment` (Raw/Wraps/Single-ply/Multi-ply/Unlimited/Straps)
  - [x] `Tested` (Yes vs empty) as a toggle
  - [x] `Event` filter appropriate to the lift:
    - [x] For squat comparisons: include rows where `Event` ∈ {SBD, SD, SB, S}
    - [x] For bench comparisons: include rows where `Event` ∈ {SBD, BD, SB, B}
    - [x] For deadlift comparisons: include rows where `Event` ∈ {SBD, BD, SD, D}
    - [x] For total comparisons: `Event == "SBD"` only
- [x] Data hygiene rules (keep it sane):
  - [x] Drop rows where the relevant `Best3*Kg` is null/<=0 (some feds can report odd negatives; ignore for “best”).
  - [x] Exclude disqualified/no-show rows when deriving “best” (e.g., `Place` in {DQ, DD, NS}) for MVP defaults.
  - [x] Require `BodyweightKg` for BW scatter; allow missing BW for histogram.

## 1) Repo + deployment layout (GitHub Pages)

- [x] Create repo (or monorepo) with:
  - [x] `/app` (Leptos project)
  - [x] `/data` (generated aggregate blobs + `latest.json`)
  - [x] `/docs` (Pages publish folder) **or** configure Pages to serve from `gh-pages` branch
- [x] Add `latest.json` format:
  - [x] `{"version":"vYYYY-MM-DD","revision":"<opl revision if available>"}`
- [ ] Ensure Leptos fetch path works on Pages subpath (base URL):
  - [ ] set `LEPTOS_SITE_ROOT` / router base path (or use relative fetches like `./data/...`)

## 2) Data pipeline (Rust + Polars, lazy-first)

- [x] Write `pipeline/src/bin/01_download.rs`:
  - [x] Download `openpowerlifting-latest.zip` (bulk CSV)
  - [x] Extract CSV to temp workspace
  - [x] Convert extracted CSV to Parquet (`openpowerlifting-latest.parquet`) for faster downstream scans
    - [x] Use a one-time CSV -> Parquet conversion per refresh run, then use Parquet as the canonical pipeline input
  - [x] Remove temporary source files after successful conversion:
    - [x] Delete downloaded ZIP (`openpowerlifting-latest.zip`)
    - [x] Delete extracted CSV (`openpowerlifting-latest.csv`)
  - [x] Record the dataset updated date + revision (from the bulk download page) into build metadata
- [x] Write `pipeline/src/bin/02_build_aggregates.rs` (Polars `LazyFrame`):
  - [x] Scan Parquet lazily (`scan_parquet`) for speed/memory efficiency
  - [x] Apply filters early (lazy predicates):
    - [x] `Sanctioned == "Yes"`
    - [x] optional `Tested` toggle outputs (build both “tested” + “all” slices)
    - [x] relevant `Event` membership for each lift type
  - [x] Compute per-lifter best:
    - [x] Group by: `Name`, `Sex`, `Equipment`, `Tested`(bucketed), plus lift type
    - [x] Aggregate: `max(Best3*Kg)` and capture `BodyweightKg` at that max
      - [ ] Use `sort_by(Best3*Kg).last()` pattern or `arg_max` logic to carry BW/Date from the best row
  - [x] Produce “records table” per slice: `{best_lift, bodyweight_at_best, ...}`

## 3) Bin strategy with “user-adjustable bin sizes”

Goal: visitors can change bin sizes *without refetching massive data*.

- [x] Choose **base (smallest) bin sizes** for stored aggregates:
  - [x] `lift_bin_base_kg = 2.5`
  - [x] `bw_bin_base_kg = 1.0`
- [x] Store aggregates at base resolution, then **re-bin client-side** by summing adjacent bins:
  - [x] Histogram rebin: combine `k` bins → bin_size = `k * base`
  - [x] Heatmap rebin: combine `kx × ky` blocks
- [x] Define allowed multipliers in UI to keep it simple and fast:
  - [x] Lift bin multipliers: 1×, 2×, 4× (2.5kg → 5kg → 10kg)
  - [x] BW bin multipliers: 1×, 2×, 5× (1kg → 2kg → 5kg)
- [x] Implement client-side rebin functions (pure, fast):
  - [x] `rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32>`
  - [x] `rebin_2d(grid: Vec<u32>, w: usize, h: usize, kx: usize, ky: usize) -> (Vec<u32>, w2, h2)`

## 4) File formats (keep payloads tiny)

- [x] Pick a compact binary format for counts (recommended):
  - [x] Header (little-endian): version, base_bin_size, min, max, dims
  - [x] Payload: `u32` counts (hist) or `u32` flattened grid (heatmap)
- [x] Add a tiny JSON “index” per population slice so the client knows which file to fetch:
  - [x] Example key: `sex=M|equip=Raw|tested=Yes|lift=D`
  - [x] Points to: `hist.bin` + `heat.bin` + metadata ranges

## 5) Build outputs (what /data contains)

- [x] `/data/vYYYY-MM-DD/`
  - [x] `/hist/{sex}/{equip}/{tested}/{lift}.bin`
  - [x] `/heat/{sex}/{equip}/{tested}/{lift}.bin`
  - [x] `/meta/{sex}/{equip}/{tested}/{lift}.json` (ranges, base bins, totals)
- [x] `/data/latest.json` updated to point to newest version folder
- [x] Optional: keep only last N versions to limit repo size
  - [x] N=4 (last month) is a good default

## 6) Leptos UI (MVP)

- [x] Inputs panel:
  - [x] Squat / Bench / Deadlift / Bodyweight (kg)
  - [x] Sex, Equipment, Tested toggles
  - [x] Bin size selectors (lift bin, BW bin) using multipliers
- [x] Outputs:
  - [x] Percentile + rank estimate (from histogram CDF)
  - [x] Histogram chart with “your lift” vertical line
  - [x] BW vs lift heatmap with “your point” overlay
- [x] Rendering approach:
  - [x] Histogram: SVG (simple) or Canvas (fast)
  - [x] Heatmap: Canvas (recommended)
- [x] Calculations client-side:
  - [x] Percentile from counts:
    - [x] `cdf = sum(counts[0..bin]) + 0.5*counts[bin]`
    - [x] `pct = cdf / total`
  - [x] “Top X%” formatting + guard rails when out of range

## 7) CI/CD (weekly scheduled refresh + Pages deploy)

- [ ] Add GitHub Actions workflow:
  - [ ] `schedule:` weekly (e.g., Sunday 03:00 UTC)
  - [ ] Manual trigger `workflow_dispatch`
- [ ] Steps:
  - [ ] Checkout repo
  - [ ] Set up Python (and cache deps)
  - [ ] Run download + aggregate scripts
  - [ ] Commit `/data` changes back to `main` **OR** push to separate `data` branch/repo
  - [ ] Build Leptos site
  - [ ] Deploy to GitHub Pages
- [ ] Add safeguards:
  - [ ] Fail build if download corrupt / row count drops unexpectedly
  - [ ] Print build metadata (dataset updated date, revision, row count)

## 8) QA + sanity checks

- [ ] Spot-check percentiles against known expectations (e.g., your 320 @ 109kg, Raw, M)
- [ ] Confirm Event filtering works (D-only meets don’t appear in SBD total)
- [ ] Confirm distributions change reasonably when toggling Tested/Equipment/Sex
- [ ] Measure payload sizes and first-load time (target: <1–2s on normal connection)

## 9) Nice-to-haves (after MVP)

- [ ] Add WeightClassKg filtering + “use recorded BW vs weight class”
- [ ] Add AgeClass / Division filters
- [ ] Add “compare totals using DOTS/Wilks/GL points” toggle (all computable client-side)
- [ ] Add “IPF-only dataset” option using `openipf-latest.zip`
