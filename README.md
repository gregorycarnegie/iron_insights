# Iron Insights 2

[![Refresh Data And Deploy](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml/badge.svg)](https://github.com/gregorycarnegie/iron_insights/actions/workflows/refresh-data-and-deploy.yml)
![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?logo=rust)
![Leptos](https://img.shields.io/badge/Leptos-0.8-ef3939)
![Polars](https://img.shields.io/badge/Polars-0.53-5A32FA)
![Trunk](https://img.shields.io/badge/Trunk-WASM-2f9e44)

A Rust + Leptos project that answers: **"How do I stack up?"** for powerlifting totals and lifts.

It builds compact histogram/heatmap aggregates from OpenPowerlifting data, serves them as static files, and renders percentile + comparison visuals in a client-side web app.

## Repo Layout

- `app/` - Leptos CSR frontend (WASM + Trunk)
- `pipeline/` - Rust data pipeline (download, aggregate, publish)
- `data/` - versioned published aggregate data (`vYYYY-MM-DD` + `latest.json`)
- `docs/` - static build output for GitHub Pages
- `scripts/qa.sh`, `scripts/qa.ps1` - integrity + payload budget checks
- `todo.md` - implementation checklist and scope notes

## Prerequisites

- Rust toolchain (stable)
- `wasm32-unknown-unknown` target
- Trunk (`cargo install trunk --locked`)
- `jq` (for `scripts/qa.sh` on Linux/macOS)
- PowerShell (for Windows helper scripts)

## Quickstart (Local)

### 1) Refresh data (optional if repo already has data)

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD

cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates

cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --version vYYYY-MM-DD \
  --keep-versions 4
```

### 2) Sync data into the app folder

From `app/` on Windows PowerShell:

```powershell
pwsh -File .\sync-data.ps1
```

On CI/Linux this is done via:

```bash
rm -rf app/data
mkdir -p app/data
cp -a data/. app/data/
```

### 3) Run frontend locally

```bash
cd app
trunk serve --open
```

The app fetches from relative paths first (for subpath hosting compatibility):

- `./data/latest.json`
- `./data/<version>/index.json`
- referenced `hist/*.bin`, `heat/*.bin`, `meta/*.json`

## Build For GitHub Pages

```bash
cd app
trunk build --release --dist ../docs --public-url "/<repo-name>/"
```

Then serve/deploy `docs/`.

## Data Pipeline Outputs

- `pipeline/output/openpowerlifting-latest.parquet`
- `pipeline/output/build_metadata.json`
- `data/vYYYY-MM-DD/`
- `data/latest.json`

Published version folders contain:

- `hist/{sex}/{equip}/{wc}/{age}/{tested}/{lift}.bin`
- `heat/{sex}/{equip}/{wc}/{age}/{tested}/{lift}.bin`
- `meta/{sex}/{equip}/{wc}/{age}/{tested}/{lift}.json`
- `index.json` plus shard indexes under `index_shards/`

## QA / Validation

Linux/macOS:

```bash
./scripts/qa.sh data docs
```

Windows PowerShell:

```powershell
pwsh -File .\scripts\qa.ps1 -DataDir data -SiteDir docs
```

Checks include:

- index -> file reference integrity
- histogram/heatmap metadata sanity
- aggregate totals non-zero
- payload budget summary
- optional URL timing probes

## CI/CD

GitHub Actions workflow: `.github/workflows/refresh-data-and-deploy.yml`

It runs on:

- push to `master`
- weekly schedule (`0 3 * * 0`)
- manual dispatch

Workflow steps:

1. Run pipeline (`01_download`, `02_build_aggregates`, `03_publish_data`)
2. Apply safeguards (row-count proxy drop check)
3. Commit refreshed `data/` back to `master` when changed
4. Build app with Trunk to `docs/`
5. Run QA script
6. Deploy `docs/` to GitHub Pages

## Notes

- Root crate (`src/main.rs`) is currently a placeholder; project functionality lives in `app/` and `pipeline/`.
- `todo.md` tracks completed work and remaining features (e.g., DOTS/Wilks/GL toggle, OpenIPF option).
