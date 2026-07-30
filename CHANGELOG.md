# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
