use std::fs::{self, File};
use std::io::{self, BufWriter, copy};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use pipeline::{BuildMetadata, DEFAULT_ZIP_URL};
use polars::prelude::{CsvReadOptions, ParquetWriter, SerReader};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = DEFAULT_ZIP_URL)]
    zip_url: String,

    #[arg(long, default_value = "pipeline/tmp")]
    temp_dir: PathBuf,

    #[arg(long, default_value = "pipeline/output")]
    output_dir: PathBuf,

    #[arg(long, default_value = "v0000-00-00")]
    dataset_version: String,

    #[arg(long)]
    dataset_revision: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    fs::create_dir_all(&args.temp_dir)
        .with_context(|| format!("failed to create temp dir {}", args.temp_dir.display()))?;
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("failed to create output dir {}", args.output_dir.display()))?;

    let zip_path = args.temp_dir.join("openpowerlifting-latest.zip");
    let csv_path = args.temp_dir.join("openpowerlifting-latest.csv");
    let parquet_path = args.output_dir.join("openpowerlifting-latest.parquet");
    let metadata_path = args.output_dir.join("build_metadata.json");

    download_zip(&args.zip_url, &zip_path)?;
    extract_first_csv(&zip_path, &csv_path)?;
    convert_csv_to_parquet(&csv_path, &parquet_path)?;

    let metadata = BuildMetadata::new(
        args.dataset_version,
        args.dataset_revision,
        args.zip_url,
        zip_path.display().to_string(),
        csv_path.display().to_string(),
        parquet_path.display().to_string(),
    );

    fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?)
        .with_context(|| format!("failed writing metadata {}", metadata_path.display()))?;

    // Keep only canonical Parquet + metadata after successful conversion.
    fs::remove_file(&zip_path)
        .with_context(|| format!("failed removing zip {}", zip_path.display()))?;
    fs::remove_file(&csv_path)
        .with_context(|| format!("failed removing csv {}", csv_path.display()))?;

    println!("Wrote: {}", parquet_path.display());
    println!("Wrote: {}", metadata_path.display());

    Ok(())
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
    let mut df = CsvReadOptions::default()
        .with_has_header(true)
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
