# Main Page UX Overhaul (Beginner-Friendly + Viral)

## 0) Page structure + routing

- [x] Use a single main landing page `/` focused on: input → result → share
- [x] Remove separate “Stats for Nerds” page and keep advanced details progressive on main

## 1) Above-the-fold onboarding (remove confusion)

- [x] Add a 1–2 sentence hero instruction: “Enter your lifts to see how you rank among X lifters.”
- [x] Add a clear primary CTA button: `Calculate my ranking`
- [x] Make the first interactive section “Your Numbers” (big inputs, minimal options)

## 2) Minimal required inputs (default simple)

- [x] Inputs: Squat, Bench, Deadlift, Bodyweight
- [x] Two basic dropdowns only: Sex, Equipment
- [x] Sensible defaults pre-selected (e.g., Sex=M, Equipment=Raw)
- [x] Inline validation + guardrails (e.g., “Deadlift must be 0–600kg”)

## 3) Advanced filters (progressive disclosure)

- [x] Move these behind an “Advanced Options” accordion:
  - [x] Tested
  - [x] Age class
  - [x] IPF class / weight class mode (if applicable)
  - [x] Bin sizes (Lift bin, BW bin)
  - [x] Compare by (kg / lbs)
- [x] Add tooltips for every advanced field (1 sentence each)
- [x] Provide “Reset to defaults” inside Advanced

## 4) Hero result card (make it instantly understandable + shareable)

- [x] Create a large “Result Card” at top of results:
  - [x] “You are stronger than **XX.X%** of lifters”
  - [x] Show “Top **Y%**” framing (e.g., Top 0.5%)
  - [x] Show dataset size used (e.g., “Compared against 402,605 lifters”)
- [x] Add strength tier label (game-like ranks):
  - [x] Beginner / Novice / Intermediate / Advanced / Elite / Legendary / Mythical
- [x] Add a short explanation line: “Higher is stronger.”

## 5) Share / brag loop (viral mechanic)

- [x] Add `Share my ranking` button
- [x] Generate a clean share image (OG-style card) containing:
  - [x] Name/handle (optional)
  - [x] BW + lifts + chosen lift focus
  - [x] Percentile + tier
  - [x] Small site watermark/logo
- [x] Add “Copy link” button that preserves query params (so friends open the same view)
- [x] Add “Download PNG” (and/or “Copy image” if browser supports)

## 6) “What if I improve?” simulator (but framed correctly)

- [x] Move simulator below the hero result card
- [x] Rename section to: “What if you got stronger?”
- [x] Sliders update:
  - [x] percentile
  - [x] tier label
  - [x] rank (e.g., ~141 / 402,605)
- [x] Add quick preset buttons:
  - [x] `+10kg DL`
  - [x] `+20kg total`
  - [x] `Meet PRs`
  - [x] `1-year projection` (simple)

## 7) Fair comparison modes (ego mode)

- [x] Add compare tabs:
  - [x] All lifters
  - [x] Same bodyweight range
  - [x] Same weight class
  - [x] Same age class
- [x] Show a sentence summary per mode:
  - [x] “At 110–120kg BW, you’re stronger than XX%”

## 8) Simple visuals first, nerd charts optional

- [x] Show 1 simple chart first (optional):
  - [x] A small percentile bar (“You” marker on a 0–100 scale)
- [x] Add button: `View distribution charts` → expands histogram/heatmap
- [x] Keep full histogram + heatmap behind expandable section on main page

## 9) Copy + language improvements

- [x] Replace jargon in main UI:
  - [x] “BW” → “Bodyweight”
  - [x] “Lift bin” → “Grouping size”
  - [x] “Tested” → “Drug tested”
- [x] Add help tooltips where jargon remains
- [x] Add a short FAQ footer:
  - [x] “What does percentile mean?”
  - [x] “Where does the data come from?”
  - [x] “Why does equipment matter?”

## 10) Performance + perceived speed

- [ ] Precompute common slices (sex/equipment/tested) for instant response - i think we already do this
- [x] Add “calculating…” micro-animation that finishes fast
- [x] Cache last result in localStorage so refresh feels instant

## 11) Instrumentation (so you know what’s working) - delayed to future release

- [ ] Track: first input completion rate
- [ ] Track: share button clicks
- [ ] Track: advanced options opened rate
- [ ] Track: drop-off points (e.g., user changes filters then leaves)

## 12) Polish

- [x] Add “Units: kg/lb” toggle (default kg; remember preference)
- [x] Add “Clear all” / “Use my last numbers”
- [x] Mobile layout pass (inputs stacked, sticky result card)
