# TODO - New Site Pages

## Active Checklist (2-5)

- [x] Progression Simulator
  - [x] Define UX for "what-if" lift sliders (squat/bench/deadlift and total impact)
  - [x] Reuse current percentile/distribution data so rank shifts update in real time
  - [x] Add clear guardrails for invalid or missing inputs

- [x] Trends Over Time
  - [x] Add time-series charts for average total/percentile shifts
  - [x] Document cohort/filter caveats to avoid misleading comparisons

## Out of Scope (for now)

- Meet Finder / Results Explorer (not feasible right now)
- DOTS/Wilks Leaderboard (OPL already provides this)
- Records Dashboard (record rules/sources are tricky)
- Lifter Profiles (too heavy for current scope)

- [ ] Weight Class Analyzer
  - [ ] Show adjacent class distributions for a selected lifter profile/slice
  - [ ] Compare likely percentile in current class vs up/down class
  - [ ] Add explanatory copy around cut/fill assumptions

- [ ] Federation Comparison
  - [ ] Build side-by-side distribution views by federation
  - [ ] Ensure normalization/filters are consistent across compared groups
  - [ ] Add quick presets (e.g., USAPL vs USPA, tested vs untested)

## Dev Speed Checklist (Trunk/WASM)

- [ ] Ensure wasm target is installed once
  - [ ] `rustup target add wasm32-unknown-unknown`
- [ ] Avoid crates.io network stalls
  - [ ] Verify DNS/proxy/firewall allows `index.crates.io` and `crates.io`
  - [ ] If needed, set a stable mirror in cargo config
- [ ] Keep dependency fetches warm
  - [ ] Run `cargo fetch` in `app/` after dependency changes
- [ ] Reuse incremental build cache
  - [ ] Keep `app/target/` persistent (don’t clean unless necessary)
  - [ ] Use the same toolchain/profile between runs
- [ ] Use fast local dev commands
  - [ ] First run: `trunk serve` (no `--open` needed each time)
  - [ ] Iteration: keep one `trunk serve` process alive and edit files
  - [ ] Optional warm-up: `cargo check` in `app/` before `trunk serve`
- [ ] Keep warnings low to reduce noise while debugging perf
  - [ ] Fix new warnings quickly so real regressions stand out
