# App (Leptos CSR)

This crate is the client-side Iron Insights web app. It is a WASM-only Leptos CSR build served with Trunk.

## Current App Surface

- `#rank` - quick percentile ranking and share output
- `#nerds` - cohort comparison, distribution diagnostics, percentile ladder, rarity, bodyweight-conditioned stats, target planning, and trends
- `#men-vs-women` - aligned male vs female cohort comparison
- `#1rm` - 1RM estimator using blended Epley + Brzycki logic
- `#plate-calc` - barbell plate loading helper

The app also includes:

- local snapshot tracking with JSON export/import
- shareable state via query parameters and hash navigation
- PNG ranking card export
- static landing pages under `landing/`

## Data Expectations

The source of truth is root `../data/`. The app reads a synced working copy in `app/data/`.

Runtime fetches currently use:

- `data/latest.json`
- `data/<version>/index.json`
- `data/<version>/index_shards/<sex>/<equip>/index.json`
- slice `hist/*.bin` and `heat/*.bin`
- `data/<version>/trends.json`
- optional `meta/*.json` when legacy verbose publish mode is enabled

Single-lift slices use `Kg`. Total slices can use `Kg`, `Dots`, `Wilks`, or `GL`.

## Run Locally

Windows PowerShell:

```powershell
cd app
pwsh -File .\sync-data.ps1
trunk serve --open
```

Linux/macOS:

```bash
cd app
rm -rf data
mkdir -p data
cp -a ../data/. data/
trunk serve --open
```

If the UI is showing an older dataset version than root `data/latest.json`, resync `app/data/`.

## Build

Local release build:

```bash
cd app
trunk build --release
```

GitHub Pages build:

```bash
cd app
trunk build --release --dist ../docs --public-url "/<repo-name>/"
```

## Static Assets

- `index.html` defines the metadata, boot shell, and Trunk copy rules
- `assets/` contains CSS and social/share assets
- `landing/` contains static guide and SEO pages
- `robots.txt` and `sitemap.xml` are copied into the final site output

Generated output lands in `app/dist/` for local Trunk builds and `../docs/` for the Pages build.
