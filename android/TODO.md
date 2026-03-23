# Android TODO

## Phase 1

- [x] Scaffold the native Android app shell.
- [x] Keep the website data contract as the source of truth.
- [x] Add a shared config for the public site base URL.
- [x] Add a Compose home screen that documents the current app state.
- [x] Add launcher icon and basic brand assets from the website mark.
- [x] Add a first website-aligned Compose theme pass for shared screens.

## Phase 2

- [x] Fetch `latest.json` from the site.
- [x] Fetch the version root index from the site.
- [x] Cache the current dataset version.
- [x] Add models and parsing for versioned indexes and a first histogram payload.
- [x] Add version-aware pruning for cached dataset payload trees.
- [x] Centralize shard and slice key parsing into a dedicated published-data contract helper.
- [x] Move reusable Rust core logic out of `app/src/core.rs` or define the first Android-side mirror.
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
- [x] Decouple comparison summary loads from histogram and heatmap fetches.

## Phase 4 — UX overhaul: data viewer → product

### P0: First-use experience (do these first)

- [x] **Hero prompt on Lookup screen.** Replace the current HeroCard/StatusCard top section with a single clear call-to-action: "How strong are you?" with three inputs (sex, lift, weight lifted) and a "Find out" button. Show the percentile result immediately. Filters like equipment, age, tested, metric stay collapsed under "Refine" or similar.
- [x] **Smart defaults.** Pre-select Male / Raw / All ages / All bodyweights / Total / Kg so the app shows a real result on launch with zero interaction. The current filter-first wall of chips is the single biggest friction point.
- [x] **Plain-English result headline.** After a percentile lookup, show a sentence like "You're stronger than 83% of similar lifters" in large text above the raw numbers. The number alone is not enough — users need the interpretation.
- [x] **Hide developer metadata.** Move "Live dataset status," "Payload sources," "Refresh latest.json," version root index, shard index, load sources, endpoint rows, and milestone rows behind a collapsible "Developer info" or "Data sources" section at the bottom of Lookup. Default collapsed.

### P1: Information hierarchy and copy

- [x] **Reduce explanatory text by ~50%.** Audit every SectionCard body text. Cut implementation-detail sentences ("These rows use embedded slice summaries only," "This screen is the fast summary layer on top of the current shard index"). Replace with one-line user-value statements or remove entirely.
- [x] **Rename "Compare" to something clearer.** Consider "Similar lifters," "Cohort," or "Nearby." The difference between Lookup and Compare is not obvious to a first-time user.
- [x] **Add section eyebrow labels that answer "what is this?"** Each card's eyebrow should tell the user what they get, not what the system is doing. Examples: "Your result" instead of "Percentile lookup," "Similar cohorts" instead of "Embedded summaries."
- [x] **Replace jargon throughout.** Rename or rewrite: "embedded summaries" → remove or say "quick comparisons," "root index" → remove from user UI, "shard index" → remove, "payload metadata" → remove, "contract view" → remove, "slice key" → remove. These terms can stay in code, but not in rendered text.
- [x] **Add interpretation under every major metric.** Wherever the app shows a raw number (percentile, cohort size, trend delta, 1RM estimate), add a short plain-English line below it. Examples: "Top 17% — well above average," "8,238 lifters in this cohort — large enough for reliable percentiles," "Your estimated max is in the intermediate range."

### P2: Screen-level improvements

#### Lookup

- [x] **Collapse filter chips into a compact summary.** After the user picks filters, show a one-line summary like "Men · Raw · All ages · Total · Kg" with an "Edit" button, instead of keeping all chip rows visible. This recovers screen space for the result.
- [x] **Add a "next milestone" callout.** After showing the percentile, show the next round-number threshold: "15 kg more to reach the 90th percentile." This gives users a goal.
- [x] **Move bodyweight-conditioned lookup below the main result.** It's a power-user feature. Most users want the simple percentile first.

#### Compare

- [x] **Lead with the comparison, not the explanation.** The current screen opens with a paragraph about what embedded summaries are. Instead, show the comparison table first, with a small "What is this?" expandable if needed.
- [x] **Add delta framing.** When comparing cohorts, highlight the difference: "+12% larger cohort," "range extends 45 kg higher." Raw side-by-side numbers require mental math.

#### Trends

- [x] **Add a punchline above the chart.** Before the sparkline, show the single most interesting takeaway in large text: "The median total for this cohort grew 23% since 2000" or "This cohort has grown from 52 to 8,238 lifters since 1965."
- [x] **Simplify chart labels.** The current chart requires understanding cohort definition, axis meaning, and log scale. Add a plain title like "Number of lifters per year" and consider a linear scale toggle.
- [x] **Reduce the metadata card.** "Bucket: year," "Series count: 176," "Sample series: sex=F…" is developer-facing. Hide behind expandable or remove.

#### Calculators

- [x] **Add outcome framing to 1RM.** Label the result tiers: "Conservative estimate," "Likely max," "Ambitious target." The current output is accurate but sterile.
- [x] **Add a "today's training" suggestion.** Based on the 1RM estimate, show 3-4 suggested working sets (e.g., "5×5 at 80 kg" or "3×3 at 90 kg") with brief rationale.
- [x] **Group training percentages into named zones.** Instead of just showing 60%, 70%, 80%… label them: "Light technique work (60%)," "Moderate volume (70%)," "Heavy work sets (80%)," "Top singles (95%)."

### P3: Visual polish and accessibility

- [x] **Increase body text contrast.** Audit SteelMist (#B2B2B8) and Graphite (#7A7A84) text on NightCard (#14141A) backgrounds. Some combinations may fail WCAG AA. Bump secondary text to at least #C0C0C8.
- [x] **Increase minimum touch targets.** Ensure all FilterChips and buttons meet 48dp minimum touch target. Some chip rows look tight in the screenshots.
- [x] **Add visual weight to primary results.** The percentile number and 1RM estimate should be the largest, boldest elements on their screens. Currently they compete with surrounding cards for attention.
- [x] **Reduce card density.** Add more vertical spacing between cards (current 16dp gaps could go to 20-24dp). This gives the eye a rest and improves scannability.
- [ ] **Consider a "first run" onboarding overlay.** A 2-3 step tooltip sequence pointing at: (1) enter your lift, (2) see your percentile, (3) explore trends. Dismiss permanently after first use.

### P4: Navigation and flow

- [x] **Make "I lifted X, how good is it?" the obvious entry point.** Consider making Lookup the persistent default tab, or adding a floating action button / bottom-sheet prompt that is always accessible regardless of current tab.
- [x] **Unify the primary user journey.** After showing a percentile result on Lookup, offer inline links: "See how this cohort has changed over time →" (goes to Trends with same filters) and "Compare to nearby cohorts →" (goes to Compare with same filters). Currently each tab is isolated.
- [x] **Add deep-link support.** Allow sharing a result (e.g., "I'm 83rd percentile for Men / Raw / Total at 500 kg") via Android share sheet. This drives organic adoption.
