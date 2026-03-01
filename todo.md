# TODO - Iron Insights 2 (Post-Review Backlog)

## P0 - Correctness and User Trust

- [x] Fix stale async result race in frontend data loading
  - [x] Add request token/versioning for shard loads (`slice_rows` fetch path)
  - [x] Add request token/versioning for hist/heat loads (`current_row` fetch path)
  - [x] Ignore late responses that do not match latest token
  - [x] Add lightweight logging message for ignored stale responses (debug only)
  - Files: `app/src/webapp.rs`

- [x] Clear visualization state on load miss/failure
  - [x] Clear `hist` + `heat` when no matching `current_row`
  - [x] Clear `hist` + `heat` when fetch fails or parse fails
  - [x] Ensure percentile/stat card shows empty-state for missing data
  - Files: `app/src/webapp.rs`

- [x] Enforce binary format version checks in frontend parser
  - [x] Parse and validate histogram version (currently only magic checked)
  - [x] Parse and validate heatmap version (currently only magic checked)
  - [x] Fail fast to `None` on unsupported version
  - [x] Surface parsing failure in `load_error` where practical
  - Files: `app/src/webapp.rs`, `pipeline/src/bin/03_publish_data.rs`

## P1 - Test Coverage for High-Risk Paths

- [x] Add unit tests for pipeline scoring and binning behavior
  - [x] `dots_points` numeric sanity tests
  - [x] `wilks_points` numeric sanity tests
  - [x] `goodlift_points` numeric sanity tests
  - [x] `build_histogram` edge bins and totals
  - [x] `build_heatmap` empty and non-empty behavior
  - Files: `pipeline/src/bin/03_publish_data.rs` (or extract to testable module)

- [x] Add unit tests for frontend binary parsing and percentile logic
  - [x] `parse_hist_bin` valid/invalid payload cases
  - [x] `parse_heat_bin` valid/invalid payload cases
  - [x] version mismatch parse rejection
  - [x] `percentile_for_value` boundary behavior
  - Files: `app/src/core.rs` (pure helper module used by `webapp.rs`)

- [x] Add regression test for rebin behavior parity (frontend vs root helpers)
  - [x] `rebin_1d` and `rebin_2d` preserve totals
  - [x] partial edge bin behavior documented in tests
  - Files: `app/src/core.rs`, `src/rebin.rs`

## P2 - Lint and CI Quality Gates

- [x] Make clippy pass with `-D warnings` across crates
  - [x] Root crate: remove useless conversion in test cleanup
  - [x] Pipeline: switch `&PathBuf` arg to `&Path`
  - [x] Pipeline: resolve redundant closure lint
  - [x] Pipeline scoring constants: allow/document precision choice or adjust literals deliberately
  - Files: `src/binary_counts.rs`, `pipeline/src/bin/02_build_aggregates.rs`, `pipeline/src/bin/03_publish_data.rs`

- [x] Add CI gates for quality
  - [x] Add `cargo test --workspace`
  - [x] Add `cargo test --manifest-path pipeline/Cargo.toml`
  - [x] Add `cargo test --manifest-path app/Cargo.toml`
  - [x] Add clippy steps (workspace + pipeline + app)
  - Files: `.github/workflows/refresh-data-and-deploy.yml`

## P3 - Code Organization (Maintainability)

- [x] Split `app/src/webapp.rs` into focused modules
  - [x] data loading/fetching
  - [x] binary parsing
  - [x] scoring + percentile math
  - [x] chart rendering
  - [x] UI component shell
  - Files: `app/src/webapp/mod.rs`, `app/src/webapp/data.rs`, `app/src/webapp/charts.rs`, `app/src/webapp/ui.rs`, `app/src/core.rs`

- [x] Add rustdoc for public API in root crate
  - [x] `rebin` helpers
  - [x] binary read/write headers and functions
  - Files: `src/rebin.rs`, `src/binary_counts.rs`

## Validation Checklist (Run Before Closing This Backlog)

- [x] `cargo test --workspace`
- [x] `cargo test --manifest-path pipeline/Cargo.toml`
- [x] `cargo test --manifest-path app/Cargo.toml`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo clippy --manifest-path pipeline/Cargo.toml --all-targets -- -D warnings`
- [x] `cargo clippy --manifest-path app/Cargo.toml --all-targets -- -D warnings`

## Notes

- Current known status from review:
  - P0 correctness fixes are implemented in frontend fetch/parse paths.
  - Pipeline now has scoring/binning tests in `03_publish_data.rs`.
  - App now has parser/percentile/rebin/scoring tests in `app/src/core.rs`.
  - Clippy `-D warnings` currently passes for workspace, pipeline, and app commands.
- Intentional sequence:
  - Do P0 first (user-visible correctness), then P1 (coverage), then P2 (gates), then P3 (refactor).
