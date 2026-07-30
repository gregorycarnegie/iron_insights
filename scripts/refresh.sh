#!/usr/bin/env bash
# Runs the full publish pipeline. Version defaults to today (vYYYY-MM-DD).
#
#   ./scripts/refresh.sh              # today
#   ./scripts/refresh.sh v2026-07-30  # explicit version
set -euo pipefail

VERSION="${1:-v$(date +%F)}"
[[ "$VERSION" =~ ^v[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || {
  echo "[refresh] ERROR: version must look like vYYYY-MM-DD, got: $VERSION" >&2
  exit 1
}

cd "$(dirname "$0")/.."
MANIFEST=iron_insights_pipeline/Cargo.toml

stage() {
  echo "[refresh] $1"
  shift
  cargo run --release --manifest-path "$MANIFEST" --bin "$@"
}

stage "01_download ($VERSION)"   01_download   -- --dataset-version "$VERSION"
stage "02_build_aggregates"      02_build_aggregates
stage "03_publish_data"          03_publish_data -- --data-dir data --version "$VERSION" --keep-versions 2
stage "04_seo_geo"               04_seo_geo    -- --data-dir data --web-dir iron_insights_web

echo "[refresh] published $VERSION — verify with ./scripts/qa.sh data docs"
