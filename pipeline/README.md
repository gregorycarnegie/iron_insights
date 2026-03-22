# Pipeline

The pipeline crate turns the OpenPowerlifting CSV ZIP into versioned binary bundles consumed by the frontend.

## 1) Download And Convert

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD \
  --dataset-revision "optional-revision"
```

Defaults:

- `--zip-url https://openpowerlifting.gitlab.io/opl-csv/files/openpowerlifting-latest.zip`
- `--temp-dir pipeline/tmp`
- `--output-dir pipeline/output`
- `--dataset-version auto`

Outputs:

- `pipeline/output/openpowerlifting-latest.parquet`
- `pipeline/output/build_metadata.json`

On success, the temporary ZIP and extracted CSV are removed and only the canonical Parquet plus metadata remain.

## 2) Build Aggregate Record Tables

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates
```

Defaults:

- `--input-parquet pipeline/output/openpowerlifting-latest.parquet`
- `--output-dir pipeline/output/records`

This stage filters to valid sanctioned meet results, derives IPF-style weight-class and age buckets, and writes per-lifter best-lift tables.

Outputs:

- `pipeline/output/records/all/squat.parquet`
- `pipeline/output/records/all/bench.parquet`
- `pipeline/output/records/all/deadlift.parquet`
- `pipeline/output/records/all/total.parquet`
- `pipeline/output/records/tested/squat.parquet`
- `pipeline/output/records/tested/bench.parquet`
- `pipeline/output/records/tested/deadlift.parquet`
- `pipeline/output/records/tested/total.parquet`

## 3) Publish Versioned `data/`

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --version vYYYY-MM-DD \
  --keep-versions 4
```

Useful options:

- `--records-dir pipeline/output/records`
- `--build-metadata-path pipeline/output/build_metadata.json`
- `--write-meta-files true` to restore legacy per-slice JSON metadata

Outputs:

- `data/vYYYY-MM-DD/index.json`
- `data/vYYYY-MM-DD/index_shards/<sex>/<equip>/index.json`
- `data/vYYYY-MM-DD/hist/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.bin`
- `data/vYYYY-MM-DD/heat/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.bin`
- `data/vYYYY-MM-DD/trends.json`
- optional `data/vYYYY-MM-DD/meta/<sex>/<equip>/<wc>/<age>/<tested>/<metric>/<lift>.json`
- `data/latest.json`

Metric behavior:

- squat, bench, and deadlift publish only `Kg`
- total publishes `Kg`, `Dots`, `Wilks`, and `GL`
- shard indexes embed summary totals, min, and max for each slice by default

Retention behavior:

- `keep_versions` prunes older directories whose names match `vYYYY-MM-DD`

## Data Model Notes

- Histograms use a compact binary format with 2.5 kg base bins for `Kg` metrics.
- Heatmaps store lift metric on the x-axis and bodyweight on the y-axis.
- Total-score metrics (`Dots`, `Wilks`, `GL`) use 2.5-point base bins.
- `trends.json` stores yearly cohort sizes plus p50 and p90 thresholds per series.

## Typical End-To-End Run

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- --dataset-version vYYYY-MM-DD
cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates
cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- --data-dir data --version vYYYY-MM-DD --keep-versions 4
```

After publish, sync root `data/` into `app/data/` before running or building the frontend.
