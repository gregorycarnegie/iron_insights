# TODO - Product And SEO Improvements

## Active Checklist

- [ ] Data Packaging / Delivery
  - [x] Reduce tiny-file count by bundling slice outputs into larger shard packs
  - [x] Split payloads into hot vs cold data for faster first load
  - [x] Keep metadata compact while merging adjacent per-slice binaries
  - [x] Validate fewer-request strategy with real page-load timing checks

- [ ] SEO Architecture
  - [x] Add crawlable static landing pages for high-intent queries
  - [x] Keep calculator UX in app, but route search traffic through static HTML entry pages
  - [x] Add unique title/description/canonical metadata per page
  - [x] Publish `robots.txt` and `sitemap.xml`
  - [ ] Submit sitemap in Search Console

- [ ] Tiered Loading Strategy
  - [x] Load percentile summary first
  - [x] Lazy-load histogram after initial result render
  - [x] Lazy-load heatmap only on expand/view intent
  - [x] Track perceived-load improvements on mobile

- [ ] Product Positioning
  - [ ] Add repo description and website URL
  - [ ] Add relevant repository topics
  - [ ] Create tagged releases for major milestones
  - [x] Document roadmap themes in README

## Feature Backlog

- [ ] Meet-Day Scorecard
  - [x] Add opener/second/third attempt helper
  - [x] Surface strongest-lift identity for current profile
  - [x] Add conservative vs aggressive suggestion mode

- [ ] Cohort Compare Expansion
  - [x] Compare against all lifters vs matched bodyweight
  - [x] Compare by age class, tested status, and federation slice
  - [x] Add quick compare presets for common cohorts

- [ ] Progress Tracking
  - [x] Save user snapshots locally
  - [x] Show percentile movement over time
  - [x] Support export/import snapshot JSON

- [ ] Target Planner
  - [x] Add "kg needed to reach top X%" projection
  - [x] Show lift-specific target guidance vs total-only guidance
  - [x] Add caveats around estimate uncertainty

- [ ] Trends Over Time
  - [x] Publish time-bucketed aggregates from pipeline
  - [x] Add trend charts for totals and percentiles
  - [x] Document sampling/cohort caveats

## Growth Checklist

- [ ] Landing Page Cluster
  - [x] Percentile calculator main page
  - [x] Squat percentile page
  - [x] Bench percentile page
  - [x] Deadlift percentile page
  - [x] Total percentile page
  - [x] Men/Women standards pages
  - [x] Tested/Untested comparison page

- [ ] Internal Linking
  - [x] Build topical clusters with bidirectional links
  - [x] Add links from FAQ/methodology into calculator pages
  - [x] Add links from calculator pages into definitions/method pages

- [ ] Structured Data
  - [x] Add `WebSite` schema
  - [x] Add `WebApplication` schema
  - [x] Add `FAQPage` schema

- [ ] Sharing And Viral Loop
  - [x] Improve Open Graph image quality and consistency
  - [x] Add shareable percentile/tier card flow
  - [x] Add lightweight "challenge a friend" comparison link flow

## Nice-To-Have

- [ ] Custom domain for stronger branding and cleaner URLs
- [ ] Long-tail keyword tracker with monthly refresh
- [ ] Search snippet A/B tests on page titles and descriptions
