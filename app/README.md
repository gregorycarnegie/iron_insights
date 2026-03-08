# App (Leptos CSR)

## Run locally

```bash
cd app
pwsh -File .\sync-data.ps1
trunk serve --open
```

This app fetches:
- `./data/latest.json`
- `./data/<version>/index.json`
- shard index entries plus referenced `hist/*.bin` and `heat/*.bin` files
- `meta/*.json` only as a legacy fallback when summary is not embedded

## Build for Pages

```bash
cd app
trunk build --release --dist ../docs
```

Then serve `docs/` via GitHub Pages.
