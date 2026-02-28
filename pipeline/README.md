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
