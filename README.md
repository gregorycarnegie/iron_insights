# Iron Insights

[![Refresh Data And Deploy](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml/badge.svg)](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml)
![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?logo=rust)
![Leptos](https://img.shields.io/badge/Leptos-0.8-ef3939)
![Polars](https://img.shields.io/badge/Polars-0.53-5A32FA)
![Trunk](https://img.shields.io/badge/Trunk-WASM-2f9e44)

Iron Insights is a Rust + Leptos powerlifting data project built around one question:
**"How do I stack up?"**

The repo downloads OpenPowerlifting, builds compact histogram and heatmap bundles, and serves a static web app with percentile ranking, cohort analysis, cross-sex comparison, and training tools.

## What Is In Here

- `app/` - Leptos CSR frontend built with Trunk
  - ranking page for quick percentile results
  - "Stats for Nerds" page for cohort comparison, distribution analysis, targets, and trends
  - "Men vs Women" page for aligned cross-sex cohort comparisons
  - 1RM calculator and plate calculator utilities
  - `landing/`, `robots.txt`, and `sitemap.xml` for static SEO pages
- `pipeline/` - Rust data pipeline that downloads, aggregates, and publishes versioned data bundles
- `data/` - published dataset snapshots such as `v2026-03-20/` plus `latest.json`
- `docs/` - GitHub Pages build output
- `scripts/qa.sh`, `scripts/qa.ps1` - integrity and payload checks for published data and site output
- `src/` - placeholder root crate; product code lives in `app/` and `pipeline/`

## Prerequisites

- Rust stable
- `wasm32-unknown-unknown` target
- Trunk (`cargo install trunk --locked`)
- `jq` for `scripts/qa.sh` on Linux/macOS
- PowerShell if you want to use the provided Windows helper scripts

## Local Workflow

### 1) Build or refresh the published data

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD

cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates

cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --version vYYYY-MM-DD \
  --keep-versions 4
```

Notes:

- `01_download` defaults to the latest OpenPowerlifting ZIP and writes `pipeline/output/openpowerlifting-latest.parquet`.
- `03_publish_data` writes the versioned bundle into root `data/`. That is the source of truth.

### 2) Sync root data into the app copy

From `app/` on Windows PowerShell:

```powershell
pwsh -File .\sync-data.ps1
```

On Linux/macOS:

```bash
rm -rf app/data
mkdir -p app/data
cp -a data/. app/data/
```

`app/data/` is a working copy used by Trunk. It can lag behind root `data/` until you resync it.

### 3) Run the frontend locally

```bash
cd app
trunk serve --open
```

The app loads:

- `data/latest.json`
- `data/<version>/index.json`
- `data/<version>/index_shards/<sex>/<equip>/index.json`
- referenced `hist/*.bin` and `heat/*.bin`
- `data/<version>/trends.json`
- optional `meta/*.json` only when verbose compatibility output is enabled

## Build For GitHub Pages

```bash
cd app
trunk build --release --dist ../docs --public-url "/<repo-name>/"
```

That mirrors the GitHub Actions deploy step and produces a static site under `docs/`.

## Published Data Layout

Each published version under `data/vYYYY-MM-DD/` contains:

- `index.json` - root shard lookup by `sex` and `equip`
- `index_shards/<sex>/<equip>/index.json` - slice lookup with embedded per-slice summary
- `hist/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.bin`
- `heat/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.bin`
- `trends.json` - yearly cohort counts plus p50/p90 thresholds
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

Workflow: `.github/workflows/refresh-data-and-deploy.yml`

It currently:

1. runs tests and clippy across the workspace
2. rebuilds the data bundle with the pipeline
3. applies a drop-threshold safeguard against suspicious aggregate shrinkage
4. commits refreshed `data/` when it changed
5. syncs `app/data/`, builds `docs/` with Trunk, runs QA, and deploys GitHub Pages

## Notes

- The public app branding is `Iron Insights`; the workspace root here still uses the local checkout name `iron_insights2`.
- `app/dist/` and `docs/` are generated outputs, not authoritative source files.
