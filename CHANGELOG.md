# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-07-31

### Changed

- **Published payload encoding: varints with zero runs collapsed replace
  fixed-width `u32` counts.** `BINARY_FORMAT_VERSION` is now 2; both parsers
  reject v1 payloads, so republishing is required.

  The heatmap grid is 97% of every `.bin` and 87% of its cells are zero, but
  every count was stored as a 4-byte `u32`. `encode_counts`/`decode_counts`
  write a LEB128 varint per count, with a zero varint marking a run followed by
  its length. Across all 23,343 published files the largest heatmap cell is
  2,565 and the largest histogram count 44,157, so nearly every count fits in
  one or two bytes.

  Measured A/B republish from identical `records/` input:

  | | before | after |
  |---|---|---|
  | version bundle | 378.4 MB | **29.1 MB** (-92.3%) |
  | `.bin` bytes | 373.3 MB | **21.6 MB** |
  | index JSON | 5.1 MB | 7.5 MB |
  | files | 23,372 | **12,369** |
  | gzipped on the wire | 100% | **74.9%** |

  The file count drops because 13,252 slices now fall under `INLINE_THRESHOLD`
  and are base64-inlined into their index shard, costing those slices no HTTP
  request at all. That moves bytes from `.bin` into JSON, which is why the index
  grew.

  GitHub Pages already gzips `.bin` responses, so this is primarily a bundle-size
  change: the published site was approaching the documented 1 GB Pages limit,
  which planned per-era time slices would have multiplied.

- **Over-engineering audit: -1,703 net lines, two dependencies.** A whole-repo
  pass for dead code, duplicated logic, and abstractions with one caller. No
  feature was dropped; the site renders identically.

  Deleted outright: `webapp/share.rs` (189 lines of PNG share-card rendering
  with no callers, `#![allow(dead_code)]` at the top), `core::heatmap` and
  core's histogram diagnostics/density APIs (~300 lines reachable only from
  their own tests), `core::histogram_mean_stddev` (`charts.rs` keeps its own
  f64 copy), `kg_to_lbs`/`lbs_to_kg` (no non-test caller; `helpers.rs`
  hardcoded the factor), the Lander and O'Conner 1RM formulas (the picker
  offers four), three write-only load-timing signals, and the
  `?sd`/`?bd`/`?dd` deltas, which round-tripped from query string to
  `localStorage` and back without ever being read.

  Deduplicated: `slug` and `parse_shard_key` existed in both `core` and the
  pipeline; `rows_from_slice_index` existed in both `state.rs` and
  `cross_sex.rs`; the DPR/backing-store canvas setup and the heatmap axis block
  were each inlined twice in `charts.rs`. The four lift fields and six cohort
  selects in `InputForm`, and the seven measurement inputs in `bodyfat.rs`,
  became one component each.

  Dropped as unused flexibility: the `--write-meta-files` flag (no script or
  workflow set it) and with it `SliceMeta`/`HistMeta`/`HeatMeta` and the app's
  `meta/` fetch fallback — shard indexes have carried the summary inline since
  1.2.0, so the cohort summary is now a `Memo` rather than an async effect with
  its own request tracker. `SliceIndexEntries::Keys`, the legacy key-list index
  form, and `entry_paths_from_slice_key` went the same way; every published
  shard uses the map form. `SliceSelectorIndex` kept nine parallel lookup maps
  holding four clones of every row to drive five dropdowns, and now filters one
  `Vec` on demand. `BuildMetadata` no longer records three paths to files the
  stage deletes before it exits.

  Removed dependencies: `wasm-bindgen-futures` (zero references) and
  `gloo-timers` (one 180 ms debounce, now `leptos::leptos_dom::helpers::set_timeout`),
  plus the unused `Clipboard`, `History`, `Navigator` and `HtmlAnchorElement`
  `web-sys` features.

- **SEO pages and the ranking view now quote the same percentile.**
  `seo_geo/stats.rs` had its own `percentile_value`/`value_percentile` using an
  upper-edge CDF convention, while the app uses `iron_insights_core`, which
  places a value at its bin's centre. The same lift could therefore be reported
  differently on the landing page and in the app it links to. The local copies
  are gone in favour of `value_for_percentile`/`percentile_for_value`; published
  medians and worked examples shift by up to one bin. Heatmap axis ticks pick up
  the shared `format_axis_tick`, so bin edges may now show one decimal.

- Removed the dead root `iron_insights` package: `main.rs` was `Hello, world!`
  and `lib.rs` re-exported only `binary_counts` (a second, now-divergent copy of
  the binary format that still wrote fixed-width `u32` under the same `IIH1`
  /`IIM1` magic) and `rebin` (a duplicate of `iron_insights_core::rebin`).
  Nothing depended on either. The root manifest is now a virtual workspace, so
  `default-members` — which existed only to keep this package out of default
  builds — went with it.

### Fixed

- **`03_publish_data` left orphaned files when republishing a version.** The
  version directory was created but never cleared, so files written by an
  earlier run survived even when the fresh index no longer referenced them.
  Which slices get their own `.bin` depends on payload size, so the encoding
  change made this visible: republishing `v2026-07-30` stranded 11,003
  unreferenced files (42.9 MB). The workflow versions by UTC date, so any
  same-day re-run hit this. The directory is now removed before publishing.

- **`scripts/qa.sh` dropped inlined slices from its aggregate total.** The TSV
  it builds guarded `meta` against empty fields but not `bin`, and inlined
  slices carry no `bin`. Bash collapses adjacent tabs, so every field after it
  shifted by one: `bin_rel` picked up the shard path, and the slice's total was
  never counted. Verified against the published index — the guard recovers all
  13,252 inlined slices and makes the reported total exact (87,607,416 rather
  than 85,919,053). Pre-existing, but it affected 52 slices before this release
  and 13,252 after.

## [1.1.0] - 2026-07-30

### Added

- `scripts/refresh.sh` and `scripts/refresh.ps1` run all four pipeline stages
  (`01_download` -> `04_seo_geo`) for one dataset version, defaulting to today
  and validating the `vYYYY-MM-DD` format. Both stop on the first failing stage.

### Fixed

- **Published bundle size: corrupt source rows no longer size histogram and heatmap axes.**
  `build_histogram` and `build_heatmap` derive their axis bounds from the observed
  min/max of the input, with no sanity bound. OpenPowerlifting contains rows whose
  bodyweight is a typo, which yields scores such as a Wilks of 281,280 — a single
  such row stretched one grid to 112,513 x 201 cells (90.5 MB) holding 245,282
  lifters in 0.067% of its cells. The 200 worst files accounted for 2.5 GB of the
  2.9 GB bundle.

  `MetricPublisher::accumulate_row` now drops rows whose metric value is
  non-finite, non-positive, or above `metric_max_valid` (1500 kg for `Kg`,
  1000 points for `Dots`/`Wilks`/`GL`), and ignores bodyweights outside
  `VALID_BW_RANGE_KG` (20-300 kg) when building heat points. The guard sits at
  the single point where rows enter, so histograms, heatmaps and trend
  thresholds are all covered.

  Measured A/B republish from identical `records/` input:

  | | before | after |
  |---|---|---|
  | bundle | 1605.4 MB | **375.1 MB** (-76.6%) |
  | largest `.bin` | 48.50 MB | **0.57 MB** |
  | slices | 25,521 | 25,521 |
  | lifter rows | 84,730,168 | 84,718,104 (-12,064, -0.014%) |

  The 12,064 dropped rows are all `Wilks`/`GL` scores; no `Kg` row was
  affected, consistent with a corrupt bodyweight producing a valid total but a
  nonsense coefficient. The cap sits in a clean gap: male Wilks tops out at 918
  and female DOTS at 808, while the discarded female Wilks values run from
  1,040 to 281,280.

  Republishing is required for the fix to take effect on existing data.

### Changed

- Dependency bumps: `anyhow` 1.0.102 -> 1.0.104, `base64` 0.22.1 -> 0.23.0,
  `clap` 4.6.1 -> 4.6.4, `serde` 1.0.228 -> 1.0.229, `serde_json` 1.0.150 ->
  1.0.151, `leptos` 0.8.19 -> 0.8.20, `js-sys`/`web-sys` 0.3.102 -> 0.3.103,
  `wasm-bindgen` 0.2.125 -> 0.2.126, `wasm-bindgen-futures` 0.4.75 -> 0.4.76,
  `wasm-bindgen-test` 0.3.75 -> 0.3.76. The `base64` 0.22 -> 0.23 major bump
  needed no code changes; both call sites use `Engine` with
  `general_purpose::STANDARD`.

## [1.0.3]

- Added the stage-4 SEO/GEO generator and unified the pipeline stage structure.
