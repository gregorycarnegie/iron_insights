use chrono::Utc;
use serde::{Deserialize, Serialize};

pub const DEFAULT_ZIP_URL: &str =
    "https://openpowerlifting.gitlab.io/opl-csv/files/openpowerlifting-latest.zip";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub built_at_utc: String,
    pub dataset_version: String,
    pub dataset_revision: Option<String>,
    pub source_zip_url: String,
    pub source_zip_path: String,
    pub source_csv_path: String,
    pub canonical_parquet_path: String,
}

impl BuildMetadata {
    pub fn new(
        dataset_version: String,
        dataset_revision: Option<String>,
        source_zip_url: String,
        source_zip_path: String,
        source_csv_path: String,
        canonical_parquet_path: String,
    ) -> Self {
        Self {
            built_at_utc: Utc::now().to_rfc3339(),
            dataset_version,
            dataset_revision,
            source_zip_url,
            source_zip_path,
            source_csv_path,
            canonical_parquet_path,
        }
    }
}
