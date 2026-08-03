use serde::{Deserialize, Serialize};

pub mod aggregate;
#[cfg(test)]
mod chain_tests;
pub mod download;
pub mod publish_data;
pub mod seo_geo;
#[cfg(test)]
mod test_support;

pub const DEFAULT_ZIP_URL: &str =
    "https://openpowerlifting.gitlab.io/opl-csv/files/openpowerlifting-latest.zip";

/// Provenance stamp written by stage 1; stage 3 reads it for the dataset version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub built_at_utc: String,
    pub dataset_version: String,
    pub dataset_revision: Option<String>,
    pub source_zip_url: String,
}
