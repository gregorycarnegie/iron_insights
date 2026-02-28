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
- `data/vYYYY-MM-DD/hist/{sex}/{equip}/{tested}/{lift}.bin`
- `data/vYYYY-MM-DD/heat/{sex}/{equip}/{tested}/{lift}.bin`
- `data/vYYYY-MM-DD/meta/{sex}/{equip}/{tested}/{lift}.json`
- `data/vYYYY-MM-DD/index.json` (slice lookup table)
- `data/latest.json`

Older `data/vYYYY-MM-DD` folders are pruned to the configured retention count.
