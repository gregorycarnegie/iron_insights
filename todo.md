# Stats For Nerds Implementation Plan

## Goal

- Keep `Ranking` beginner-friendly and fast: enter numbers, get percentile, understand result.
- Move advanced analytics to a dedicated `Stats for Nerds` tab so casual visitors never land in histogram/trends/debug territory by default.
- Reuse current published data as-is: no new shard dimensions, no new per-slice files, no heavier data packaging requirement.

## Current Constraints

- Keep the existing shard strategy: root index shards only by `sex + equipment`.
- Reuse already-published payloads:
  - slice index entry `summary`
  - `hist/*.bin`
  - `heat/*.bin`
  - `trends.json`
- Do not scope in features that require new aggregate dimensions such as federation, country, attempt history, meet-level browsing, or individual lifter records.

## Product Split

### `Ranking` tab should contain

- `OnboardingPanel`
- `ResultCardPanel`
- simple percentile bar from `PercentilePanel`
- short plain-English cohort sentence
- share actions
- FAQ

### `Stats for Nerds` tab should contain

- `CompareModePanel`
- `ChartsPanel`
- `SimulatorPanel`
- `ProgressPanel`
- `TrendsPanel`
- cross-sex comparison tools
- methodology/debug details
- bin-size controls
- any advanced percentile math or cohort diagnostics

### `1RM Calculator` tab should remain separate

- Keep the existing `OneRepMaxPanel` as its own utility page.

### `MeetDayPanel` decision

- Keep `MeetDayPanel` off the first implementation of `Stats for Nerds`.
- For now leave it on `Ranking` only if the page still feels light after the cleanup.
- If `Ranking` still feels crowded, move `MeetDayPanel` into a future `Tools` tab rather than calling it nerd analytics.

## Phase 1: Navigation And Page Structure

- [x] Add `AppPage::StatsForNerds` in `app/src/webapp/mod.rs`.
- [x] Update top navigation to `Ranking | Stats for Nerds | 1RM Calculator | Guides`.
- [x] Add hash routing for the new tab, e.g. `#rank`, `#nerds`, `#1rm`.
- [x] Keep `Ranking` as the default landing tab for first-time visitors.
- [x] Preserve existing query-param behavior so shared ranking links still open directly into the ranking experience.

### Acceptance criteria

- A first-time visitor sees no advanced analytics unless they click `Stats for Nerds`.
- Existing ranking links still work.
- `1RM Calculator` behavior remains unchanged.

## Phase 2: Simplify The `Ranking` Tab

- [x] Remove advanced panels from the default ranking flow:
  - `CompareModePanel`
  - `ChartsPanel`
  - `SimulatorPanel`
  - `ProgressPanel`
  - `TrendsPanel`
- [x] Remove `summary_load_ms`, `hist_load_ms`, `heat_load_ms`, and other debug-style blurbs from the ranking hero.
- [x] Keep only one cohort line in plain language, for example:
  - `Compared against 12,438 lifters in this cohort.`
- [x] Keep the percentile bar simple and non-technical.
- [x] Review the ranking page top-to-bottom for jargon and remove or rewrite any remaining “advanced” language.

### Acceptance criteria

- No histogram, heatmap, trends chart, target planner, or bin controls appear on `Ranking`.
- The entire ranking flow answers “how strong am I?” without requiring any chart literacy.

## Phase 3: Build The `Stats for Nerds` Tab

- [x] Create a dedicated nerd tab layout in `app/src/webapp/mod.rs` or extract a new page component under `app/src/webapp/components/`.
- [x] Move existing advanced panels into this tab:
  - `CompareModePanel`
  - `ChartsPanel`
  - `SimulatorPanel`
  - `ProgressPanel`
  - `TrendsPanel`
- [x] Add a short intro line at the top of the tab:
  - `Advanced distributions, cohort comparisons, and methodology details for people who want the full story.`
- [x] Organize the tab into sections instead of one long dump:
  - cohort
  - distributions
  - targets and simulations
  - trends
  - methodology/debug

### Acceptance criteria

- The advanced tab feels intentionally analytical, not like leftover clutter from the main page.
- Existing advanced features remain available, just relocated.

## Phase 4: Data-Loading Changes

- [x] Gate `setup_trends_effect` behind `active_page == AppPage::StatsForNerds` in `app/src/webapp/state.rs` and `app/src/webapp/mod.rs`.
- [x] Keep histogram loading available for ranking calculation because current percentile math depends on histogram bins.
- [x] Keep heatmap loading nerd-only.
  - Replace `show_main_charts` with a nerd-tab-specific condition, or make charts always available once the nerd tab is open.
- [x] Keep summary loading lightweight and available to both tabs.
- [x] Hide all load-timing/debug output from `Ranking`; keep it optional in nerd view.

### Notes

- `trends.json` is currently fetched eagerly on app load. This should become lazy.
- Histogram fetches cannot be fully deferred without changing how percentile results are computed.
- None of these changes require new shard boundaries.

### Acceptance criteria

- `Ranking` does not fetch `trends.json`.
- `Stats for Nerds` loads advanced payloads only when opened.
- No pipeline changes are required for Phase 4.

## Phase 5: Nerd-Only Features Using Existing Data

### 5.1 Distribution Diagnostics Card

- [x] Add a diagnostics card derived from the current histogram:
  - p1
  - p5
  - p10
  - p25
  - p50
  - p75
  - p90
  - p95
  - p99
  - IQR
  - central 80% range
  - mode bin
  - occupied bin count
  - sparsity score
  - sample-quality warning for tiny cohorts
- [x] Implement these as local computations from `HistogramBin`.

### 5.2 `Kg Per Percentile` Panel

- [x] Add a panel that answers:
  - how much weight to move up the next 1 percentile point
  - how much weight to move up the next 5 percentile points
  - how much weight to move up the next 10 percentile points
  - what `+2.5`, `+5`, and `+10 kg` would likely buy in percentile terms
- [x] Reuse the current inverse-percentile helpers instead of creating a new data format.

### 5.3 Rarity / Crowding Panel

- [x] Add a simple label based on local histogram density:
  - dense middle
  - moderately common
  - rare air
  - extreme tail
- [x] Show the user’s current bin count and neighboring-bin counts.

### 5.4 Bodyweight-Conditioned Percentile

- [x] Use the existing heatmap to estimate:
  - where the lifter sits among nearby bodyweights
  - how dense their bodyweight-and-lift neighborhood is
- [x] Replace the current generic “same bodyweight range” summary with an actual heatmap-derived metric inside the nerd tab.

### 5.5 Cohort Comparison Table

- [x] Add a comparison table for the current slice versus:
  - `All Ages`
  - `All` weight classes
  - tested vs all
  - kg vs Dots
  - kg vs Wilks
  - kg vs Goodlift
- [x] Use embedded shard summaries for cheap comparison rows.
- [x] Load additional histograms only when exact percentile deltas are needed.

### 5.6 Cross-Sex Comparison Panel

- [x] Add a nerd-only `Men vs Women` comparison panel.
- [x] Support side-by-side comparison of the current input against matching male and female cohorts under the same:
  - equipment
  - tested status
  - age class
  - weight class when possible
  - lift
  - metric
- [x] Show the user’s percentile in both cohorts when the same raw lift value is applied.
- [x] Add an `equivalent percentile` view:
  - if this lift is 82nd percentile among men, what lift is 82nd percentile among women
  - and vice versa
- [x] Prefer normalized metrics for the clearest “fair” comparison:
  - Dots
  - Wilks
  - Goodlift
- [x] If the user is viewing raw kg, add explicit caveat copy that cross-sex raw-load comparisons are descriptive, not apples-to-apples.
- [x] Use existing shard lookups for `sex=M` and `sex=F`; do not introduce any mixed-sex shard or new aggregate family.
- [x] Gracefully fall back when a matching opposite-sex cohort is missing or too small.

### 5.7 Trend Delta Panel

- [x] Extend the trends section to show:
  - cohort size growth over time
  - p50 drift over time
  - p90 drift over time
  - whether the user’s current lift would have cleared historical p50 or p90 in prior years
- [x] Keep this limited to the dimensions currently present in `trends.json`:
  - sex
  - equipment
  - tested
  - lift
  - metric

### 5.8 Methodology / Debug Box

- [x] Add a nerd-only methodology box with:
  - exact slice key
  - shard key
  - dataset version
  - dataset revision
  - histogram bin width
  - heatmap dimensions
  - summary total
  - summary min/max
  - optional load timings
  - note that percentile is computed from a mid-bin CDF approximation

## Phase 6: Supporting Refactors

- [x] Extract shared ranking state from `app/src/webapp/mod.rs` into cleaner page-level view sections if the file becomes too large.
- [x] Consider creating dedicated components for new nerd panels:
  - `distribution_diagnostics.rs`
  - `percentile_ladder.rs`
  - `rarity_panel.rs`
  - `cohort_comparison.rs`
  - `methodology_box.rs`
- [x] Keep `core.rs` as the home for histogram/heatmap-derived math helpers where practical.
- [x] Add unit tests for new percentile, quantile, density, and comparison helpers.

## Out Of Scope For This Plan

- Federation comparison across organizations
- Country or regional splits
- Individual lifter profiles
- Meet finder / results explorer
- Attempt-by-attempt analytics
- Year-by-year full percentile distributions for every slice

## Suggested Delivery Order

### Milestone 1: UX split

- [x] Add the `Stats for Nerds` tab
- [x] Move existing advanced panels out of `Ranking`
- [x] Remove debug-heavy copy from the main page

### Milestone 2: Lazy advanced loading

- [x] Gate `trends.json` behind the nerd tab
- [x] Keep heatmap nerd-only
- [x] Verify the ranking path still feels fast

### Milestone 3: First nerd-only value

- [x] Distribution diagnostics card
- [x] `kg per percentile` panel
- [x] rarity/crowding panel

### Milestone 4: Better cohort intelligence

- [x] bodyweight-conditioned percentile
- [x] cohort comparison table
- [x] cross-sex comparison panel
- [x] trend delta panel

### Milestone 5: Polish

- [x] methodology/debug box
- [x] test coverage
- [x] copy and layout pass

## Definition Of Done

- [x] Casual visitors can use `Ranking` without seeing advanced charts, trend jargon, bin controls, or debug information.
- [x] `Stats for Nerds` contains all advanced analytics in one intentional place.
- [x] At least the following nerd features ship without changing shard strategy:
  - distribution diagnostics
  - `kg per percentile`
  - rarity/crowding
  - bodyweight-conditioned percentile
  - cohort comparison
  - trend delta
  - methodology/debug
- [x] No new shard dimension or extra per-slice data family is introduced.
- [x] Existing ranking and 1RM behavior still works.
