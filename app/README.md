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
- referenced `hist/*.bin` and `heat/*.bin` files

## Build for Pages

```bash
cd app
trunk build --release --dist ../docs
```

Then serve `docs/` via GitHub Pages.
