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
- [x] One test spans 01 → 04 in a single run (`src/chain_tests.rs`): a synthetic
  CSV in at stage 1, assertions on the page stage 4 renders. It covers the two
  things the per-stage tests cannot — that stage 1's `dataset_version` is the
  version stage 3 publishes under, and that every lifter survives all four
  stages into the published histogram. The fixture must stay wide on both axes
  or its payload drops under `INLINE_THRESHOLD`, gets inlined, and leaves stage
  4 asserting only its statless fallback

### Mutation testing

Run as an occasional audit, not a CI gate. Four shards over the two native
crates, ~7 min each on 32 cores. **Shards are 0-indexed and an out-of-range
shard fails silently** — `--shard 4/4` returns a handful of stray mutants
instead of erroring, so `0/4`..`3/4` is the correct set:

```sh
cargo mutants -p iron_insights_core -p iron_insights_pipeline \
  --test-tool=nextest -j 8 --shard 0/4 --output mutants-0   # ...1/4, 2/4, 3/4
cat mutants-*/mutants.out/missed.txt | sort -u
```

The web crate cannot be included: it has no native `cargo test` build.

Baseline run (966 mutants): 547 caught, 191 timeout, 43 unviable, 185 missed.
Timeouts are almost all loop conditions mutated into infinite loops, so they
are detected in practice.

Addressed so far — re-verified per file, `missed` before to after:

| file | before | after |
|---|---|---|
| `scoring.rs` | 74 | 0 |
| `bodyfat.rs` | 24 | 0 |
| `publish_data/versioning.rs` | 13 | 0 |
| `seo_geo/snippets.rs` | 14 | 0 |
| `publish_data/metric.rs` | 8 | 0 |
| `publish_data/histogram.rs` | 11 | 2 (equivalent) |
| `iron_insights_core/binary.rs` | 11 | 1 (equivalent) |
| `seo_geo/stats.rs` | 5 | 0 |

`stats.rs` came down to 2 by way of the stage 4 end-to-end tests, and the last
two were both the `100 * f / m` sex ratio — a second copy of the arithmetic in
`snippets.rs`, which was already covered. Deleting the copy killed them: both
call sites now go through `stats::ratio_pct`, so the existing snippets test
reaches it. Verified by mutating `*` to `+` and watching that test fail.

Re-run over both files together after the Wilks correction changed
`scoring.rs`: **136 mutants, 133 caught, 3 unviable, 0 missed.**

```sh
cargo mutants -p iron_insights_core -p iron_insights_pipeline \
  --file iron_insights_core/src/scoring.rs \
  --file iron_insights_pipeline/src/seo_geo/stats.rs \
  --test-tool=nextest -j 8
```

Not yet addressed (the whole-repo number has not been re-measured since):

- [ ] a long tail of 1-3 each in `seo_geo/render.rs`, `trends.rs`, `records.rs`, `accumulation.rs`, `rebin.rs`, `histogram.rs` (core)

### Beyond mutation: are the numbers actually right?

Mutation testing proves the tests notice when the code changes. It says nothing
about whether the code was right to begin with — a coefficient mistyped before
the first test was written is stable, so every test agrees with it.

Closing that gap for `scoring.rs` meant checking each literal against the
reference implementation the module names, OpenPowerlifting's `coefficients`
crate. DOTS and IPF Goodlift matched exactly. **The 2020 Wilks female
polynomial did not** — three coefficients and both clamps were wrong, and the
bad cubic term drove the denominator negative above ~144 kg, scoring those
lifters zero. See the CHANGELOG.

`wilks_matches_the_published_reference_multipliers` now asserts against the
multipliers OpenPowerlifting tabulates, so it is a real verification rather
than a recording of our own output. The remaining pins (DOTS, Goodlift,
bodyfat) still only detect drift — their coefficients are verified, but the
sources publish no worked values to check the outputs against.

- [x] `bodyfat.rs` had the same literal-by-literal check: Navy (both sexes,
  metric form), YMCA, Jackson-Pollock 3-site and 7-site (both sexes), and Siri.
  **All match their published sources** — no second Wilks. Worth knowing that
  Jackson-Pollock was validated on ages 18-61 while these functions accept
  0-120, and that the Navy `495/450` form is the centimetre variant, which is
  what the callers feed it
- [x] `GL` published 48 fewer rows than `Dots` and `Wilks`. Traced to four
  female rows with a typo'd bodyweight under 17.7 kg, where the female Goodlift
  denominator turns negative. Not a Goodlift bug — the reverse: DOTS and Wilks
  were *clamping* those bodyweights to 40 kg and publishing a fabricated score,
  while Goodlift dropped them. `records.rs` now declines to score any bodyweight
  outside `VALID_BW_RANGE_KG`, so all three agree; the kg histogram still counts
  the lift, since it needs no bodyweight

Deliberately not changed: OpenPowerlifting's Goodlift returns zero below 35 kg,
so it publishes no GL for any lifter under that weight. Ours scores them.
Matching the reference would drop 635 real children in the 20-35 kg band from
GL while DOTS and Wilks keep scoring them (clamped to 40), trading one
inconsistency for another. Left as-is on purpose.

Known equivalent mutants — no test can kill these, do not chase them:

- `publish_data/histogram.rs` `(width - 1)` / `(height - 1)` / `(bins - 1)` clamp bounds: indices are derived from the same edges that size the array, so the clamp never binds
- `iron_insights_core/binary.rs:217` `<` vs `<=` in `parse_combined_bin`: a 10-byte payload is rejected either way, because the embedded histogram blob would be empty
- `aggregate.rs` — `stack_size(64 * 1024 * 1024)` arithmetic on the worker thread, and `run`/`build_all` replaced with `Ok(())`; both entry points parse argv and cannot be driven from a test

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
- [x] Both QA scripts reimplemented the slice-key to path mapping that `iron_insights_core::parse_slice_key` owns. Those parsers existed only for the legacy key-list index form, which is no longer published, so they were deleted along with that branch and the dead `meta/` handling in both scripts and the CI safeguard. `qa.ps1` had also drifted to reading `hist`/`heat` fields that no longer exist, so its file check was verifying nothing
