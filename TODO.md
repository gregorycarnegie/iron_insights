# Iron Insights Android TODO

## Decisions

- Keep Android in this repo under `android/`.
- Keep the existing Rust pipeline and published `data/` bundle as the only source of truth.
- Have Android consume the same public dataset the website uses instead of building a second backend.
- Split CI/CD by workflow file rather than by repository.

## Current Website Data Contract

- Base public site: `https://gregorycarnegie.github.io/iron_insights/`
- Dataset pointer: `data/latest.json`
- Versioned root index: `data/<version>/index.json`
- Shard index: `data/<version>/index_shards/<sex>/<equip>/index.json`
- Binary payloads: `hist/*.bin` and `heat/*.bin`
- Trends payload: `data/<version>/trends.json`

## Phase 1: Android Foundation

- [x] Create native Android project under `android/`
- [x] Use Kotlin, Compose, and Gradle Kotlin DSL
- [x] Add app shell with an explicit dataset base URL config
- [x] Add package structure for config, data, domain, and UI
- [x] Add a home screen that states current app status and next milestones
- [x] Add app icon and brand assets adapted from existing web assets
- [x] Add a first website-aligned theme pass for Compose surfaces and navigation

## Phase 2: Data Client

- [x] Add HTTP client for `latest.json`, root index, shard index, and `trends.json`
- [x] Define Kotlin models mirroring the existing JSON schema
- [x] Fetch and parse a first histogram binary for the preferred lookup slice
- [x] Fetch and parse the matching heatmap binary for nearby-bodyweight ranking
- [x] Add repository layer with version-aware caching
- [x] Add offline-friendly caching strategy for indexes and binary payloads
- [x] Add error handling for missing versions, parse failures, and stale cache
- [x] Add version-aware pruning for cached dataset payload trees

## Phase 3: Shared Logic

- [x] Extract the published contract layer first: JSON models plus shard and slice key parsing
- [x] Extract binary histogram and heatmap parsing into a shared crate or mirrored implementation
- [x] Extract percentile and distribution logic from the current web app into reusable core code
- [ ] Decide whether Android should call shared Rust through JNI or use a Kotlin port first
- [x] Keep binary format versioning explicit and tested

## Phase 4: First User Features

- [x] Implement percentile lookup flow
- [x] Implement filter controls for sex, equipment, bodyweight class, age, tested status, lift, and metric
- [x] Show percentile, rank, and cohort size
- [x] Add bodyweight-conditioned percentile flow
- [x] Add yearly cohort trend snapshot card
- [x] Add trends screen
- [x] Add comparison screen backed by embedded shard summaries
- [x] Add calculators for 1RM and plate loading
- [x] Decouple comparison summary loads from histogram and heatmap fetches

## CI/CD Split

- [x] Keep website/data deployment in its own workflow
- [x] Add `android-ci.yml` for PR validation
- [x] Add `android-release.yml` for signed releases
- [x] Document Android release workflow inputs and required secrets
- [x] Use workflow `paths` filters so website and Android pipelines do not trigger each other unnecessarily
- [ ] Configure GitHub environments and secrets for Android signing and store upload

## Code Quality

- [x] Canvas context `expect()` in `app/src/webapp/components/plate_calc.rs` `draw_barbell()` — replace with early return so a missing 2D context fails silently rather than panicking in WASM
- [ ] No unit tests in the `app` crate — WASM CSR makes unit tests awkward but integration/snapshot tests would catch regressions in complex components
- [ ] Component size — `trends.rs` (~474 LOC), `plate_calc.rs` (~562 LOC), and `one_rep_max.rs` (~434 LOC) exceed a comfortable reading size; candidates for sub-component extraction
- [x] No `///` doc comments on public types in `iron_insights_core` (`HistogramBin`, `HeatmapBin`, `HistogramDiagnostics`, etc.)
- [x] IPF weight class boundaries are hardcoded in `pipeline/src/bin/02_build_aggregates.rs`; if they ever change they have only one owner today but a shared-constants home in `iron_insights_core` would make the contract explicit

## Immediate Next Slice

- [x] Finish initial `android/` scaffold
- [x] Add real network layer and first endpoint fetch
- [x] Add tests for histogram codec and lookup math
- [x] Add first comparison surface on top of the shared selector state
- [x] Move `app/src/core.rs` into a reusable shared engine or define the first compatibility layer around it
