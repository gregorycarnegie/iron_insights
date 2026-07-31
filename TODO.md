# Iron Insights TODO

Current focus: website + published dataset only. The Android client has been removed from this repo for now so product and engineering effort stays centered on the site.

## Website

- [x] Add integration or snapshot coverage for the highest-risk flows in `iron_insights_web` — 29 `wasm-bindgen-test` cases (six per-page render smoke tests, the selector cascade snapshot, plate/1RM/percentile helpers). They previously compiled but never ran; see [Testing](#testing).
- [ ] Break up the largest UI modules — currently `app.rs` (862), `cross_sex.rs` (773), `charts.rs` (720), `nerds.rs` (675)
- [ ] Keep tightening the first-run and mobile experience in the main site
- [ ] Continue burning down the detailed UI checklist in `iron_insights_web/todo.md`

## Testing

The `iron_insights_web` tests run in headless Chrome via `wasm-bindgen-test`.
`.cargo/config.toml` points the wasm target at `wasm-bindgen-test-runner`, which
needs two things installed:

```sh
cargo install wasm-bindgen-cli --version 0.2.126 --locked   # must match Cargo.lock
# plus a chromedriver whose major version matches the local Chrome, on PATH
# (or pointed at by $CHROMEDRIVER) — https://googlechromelabs.github.io/chrome-for-testing/
```

Then:

```sh
cargo test --workspace --exclude iron_insights_web
cargo test --manifest-path iron_insights_web/Cargo.toml --target wasm32-unknown-unknown
```

CI runs both, and resolves chromedriver from the runner image's `CHROMEWEBDRIVER`.
Bumping `wasm-bindgen` means bumping the pinned version in the workflow too.

- [x] Every stage now has end-to-end coverage, driven from shared fixtures in `src/test_support.rs`:
  - **01** — a hand-built zip (with a decoy non-CSV entry) through extraction, parquet conversion and metadata stamping. The `Float32` schema override is asserted on a wholly-empty column, since that is what stops stage 2's `> 0` filters silently matching nothing. Only the HTTP GET is left uncovered; `run` does the fetch and delegates the rest to `convert_downloaded_zip`
  - **02** — a synthetic source parquet through filtering, per-lifter best aggregation and cohort splitting, asserting the output columns are exactly the seven stage 3 reads
  - **03** — records parquet in, published tree out, read back as raw JSON with payloads decoded by `iron_insights_core::parse_combined_bin`
  - **04** — runs a real stage 3 publish first, then generates against it, so the 03 → 04 seam is covered. The fixture is deliberately wide enough to exceed `INLINE_THRESHOLD`, otherwise no `.bin` files exist and stage 4 would silently test only its statless fallback
- [ ] No test spans 01 → 04 in one run. Each stage is covered against fixtures of the next one's input shape, which catches schema drift but not a stage that is skipped entirely
- [ ] Consider `cargo-mutants` as an occasional audit rather than a CI gate; it is the only way to tell whether these tests actually catch anything, and a manual spot-check (breaking a published total, confirming a red test) is the cheap version
- [ ] 4 pre-existing `clippy::float_cmp` warnings in `helpers.rs` proptests — exact float equality is intentional there, so either `#[allow]` them with a reason or switch to an epsilon compare

## GEO (Generative Engine Optimization)

Audit of the static SEO pages, `robots.txt`, and `sitemap.xml` against Google's AI-optimization guide and GEO citation research. Strong technical foundation (AI crawlers allowed, static no-JS pages, valid JSON-LD, FAQ/Breadcrumb schema); the gaps below are mostly editorial. Edit sources in `iron_insights_web/seo/` (+ `index.html`/`robots.txt`/`sitemap.xml`), then rebuild — `dist/` is generated output.

Implemented in the pipeline's `04_seo_geo` stage (`iron_insights_pipeline/src/bin/04_seo_geo.rs`, 2026-06-20): the generator reads the published `.bin` cohort histograms and injects the real figures, so the tables below regenerate with every weekly data refresh. (Replaced the former `scripts/build_seo_pages.py`.)

P1 — pages describe data instead of showing it (biggest win):
- [x] Add a real data table with actual figures to `powerlifting-strength-standards` (median + 90th-percentile squat/bench/deadlift/total for common weight classes)
- [x] Add actual male/female ratios per lift to `male-vs-female-strength-comparison` (now shows the median table + DOTS-adjusted gap)
- [x] Add 2-3 worked numeric examples to `strength-percentile-calculator`
- [x] Generate these tables from the weekly pipeline so they stay current (parsed from `data/<version>/bin/...` IIC1 histograms)

P2 — freshness and precision:
- [x] Add `dateModified`/`datePublished` to each page's JSON-LD and a visible "data current as of <date>" line in the body
- [x] Replace vague "2M+" with the precise result count + date (now "2.8 million" / exact 2,844,167, computed from the data)

P3 — authority (E-E-A-T):
- [x] Add a `Person` node to the schema `@graph` and reference it as `publisher`/`author` (homepage + every SEO page)
- [x] Add a "How Iron Insights works" methodology page (`how-iron-insights-works`)

P3 — homepage:
- [x] Expand the `index.html` `<noscript>` into a real content summary (added headline medians, sex gap, methodology link)

P4 — polish:
- [ ] Replace `og:image` (still the favicon SVG, in both `iron_insights_web/index.html` and `seo_geo/render.rs`) with a 1200x630 raster — needs a real image asset
- [x] Add `HowTo` schema to the plate-calculator and 1RM pages
- [x] `llms.txt`: skipped — Google ignores it and payoff is low

## Pipeline And Publishing

- [ ] Revisit content-addressed `.bin` filenames if hosting moves from GitHub Pages to a CDN with immutable cache headers
- [ ] Keep the published-data contract in the root `README.md` and `iron_insights_web/README.md` synchronized
- [ ] Track payload size, slice counts, and refresh safety rails as the dataset grows
- [ ] `scripts/qa.sh` and `scripts/qa.ps1` each reimplement the slice-key to path mapping that `iron_insights_core::parse_slice_key` owns — a third and fourth copy, in two languages
- [ ] The CI safeguard's `meta/`-directory branch and `qa.sh`'s key-list index branch are both dead: no `meta/` tree is written and every shard uses the map form
