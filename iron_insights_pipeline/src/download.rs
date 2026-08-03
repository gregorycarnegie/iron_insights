//! Stage 1: download the OpenPowerlifting CSV dump and convert it to a
//! canonical Parquet file, stamping a [`BuildMetadata`] snapshot alongside it.

use std::{
    fs::{self, File},
    io::{self, BufWriter, copy},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use clap::Parser;
use polars::prelude::{
    CsvReadOptions, DataType, Field, ParquetWriter, Schema, SchemaRef, SerReader,
};

use crate::{BuildMetadata, DEFAULT_ZIP_URL};

#[cfg(test)]
pub(crate) use Args as DownloadArgs;

/// Crate-visible so the stage-chain test can drive a conversion; the binary
/// still reaches it only through [`run`].
#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[arg(long, default_value = DEFAULT_ZIP_URL)]
    pub(crate) zip_url: String,

    #[arg(long, default_value = "iron_insights_pipeline/tmp")]
    pub(crate) temp_dir: PathBuf,

    #[arg(long, default_value = "iron_insights_pipeline/output")]
    pub(crate) output_dir: PathBuf,

    #[arg(long, default_value = "auto")]
    pub(crate) dataset_version: String,

    #[arg(long)]
    pub(crate) dataset_revision: Option<String>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    fs::create_dir_all(&args.temp_dir)
        .with_context(|| format!("failed to create temp dir {}", args.temp_dir.display()))?;
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("failed to create output dir {}", args.output_dir.display()))?;

    let zip_path = args.temp_dir.join("openpowerlifting-latest.zip");

    download_zip(&args.zip_url, &zip_path)?;
    convert_downloaded_zip(&args, &zip_path)
}

/// Everything after the network fetch: zip in, canonical parquet plus metadata
/// out, temporaries removed. Split from [`run`] so it can be tested against a
/// hand-built zip without reaching for the network.
pub(crate) fn convert_downloaded_zip(args: &Args, zip_path: &Path) -> Result<()> {
    let csv_path = args.temp_dir.join("openpowerlifting-latest.csv");
    let parquet_path = args.output_dir.join("openpowerlifting-latest.parquet");
    let metadata_path = args.output_dir.join("build_metadata.json");

    extract_first_csv(zip_path, &csv_path)?;
    convert_csv_to_parquet(&csv_path, &parquet_path)?;

    let metadata = BuildMetadata {
        built_at_utc: Utc::now().to_rfc3339(),
        dataset_version: resolve_dataset_version(&args.dataset_version),
        dataset_revision: args.dataset_revision.clone(),
        source_zip_url: args.zip_url.clone(),
    };

    fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?)
        .with_context(|| format!("failed writing metadata {}", metadata_path.display()))?;

    // Keep only canonical Parquet + metadata after successful conversion.
    fs::remove_file(zip_path)
        .with_context(|| format!("failed removing zip {}", zip_path.display()))?;
    fs::remove_file(&csv_path)
        .with_context(|| format!("failed removing csv {}", csv_path.display()))?;

    println!("Wrote: {}", parquet_path.display());
    println!("Wrote: {}", metadata_path.display());

    Ok(())
}

fn resolve_dataset_version(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("auto")
        || trimmed == "v0000-00-00"
        || trimmed == "vYYYY-MM-DD"
    {
        return format!("v{}", Utc::now().format("%Y-%m-%d"));
    }
    if trimmed.starts_with('v') {
        trimmed.to_string()
    } else {
        format!("v{trimmed}")
    }
}

fn download_zip(zip_url: &str, zip_path: &Path) -> Result<()> {
    let response = reqwest::blocking::get(zip_url)
        .with_context(|| format!("failed requesting {zip_url}"))?
        .error_for_status()
        .with_context(|| format!("server returned error for {zip_url}"))?;

    let mut writer = BufWriter::new(
        File::create(zip_path)
            .with_context(|| format!("failed creating zip file {}", zip_path.display()))?,
    );
    let mut reader = io::BufReader::new(response);
    copy(&mut reader, &mut writer)
        .with_context(|| format!("failed writing zip file {}", zip_path.display()))?;

    Ok(())
}

fn extract_first_csv(zip_path: &Path, csv_out_path: &Path) -> Result<()> {
    let reader =
        File::open(zip_path).with_context(|| format!("failed opening {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(reader)
        .with_context(|| format!("failed reading zip archive {}", zip_path.display()))?;

    let csv_index = (0..archive.len())
        .find(|i| {
            archive
                .by_index(*i)
                .ok()
                .and_then(|f| f.enclosed_name().map(|p| p.to_path_buf()))
                .and_then(|p| p.extension().map(|e| e.eq_ignore_ascii_case("csv")))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("no CSV file found in {}", zip_path.display()))?;

    let mut csv_file = archive
        .by_index(csv_index)
        .with_context(|| format!("failed reading CSV entry {csv_index} from zip"))?;

    let mut out = File::create(csv_out_path)
        .with_context(|| format!("failed creating {}", csv_out_path.display()))?;
    copy(&mut csv_file, &mut out)
        .with_context(|| format!("failed extracting CSV to {}", csv_out_path.display()))?;

    Ok(())
}

fn convert_csv_to_parquet(csv_path: &Path, parquet_path: &Path) -> Result<()> {
    let schema_overwrite = opl_numeric_schema_overrides();
    let mut df = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(Some(10_000))
        .with_schema_overwrite(Some(schema_overwrite))
        .try_into_reader_with_file_path(Some(csv_path.to_path_buf()))
        .with_context(|| format!("failed opening csv {}", csv_path.display()))?
        .finish()
        .with_context(|| format!("failed parsing csv {}", csv_path.display()))?;

    let mut out = File::create(parquet_path)
        .with_context(|| format!("failed creating {}", parquet_path.display()))?;
    ParquetWriter::new(&mut out)
        .finish(&mut df)
        .with_context(|| format!("failed writing parquet {}", parquet_path.display()))?;

    Ok(())
}

fn opl_numeric_schema_overrides() -> SchemaRef {
    let numeric_cols = [
        "BodyweightKg",
        "Squat1Kg",
        "Squat2Kg",
        "Squat3Kg",
        "Squat4Kg",
        "Best3SquatKg",
        "Bench1Kg",
        "Bench2Kg",
        "Bench3Kg",
        "Bench4Kg",
        "Best3BenchKg",
        "Deadlift1Kg",
        "Deadlift2Kg",
        "Deadlift3Kg",
        "Deadlift4Kg",
        "Best3DeadliftKg",
        "TotalKg",
        "Wilks",
        "McCulloch",
        "Glossbrenner",
        "IPFPoints",
        "Dots",
        "Goodlift",
    ];

    Arc::new(Schema::from_iter(
        numeric_cols
            .into_iter()
            .map(|name| Field::new(name.into(), DataType::Float32)),
    ))
}

#[cfg(test)]
mod tests {
    use super::{Args, DEFAULT_ZIP_URL, convert_downloaded_zip, resolve_dataset_version};
    use chrono::Utc;
    use polars::prelude::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// The columns `opl_numeric_schema_overrides` forces to `Float32`, plus the
    /// string columns stage 2 filters on. `Squat4Kg` is left empty on every row
    /// on purpose: without the override it would infer as null/string, and stage
    /// 2's numeric comparisons would quietly stop matching.
    const CSV: &str = "\
Name,Sex,Event,Equipment,Tested,Sanctioned,Place,Date,Age,BodyweightKg,Squat1Kg,Squat2Kg,Squat3Kg,Squat4Kg,Best3SquatKg,Bench1Kg,Bench2Kg,Bench3Kg,Bench4Kg,Best3BenchKg,Deadlift1Kg,Deadlift2Kg,Deadlift3Kg,Deadlift4Kg,Best3DeadliftKg,TotalKg,Wilks,McCulloch,Glossbrenner,IPFPoints,Dots,Goodlift\n\
Ada Lovelace,F,SBD,Raw,Yes,Yes,1,2026-03-07,30,63,100,110,120,,120,60,65,70,,70,130,140,150,,150,340,400.1,400.1,395.0,410.2,405.5,80.1\n\
Alan Turing,M,SBD,Raw,Yes,Yes,1,2026-04-01,35,92,200,210,220,,220,130,135,140,,140,240,250,260,,260,620,410.5,412.0,405.0,420.0,415.0,82.5\n";

    fn zip_containing(dir: &std::path::Path, entries: &[(&str, &str)]) -> std::path::PathBuf {
        let zip_path = dir.join("openpowerlifting-latest.zip");
        let file = std::fs::File::create(&zip_path).expect("create zip");
        let mut writer = ::zip::ZipWriter::new(file);
        let options = ::zip::write::SimpleFileOptions::default()
            .compression_method(::zip::CompressionMethod::Stored);

        for (name, body) in entries {
            writer.start_file(*name, options).expect("start zip entry");
            writer.write_all(body.as_bytes()).expect("write zip entry");
        }
        writer.finish().expect("finish zip");
        zip_path
    }

    /// Runs everything downstream of the network fetch against a synthetic zip.
    fn run_stage_1(entries: &[(&str, &str)]) -> (TempDir, std::path::PathBuf) {
        let temp = TempDir::new().expect("temp dir");
        let temp_dir = temp.path().join("tmp");
        let output_dir = temp.path().join("output");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");

        let zip_path = zip_containing(&temp_dir, entries);

        convert_downloaded_zip(
            &Args {
                zip_url: DEFAULT_ZIP_URL.to_string(),
                temp_dir,
                output_dir: output_dir.clone(),
                dataset_version: "2026-07-31".to_string(),
                dataset_revision: Some("abc123".to_string()),
            },
            &zip_path,
        )
        .expect("stage 1 conversion should succeed");

        (temp, output_dir)
    }

    #[test]
    fn converts_the_zipped_csv_into_a_canonical_parquet() {
        let (_temp, output_dir) = run_stage_1(&[
            // A non-CSV entry first, to prove the CSV is selected by extension
            // rather than by being the first thing in the archive.
            ("README.txt", "not the data"),
            ("openpowerlifting-2026-07-31.csv", CSV),
        ]);

        let parquet = output_dir.join("openpowerlifting-latest.parquet");
        assert!(parquet.is_file(), "no parquet written");

        let df = LazyFrame::scan_parquet(
            parquet.to_string_lossy().as_ref().into(),
            ScanArgsParquet::default(),
        )
        .expect("scan parquet")
        .collect()
        .expect("collect parquet");

        assert_eq!(df.height(), 2, "both rows should survive conversion");

        // The schema override is the whole point of this stage: these must be
        // numeric even when a column is entirely empty, or stage 2's `> 0`
        // filters silently match nothing.
        for column in ["BodyweightKg", "Best3SquatKg", "TotalKg", "Squat4Kg"] {
            assert_eq!(
                df.column(column).expect(column).dtype(),
                &DataType::Float32,
                "{column} should be Float32"
            );
        }
    }

    #[test]
    fn stamps_metadata_and_clears_the_temporaries() {
        let (temp, output_dir) = run_stage_1(&[("data.csv", CSV)]);

        let metadata: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output_dir.join("build_metadata.json")).expect("read metadata"),
        )
        .expect("metadata is valid json");

        // Stage 3 reads exactly these two fields to pick the published version.
        assert_eq!(metadata["dataset_version"], "v2026-07-31");
        assert_eq!(metadata["dataset_revision"], "abc123");
        assert!(
            metadata["built_at_utc"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
            "built_at_utc should be stamped"
        );

        // The zip and extracted CSV are large and disposable; leaving them
        // behind is what pushed earlier runs near the runner's disk limit.
        let tmp_dir = temp.path().join("tmp");
        assert!(!tmp_dir.join("openpowerlifting-latest.zip").exists());
        assert!(!tmp_dir.join("openpowerlifting-latest.csv").exists());
    }

    #[test]
    fn fails_when_the_archive_has_no_csv() {
        let temp = TempDir::new().expect("temp dir");
        let temp_dir = temp.path().join("tmp");
        let output_dir = temp.path().join("output");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");

        let zip_path = zip_containing(&temp_dir, &[("README.txt", "no data here")]);

        let result = convert_downloaded_zip(
            &Args {
                zip_url: DEFAULT_ZIP_URL.to_string(),
                temp_dir,
                output_dir,
                dataset_version: "auto".to_string(),
                dataset_revision: None,
            },
            &zip_path,
        );

        // A silent success here would publish an empty dataset over a good one.
        assert!(result.is_err(), "a CSV-less archive must not succeed");
    }

    #[test]
    fn resolve_dataset_version_handles_auto_and_placeholders() {
        let today = format!("v{}", Utc::now().format("%Y-%m-%d"));
        assert_eq!(resolve_dataset_version("auto"), today);
        assert_eq!(resolve_dataset_version("AUTO"), today);
        assert_eq!(resolve_dataset_version(""), today);
        assert_eq!(resolve_dataset_version("v0000-00-00"), today);
        assert_eq!(resolve_dataset_version("vYYYY-MM-DD"), today);
    }

    #[test]
    fn resolve_dataset_version_normalizes_prefix() {
        assert_eq!(resolve_dataset_version("2026-06-20"), "v2026-06-20");
        assert_eq!(resolve_dataset_version("v2026-06-20"), "v2026-06-20");
        assert_eq!(resolve_dataset_version("  2026-06-20  "), "v2026-06-20");
    }
}
