# todo.md — Leptos “How do I stack up?” (GitHub Pages, weekly data refresh)

## 0) Decide the “best lift” definition (based on OPL bulk CSV docs)

- [ ] Use **sanctioned meets only** (`Sanctioned == "Yes"`) so we match how rankings/records treat unsanctioned meets (OPL docs: unsanctioned “do not count for rankings or records”).
- [ ] Use **Best3 lift columns** as the canonical meet result for each lift:
  - [ ] `Best3SquatKg`
  - [ ] `Best3BenchKg`
  - [ ] `Best3DeadliftKg`
  - [ ] (Optional for total) `TotalKg` (only present when all 3 lifts succeed; blank if failed/DQ/etc.)
- [ ] Define **per-lifter best (recommended)** to avoid prolific lifters overweighting distributions:
  - [ ] Identify a lifter by `Name` (OPL disambiguates duplicates via `#` suffix, e.g. `John Doe #1`).
  - [ ] For each lifter, compute `best_lift = max(Best3*Kg)` within a chosen population slice.
  - [ ] Store the *meet context* for that best lift (at least `BodyweightKg`, `Date`, `Federation`, `MeetName`) so you can plot BW vs lift at time of the best.
- [ ] Define population slices (MVP):
  - [ ] `Sex` (M/F/Mx)
  - [ ] `Equipment` (Raw/Wraps/Single-ply/Multi-ply/Unlimited/Straps)
  - [ ] `Tested` (Yes vs empty) as a toggle
  - [ ] `Event` filter appropriate to the lift:
    - [ ] For squat comparisons: include rows where `Event` ∈ {SBD, SD, SB, S}
    - [ ] For bench comparisons: include rows where `Event` ∈ {SBD, BD, SB, B}
    - [ ] For deadlift comparisons: include rows where `Event` ∈ {SBD, BD, SD, D}
    - [ ] For total comparisons: `Event == "SBD"` only (or make it an explicit toggle)
- [ ] Data hygiene rules (keep it sane):
  - [ ] Drop rows where the relevant `Best3*Kg` is null/<=0 (some feds can report odd negatives; ignore for “best”).
  - [ ] Exclude disqualified/no-show rows when deriving “best” (e.g., `Place` in {DQ, DD, NS}) **unless** the lift fields are still valid and you intentionally want them (default: exclude).
  - [ ] Require `BodyweightKg` for BW scatter; allow missing BW for histogram.

## 1) Repo + deployment layout (GitHub Pages)

- [ ] Create repo (or monorepo) with:
  - [ ] `/app` (Leptos project)
  - [ ] `/data` (generated aggregate blobs + `latest.json`)
  - [ ] `/docs` (Pages publish folder) **or** configure Pages to serve from `gh-pages` branch
- [ ] Add `latest.json` format:
  - [ ] `{"version":"vYYYY-MM-DD","revision":"<opl revision if available>"}`
- [ ] Ensure Leptos fetch path works on Pages subpath (base URL):
  - [ ] set `LEPTOS_SITE_ROOT` / router base path (or use relative fetches like `./data/...`)

## 2) Data pipeline (Rust + Polars, lazy-first)

- [ ] Write `pipeline/src/bin/01_download.rs`:
  - [ ] Download `openpowerlifting-latest.zip` (bulk CSV)
  - [ ] Extract CSV to temp workspace
  - [ ] Record the dataset updated date + revision (from the bulk download page) into build metadata
- [ ] Write `pipeline/src/bin/02_build_aggregates.rs` (Polars `LazyFrame`):
  - [ ] Scan CSV lazily (`LazyCsvReader` / `scan_csv`) with explicit dtypes for speed/memory
  - [ ] Apply filters early (lazy predicates):
    - [ ] `Sanctioned == "Yes"`
    - [ ] optional `Tested` toggle outputs (build both “tested” + “all” slices)
    - [ ] relevant `Event` membership for each lift type
  - [ ] Compute per-lifter best:
    - [ ] Group by: `Name`, `Sex`, `Equipment`, `Tested`(bucketed), plus lift type
    - [ ] Aggregate: `max(Best3*Kg)` and capture `BodyweightKg` at that max
      - [ ] Use `sort_by(Best3*Kg).last()` pattern or `arg_max` logic to carry BW/Date from the best row
  - [ ] Produce “records table” per slice: `{best_lift, bodyweight_at_best, ...}`

## 3) Bin strategy with “user-adjustable bin sizes”

Goal: visitors can change bin sizes *without refetching massive data*.

- [ ] Choose **base (smallest) bin sizes** for stored aggregates:
  - [ ] `lift_bin_base_kg = 2.5` (or 1.0 if you can keep files small)
  - [ ] `bw_bin_base_kg = 1.0` (or 2.0 if you need smaller files)
- [ ] Store aggregates at base resolution, then **re-bin client-side** by summing adjacent bins:
  - [ ] Histogram rebin: combine `k` bins → bin_size = `k * base`
  - [ ] Heatmap rebin: combine `kx × ky` blocks
- [ ] Define allowed multipliers in UI to keep it simple and fast:
  - [ ] Lift bin multipliers: 1×, 2×, 4× (2.5kg → 5kg → 10kg)
  - [ ] BW bin multipliers: 1×, 2×, 5× (1kg → 2kg → 5kg)
- [ ] Implement client-side rebin functions (pure, fast):
  - [ ] `rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32>`
  - [ ] `rebin_2d(grid: Vec<u32>, w: usize, h: usize, kx: usize, ky: usize) -> (Vec<u32>, w2, h2)`

## 4) File formats (keep payloads tiny)

- [ ] Pick a compact binary format for counts (recommended):
  - [ ] Header (little-endian): version, base_bin_size, min, max, dims
  - [ ] Payload: `u32` counts (hist) or `u32` flattened grid (heatmap)
- [ ] Add a tiny JSON “index” per population slice so the client knows which file to fetch:
  - [ ] Example key: `sex=M|equip=Raw|tested=Yes|lift=D`
  - [ ] Points to: `hist.bin` + `heat.bin` + metadata ranges

## 5) Build outputs (what /data contains)

- [ ] `/data/vYYYY-MM-DD/`
  - [ ] `/hist/{sex}/{equip}/{tested}/{lift}.bin`
  - [ ] `/heat/{sex}/{equip}/{tested}/{lift}.bin`
  - [ ] `/meta/{sex}/{equip}/{tested}/{lift}.json` (ranges, base bins, totals)
- [ ] `/data/latest.json` updated to point to newest version folder
- [ ] Optional: keep only last N versions to limit repo size
  - [ ] N=4 (last month) is a good default

## 6) Leptos UI (MVP)

- [ ] Inputs panel:
  - [ ] Squat / Bench / Deadlift / Bodyweight (kg)
  - [ ] Sex, Equipment, Tested toggles
  - [ ] Bin size selectors (lift bin, BW bin) using multipliers
- [ ] Outputs:
  - [ ] Percentile + rank estimate (from histogram CDF)
  - [ ] Histogram chart with “your lift” vertical line
  - [ ] BW vs lift heatmap with “your point” overlay
- [ ] Rendering approach:
  - [ ] Histogram: SVG (simple) or Canvas (fast)
  - [ ] Heatmap: Canvas (recommended)
- [ ] Calculations client-side:
  - [ ] Percentile from counts:
    - [ ] `cdf = sum(counts[0..bin]) + 0.5*counts[bin]`
    - [ ] `pct = cdf / total`
  - [ ] “Top X%” formatting + guard rails when out of range

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
