# IRONSCALE web app

This crate is a client-side Leptos app. Trunk serves `index.html`, copies
`assets/`, and copies the static data bundle from `data/`.

## Data pipeline contract

The browser never reads the full OpenPowerlifting export. It follows a compact,
versioned lookup chain:

1. `data/latest.json`
   - Small pointer file loaded on startup.
   - Shape: `{ "version": "vYYYY-MM-DD", "revision": "optional label" }`.
   - `version` selects the immutable dataset directory under `data/`.

2. `data/<version>/index.json`
   - Root shard index.
   - Maps shard keys to per-sex/per-equipment shard JSON files.
   - Shard keys use `sex=<M|F>|equip=<equipment>`, for example
     `sex=M|equip=Raw`.

3. Shard JSON
   - Loaded after the user selects sex and equipment.
   - Contains slice keys for weight class, age class, tested status, lift, and
     metric.
   - The app accepts both supported shard shapes:
     - map form: slice key -> entry metadata
     - key list form: slice key only, with paths derived from the key

4. Slice entry
   - Resolves the selected cohort to a combined binary payload and optional
     summary metadata.
   - Important fields:
     - `bin`: relative path to the combined `.bin` payload.
     - `meta`: optional path to JSON metadata when the summary is not inlined.
     - `inline`: optional base64-encoded combined payload for sparse cohorts.
     - `summary`: optional `{ min_kg, max_kg, total }` cohort summary.

5. Combined `.bin` payload
   - Parsed by `iron_insights_core::parse_combined_bin`.
   - Contains both the one-dimensional lift histogram and the two-dimensional
     lift-by-bodyweight heatmap.
   - The app caches fetched binary payloads by URL for the life of the page.

## Runtime loading flow

Startup loads `latest.json`, then the selected root `index.json`, then the
current sex/equipment shard. Selector options are derived from the shard rows.
When the user computes a percentile, the selected slice payload is loaded and
parsed into histogram and heatmap data.

Cross-sex comparison follows the same contract, but loads the male and female
shards for the selected equipment in parallel and chooses the closest matching
male/female slice for the current filters.
