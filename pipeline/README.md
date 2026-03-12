# Pipeline

## Download + Convert

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 01_download -- \
  --dataset-version vYYYY-MM-DD \
  --dataset-revision "optional-revision"
```

Outputs:

- `pipeline/output/openpowerlifting-latest.parquet`
- `pipeline/output/build_metadata.json`

Temporary ZIP/CSV files are deleted after successful conversion.

## Build Aggregates

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 02_build_aggregates
```

Outputs:

- `pipeline/output/records/all/*.parquet`
- `pipeline/output/records/tested/*.parquet`

## Publish Versioned `/data`

```bash
cargo run --manifest-path pipeline/Cargo.toml --bin 03_publish_data -- \
  --data-dir data \
  --keep-versions 4
```

Outputs:

- `data/vYYYY-MM-DD/hist/.../*.bin`
- `data/vYYYY-MM-DD/heat/.../*.bin`
- `data/vYYYY-MM-DD/index.json` (root shard lookup)
- `data/vYYYY-MM-DD/index_shards/.../index.json` (slice lookup + embedded summary)
- `data/vYYYY-MM-DD/trends.json` (year-bucketed cohort size + p50/p90 thresholds)
- `data/latest.json`

Older `data/vYYYY-MM-DD` folders are pruned to the configured retention count.
