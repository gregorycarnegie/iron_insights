# IRONSCALE app — improvement checklist

Current rating: **7/10**. Engineering is solid; the drag is structure, UX clarity, and verification.

## 1. Break up the god-file (`src/webapp/mod.rs`, 1498 lines)

- [ ] Extract cross-sex loading (rows, hist, heat, lift comparisons — 4 effects) into `webapp/cross_sex.rs`.
- [ ] Extract URL query / hash sync and `localStorage` unit persistence into `webapp/persistence.rs`.
- [ ] Move the `App` component's 60+ signal declarations into a typed `AppState` struct with grouped sub-states (`UserInput`, `Selection`, `CrossSexState`, `UiFlags`).
- [ ] Keep `mod.rs` to wiring + `run()` only — target under 300 lines.

## 2. Kill the 45-field context structs

- [ ] Replace `RankingCtx`, `InputFormCtx`, etc. with a single `AppState` passed via Leptos context (`provide_context` / `use_context`).
- [ ] Delete the `#[allow(dead_code)]` on `InputFormCtx` by actually removing unused fields (`_heat`, `_rebinned_heat`, `_canvas_ref` in `RankingPage`).

## 3. Product UX (from memory: "feels like a data viewer, not a product")

- [ ] First-run empty state: show a *sample* percentile result before the user types anything, so the payoff is visible.
- [ ] Hide filter controls (equip, age, tested, metric) behind an "Advanced" disclosure; pick smart defaults.
- [ ] Replace `STATS FOR NERDS` / `MEN VS WOMEN` uppercase jargon tabs with plain titles in the main nav; keep the typographic style for headings only.
- [ ] Add a one-sentence plain-English summary above every chart ("You out-lift 73% of 120kg male raw lifters").
- [ ] Mobile layout pass — the 240px sticky sidebar will break below ~720px; add a top-nav fallback.

## 4. Styling debt

- [ ] Purge inline `style="..."` strings in `ranking.rs` (25+ instances) — move to CSS classes in `assets/style.css`.
- [ ] Split `style.css` (1098 lines) into `base.css`, `layout.css`, `pages.css`, `components.css` via `@import` or a trunk pipeline.
- [ ] Define a spacing/size token scale (`--space-1` … `--space-8`) so magic pixel values (`padding:80px 40px`) stop multiplying.

## 5. Loading / error UX

- [ ] Replace the single plain `LOADING IRONSCALE` shell in `index.html` with skeleton panels that match the real layout (less layout shift).
- [ ] Consolidate the 8 separate `load_error`, `cross_sex_*_error` signals behind a toast/banner system; today they fight for the same real estate.
- [ ] Show a retry button when a shard fetch fails (currently silent after the first error string is set).

## 6. Testing (currently none in `app/src`)

- [ ] Add `#[cfg(test)]` unit tests for pure helpers: `calc_plates`, `format_count`, `next_unlock`, `tier_for_percentile`, `comparable_lift_value`.
- [ ] Add WASM-in-browser smoke tests via `wasm-bindgen-test` — one per page rendering without panicking.
- [ ] Add a snapshot of the selector index transitions (sex→equip→wc→age) to lock in cascade behavior.

## 7. Performance / correctness

- [ ] Memoize `rows_from_slice_index` output — it's re-sorted on every shard load even when inputs are identical.
- [ ] Cache already-fetched `.bin` payloads in a `HashMap<url, Bytes>` so switching filters doesn't re-download shards.
- [ ] Debounce the number inputs; currently each keystroke triggers a histogram redraw via `user_lift` memo.
- [ ] Verify canvas `devicePixelRatio` handling in `draw_ranking_distribution_canvas` and `draw_heatmap` — the window resize listener redraws but may not re-scale for HiDPI.

## 8. Accessibility

- [ ] Add visible focus styles for all `.nav-item`, tier-ladder marks, and custom toggles (unit switch).
- [ ] Verify color contrast: `--ink-mute #52504c` on `--bg #0b0b0d` is ~4.3:1 — borderline for small text.
- [ ] Ensure tier ladder keyboard-navigates and announces via `aria-live` when the "YOU" marker moves.
- [ ] Give the canvas charts a visually-hidden text equivalent (current aria-label is generic).

## 9. Hygiene

- [ ] Enable `#![warn(clippy::pedantic)]` on the `webapp` module and fix the fallout.
- [ ] Scan for and remove any leftover `#[allow(dead_code)]` (at least `InputFormCtx`).
- [ ] Document the data-pipeline contract in `app/README.md`: `data/latest.json` → `root_index` → shard → slice → bin payload, so onboarding doesn't require reading all of `mod.rs`.
