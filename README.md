# Iron Insights

[![Refresh Data And Deploy](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml/badge.svg)](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml)
![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?logo=rust)
![Kotlin](https://img.shields.io/badge/Kotlin-2.3.20-7F52FF?logo=kotlin&logoColor=white)
![Leptos](https://img.shields.io/badge/Leptos-0.8-ef3939)
![Polars](https://img.shields.io/badge/Polars-0.53-5A32FA)
![Trunk](https://img.shields.io/badge/Trunk-WASM-2f9e44)

Iron Insights is a Rust + Leptos powerlifting data project built around one question:
**"How do I stack up?"**

The repo downloads OpenPowerlifting, builds compact histogram and heatmap bundles, serves a static web app, and now includes a native Android client that consumes the same published data contract.

## What Is In Here

- `app/` - Leptos CSR frontend built with Trunk
  - ranking page for quick percentile results
  - "Stats for Nerds" page for cohort comparison, distribution analysis, targets, and trends
  - "Men vs Women" page for aligned cross-sex cohort comparisons
  - 1RM calculator and plate calculator utilities
  - `landing/`, `robots.txt`, and `sitemap.xml` for static SEO pages
- `android/` - native Kotlin + Jetpack Compose Android client
  - lookup, comparison, trends, and calculator screens backed by the published site dataset
  - Android-specific setup and release notes in `android/README.md` and `android/RELEASING.md`
- `iron_insights_core/` - shared Rust crate for published-data contracts and binary format logic used by the web app and pipeline
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
- Android Studio plus Android SDK Platform 35 if you want to run the Android app locally

See `android/README.md` for the Android-specific workflow, release inputs, and output paths.

## Local Workflow

### 1) Build or refresh the published data

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD

cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates

cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --version vYYYY-MM-DD \
  --keep-versions 2
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

### 4) Run the Android app locally

The Android client consumes the same published payloads as the website.

Best path:

- open `android/` in Android Studio
- let Studio install any missing SDK pieces
- run the `app` configuration on a device or emulator

Command-line builds also work with the checked-in Gradle wrapper:

```bash
./android/gradlew -p android testDebugUnitTest
./android/gradlew -p android :app:assembleDebug
```

Detailed Android setup, release-signing inputs, and output locations are documented in `android/README.md`.

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

Workflows:

- `.github/workflows/refresh-data-and-deploy.yml` - pipeline refresh, web build, QA, and Pages deploy
- `.github/workflows/android-ci.yml` - Android debug build plus unit tests
- `.github/workflows/android-release.yml` - signed Android release bundle build and optional Play upload

## Notes

- The public app branding is `Iron Insights`; the workspace root here still uses the local checkout name `iron_insights2`.
- `app/dist/` and `docs/` are generated outputs, not authoritative source files.
