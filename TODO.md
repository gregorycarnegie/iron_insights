# Iron Insights TODO

Current focus: website + published dataset only. The Android client has been removed from this repo for now so product and engineering effort stays centered on the site.

## Website

- [ ] Add integration or snapshot coverage for the highest-risk flows in `iron_insights_web`
- [ ] Break up the largest UI modules (`trends.rs`, `plate_calc.rs`, `one_rm.rs`) before they sprawl further
- [ ] Keep tightening the first-run and mobile experience in the main site
- [ ] Continue burning down the detailed UI checklist in `iron_insights_web/todo.md`

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
- [ ] Replace `og:image` (currently the favicon SVG) with a 1200x630 raster — needs a real image asset; not yet done
- [x] Add `HowTo` schema to the plate-calculator and 1RM pages
- [x] `llms.txt`: skipped — Google ignores it and payoff is low

## Pipeline And Publishing

- [ ] Revisit content-addressed `.bin` filenames if hosting moves from GitHub Pages to a CDN with immutable cache headers
- [ ] Keep the published-data contract in the root `README.md` and `iron_insights_web/README.md` synchronized
- [ ] Track payload size, slice counts, and refresh safety rails as the dataset grows
