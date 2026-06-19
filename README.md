# Iron Insights

[![Refresh Data And Deploy](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml/badge.svg)](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml)
![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?logo=rust)
![Leptos](https://img.shields.io/badge/Leptos-0.8-ef3939)
![Polars](https://img.shields.io/badge/Polars-0.53-5A32FA)
![Trunk](https://img.shields.io/badge/Trunk-WASM-2f9e44)

Iron Insights is a Rust + Leptos powerlifting data project built around one question:
**"How do I stack up?"**

The repo downloads OpenPowerlifting, builds compact published data bundles, and serves a static web app. The Android client has been removed from this repo for now so work can stay focused on the site.

## What Is In Here

- `iron_insights_web/` - Leptos CSR frontend built with Trunk
  - ranking page for quick percentile results
  - "Stats for Nerds" page for cohort comparison, distribution analysis, targets, and trends
  - "Men vs Women" page for aligned cross-sex cohort comparisons
  - 1RM calculator and plate calculator utilities
  - `seo/`, `robots.txt`, and `sitemap.xml` for static, crawlable SEO/GEO landing pages (one answer-first page per tool); regenerate with `python scripts/build_seo_pages.py`, audit with `python scripts/seo_audit.py`
- `iron_insights_core/` - shared Rust crate for published-data contracts and binary format logic used by the web app and pipeline
- `iron_insights_pipeline/` - Rust data pipeline that downloads, aggregates, and publishes versioned data bundles
- `data/` - published dataset snapshots such as `v2026-03-20/` plus `latest.json`
- `docs/` - GitHub Pages build output
- `scripts/qa.sh`, `scripts/qa.ps1` - integrity and payload checks for published data and site output
- `src/` - placeholder root crate; product code lives in `iron_insights_web/` and `iron_insights_pipeline/`

## Prerequisites

- Rust stable
- `wasm32-unknown-unknown` target
- Trunk (`cargo install trunk --locked`)
- `jq` for `scripts/qa.sh` on Linux/macOS
- PowerShell if you want to use the provided Windows helper scripts

See `iron_insights_web/README.md` for the frontend data-contract details.

## Local Workflow

### 1) Build or refresh the published data

```bash
cargo run --manifest-path iron_insights_pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD

cargo run --manifest-path iron_insights_pipeline/Cargo.toml --bin 02_build_aggregates

cargo run --manifest-path iron_insights_pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --version vYYYY-MM-DD \
  --keep-versions 2
```

Notes:

- `01_download` defaults to the latest OpenPowerlifting ZIP and writes `iron_insights_pipeline/output/openpowerlifting-latest.parquet`.
- `03_publish_data` writes the versioned bundle into root `data/`. That is the source of truth.

### 2) Sync root data into the app copy

From `iron_insights_web/` on Windows PowerShell:

```powershell
pwsh -File .\sync-data.ps1
```

On Linux/macOS:

```bash
rm -rf iron_insights_web/data
mkdir -p iron_insights_web/data
cp -a data/. iron_insights_web/data/
```

`iron_insights_web/data/` is a working copy used by Trunk. It can lag behind root `data/` until you resync it.

### 3) Run the frontend locally

```bash
cd iron_insights_web
trunk serve --open
```

The app loads:

- `data/latest.json`
- `data/<version>/index.json`
- `data/<version>/index_shards/<sex>/<equip>/index.json`
- referenced `bin/*.bin`
- `data/<version>/trends_shards/<sex>/<equip>/trends.json`
- optional `meta/*.json` only when verbose compatibility output is enabled

## Build For GitHub Pages

```bash
cd iron_insights_web
trunk build --release --dist ../docs --public-url "/<repo-name>/"
```

That mirrors the GitHub Actions deploy step and produces a static site under `docs/`.

## Published Data Layout

Each published version under `data/vYYYY-MM-DD/` contains:

- `index.json` - root shard lookup by `sex` and `equip`
- `index_shards/<sex>/<equip>/index.json` - slice lookup with embedded per-slice summary
- `bin/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.bin`
- `trends_shards/<sex>/<equip>/trends.json` - yearly cohort counts plus p50/p90 thresholds per shard
- optional `meta/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.json`

Metric behavior:

- squat, bench, and deadlift publish only `Kg`
- total publishes `Kg`, `Dots`, `Wilks`, and `GL`
- default publish mode embeds summary in shard indexes and skips `meta/`
- `--write-meta-files true` writes legacy per-slice JSON metadata

## QA And Validation

Linux/macOS:

```bash
./scripts/qa.sh data docs
```

Windows PowerShell:

```powershell
pwsh -File .\scripts\qa.ps1 -DataDir data -SiteDir docs
```

Checks include slice reference integrity, histogram and heatmap sanity, non-zero totals, and payload size reporting.

## CI/CD

Workflows:

- `.github/workflows/refresh-data-and-deploy.yml` - pipeline refresh, web build, QA, and Pages deploy

## Notes

- The public app branding is `Iron Insights`; the workspace root here still uses the local checkout name `iron_insights2`.
- `iron_insights_web/dist/` and `docs/` are generated outputs, not authoritative source files.
