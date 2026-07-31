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

### Added

- **End-to-end coverage for pipeline stages 1, 2 and 4.** Each stage's `run`
  now parses argv and delegates to a function tests can call directly, and
  fixtures live in a shared `src/test_support.rs` so stages 3 and 4 exercise the
  same publish path rather than two approximations of it. Pipeline tests: 26 to
  47.

  Stage 1 runs a hand-built zip — including a decoy non-CSV entry, to prove the
  CSV is chosen by extension rather than by position — through extraction,
  parquet conversion, metadata stamping and temp-file cleanup. Only the HTTP GET
  is left uncovered. The `Float32` schema override is asserted against a column
  that is empty on every row, because that is the case where inference would
  otherwise pick a non-numeric type and stage 2's `> 0` filters would quietly
  stop matching; removing the override turns that column `Int64` and fails the
  test.

  Stage 2 takes a synthetic source parquet through the validity filters
  (DQ/DD/NS, unsanctioned, missing bodyweight, wrong event), per-lifter best
  aggregation, and the tested/all split, and pins the output columns to exactly
  the seven stage 3 reads — the cross-stage contract that `Name` and
  `TestedBucket` must not cross.

  Stage 4 runs a real stage 3 publish first and generates against its output, so
  the seam between them is covered rather than a hand-built `bin/` tree. The
  fixture is deliberately wide enough to exceed `INLINE_THRESHOLD`: a smaller one
  would be inlined into the index, leaving stage 4 no `.bin` files to read, and
  the test would pass while exercising only the statless fallback. Paired tests
  assert the real figures appear when payloads exist and the generic copy appears
  when they do not, so neither can silently become the other.

- **End-to-end coverage for `03_publish_data`.** `run` now parses argv and
  delegates to a `publish(&Args)` that tests can drive against a
  `tempfile::TempDir`: a synthetic records parquet goes in, and the published
  tree is read back the way the app reads it — raw JSON off disk, payloads
  through `iron_insights_core::parse_combined_bin`. Deliberately not via the
  crate's own serializable types, so a rename that breaks the web app fails the
  test instead of passing on both sides.

  Covers the version-tree layout, that every published slice key parses with the
  shared contract, that summary totals match the input rows, that republishing a
  version clears orphans rather than merging, and that pruning keeps the newest
  N. Verified non-vacuous by breaking a published total and confirming a red
  test.

- **Rendered-page invariants for the SEO stage.** Run against whatever
  `build_pages` returns, so a page added later is covered without touching the
  tests: no unsubstituted `[[TOKEN]]` placeholders survive rendering, slugs are
  unique and URL-safe (output is `seo/<slug>/index.html`, so a duplicate would
  silently overwrite), every page appears in `sitemap.xml`, `robots.txt`
  advertises the sitemap, and — the coupling most likely to rot — every page has
  a matching `copy-dir` link in `iron_insights_web/index.html`, without which
  trunk generates the page on every refresh and then drops it from the deploy.

- **Coverage for the rest of the mutation-testing survivors.** Core tests 70 to
  80, pipeline 47 to 63. Per file, surviving mutants before and after:
  `bodyfat.rs` 24 to 0, `versioning.rs` 13 to 0, `snippets.rs` 14 to 0,
  `metric.rs` 8 to 0, `histogram.rs` 11 to 2, `binary.rs` 11 to 1. The three
  that remain are equivalent mutants, listed in `TODO.md` so they are not
  chased again.

  Several needed a specific fixture rather than more assertions, which is the
  interesting part:

  - `versioning.rs` had no tests at all. Pruning is checked with five versions
    keeping two, because the obvious three-keeping-two case cannot tell
    `len - keep` from `len / keep`. Another asserts `latest.json` and unrelated
    directories survive, which is what breaks if the version-name check ever
    returns true unconditionally.
  - The heatmap y-axis was untestable with the production `BW_BIN_BASE_KG` of
    1.0, where dividing and multiplying by the base are the same operation, so
    the test uses 2.0. It also needs an 11-row grid: on a short one a wrongly
    multiplied index and the correct index both clamp to the same cell.
  - Cell-placement assertions replace sum-only ones. Clamping preserves the
    total, so a index computed from the wrong edge still lands somewhere and a
    sum assertion cannot see it.
  - `binary.rs` needed payloads of exactly 22 and 38 bytes for the `len < N`
    guards, right-length-wrong-magic for the `||`, and a run-overrun payload
    where `len - out.len()` and `len + out.len()` actually diverge. The
    `MAX_CELLS` boundary is affordable to test only because a zero run encodes
    four million cells in a handful of bytes.
  - The Navy, YMCA and Jackson-Pollock 3-site formulas are pinned alongside the
    7-site one, and each of the seven skinfold sites gets its own zero-reading
    case: with six other positive readings, a single dropped bound is invisible.

- The four `clippy::float_cmp` warnings in the web helpers are now `#[expect]`
  with a reason rather than silenced or loosened. `comparable_lift_value`
  returns the lift verbatim, so exact equality is the property under test; an
  epsilon would let a rounded or scaled value pass.

### Fixed

- **The `iron_insights_web` test suite never ran.** CI invoked it with
  `--no-run` and no WebDriver runner was configured, so all 29
  `wasm-bindgen-test` cases — six per-page render smoke tests, the selector
  cascade snapshot, and the plate/1RM/percentile helpers — were only ever
  type-checked. `.cargo/config.toml` now points the wasm target at
  `wasm-bindgen-test-runner`, and CI resolves chromedriver from the runner
  image's `CHROMEWEBDRIVER` and fails if it is absent, so the gate cannot
  silently degrade to a compile check again.

  Running them surfaced one real bug, below. See `TODO.md` for the local setup.

- **"All" sorted last in the weight-class dropdown, below `120+`.**
  `age_class_sort_key` special-cases "All Ages" to sort first, but
  `ipf_class_sort_key` had no equivalent, so the aggregate class fell into the
  unknown bucket `(2, i32::MAX)` and sorted after every numeric class — despite
  being the default selection. The never-run selector snapshot asserted the
  intended order (`["All", "83", "93"]`) and had been wrong since it was
  written. The two sort keys are now consistent.

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
