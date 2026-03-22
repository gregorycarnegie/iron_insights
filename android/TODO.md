# Android TODO

## Phase 1

- [x] Scaffold the native Android app shell.
- [x] Keep the website data contract as the source of truth.
- [x] Add a shared config for the public site base URL.
- [x] Add a Compose home screen that documents the current app state.

## Phase 2

- [x] Fetch `latest.json` from the site.
- [x] Fetch the version root index from the site.
- [x] Cache the current dataset version.
- [x] Add models and parsing for versioned indexes and a first histogram payload.
- [ ] Move reusable Rust core logic out of `app/src/core.rs` or define the first Android-side mirror.
- [x] Build a selector-driven percentile ranking screen in Compose.

## Phase 3

- [x] Add navigation between lookup and trends surfaces.
- [x] Add a calculators surface with 1RM and plate loading tools.
- [x] Add a comparison surface on top of embedded slice summaries.
- [x] Define a separate Android CI workflow.
- [x] Add a separate Android release workflow.
- [x] Document the Android release workflow and required environment secrets.
- [ ] Configure the `android-release` environment secrets and optional Play upload credentials.
- [x] Add offline cache and fallback error handling.
- [x] Port the heatmap codec and bodyweight-conditioned percentile flow.
- [ ] Decouple comparison summary loads from histogram and heatmap fetches.
