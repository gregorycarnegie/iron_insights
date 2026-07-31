use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::BuildMetadata;

pub(super) fn read_optional_build_metadata(path: &Path) -> Result<Option<BuildMetadata>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).with_context(|| format!("failed reading {}", path.display()))?;
    let metadata: BuildMetadata = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed parsing {}", path.display()))?;
    Ok(Some(metadata))
}

pub(super) fn resolve_version(cli: Option<String>, metadata: Option<&BuildMetadata>) -> String {
    if let Some(version) = cli {
        return normalize_version(&version);
    }

    if let Some(meta) = metadata {
        let normalized = normalize_version(&meta.dataset_version);
        if is_valid_effective_version(&normalized) {
            return normalized;
        }
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    format!("v{today}")
}

fn normalize_version(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

fn is_valid_effective_version(version: &str) -> bool {
    if version == "vYYYY-MM-DD" || version == "v0000-00-00" {
        return false;
    }
    is_version_dir_name(version)
}

pub(super) fn prune_old_versions(data_dir: &Path, keep_versions: usize) -> Result<()> {
    let mut versions: Vec<PathBuf> = fs::read_dir(data_dir)
        .with_context(|| format!("failed reading {}", data_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_version_dir_name)
        })
        .collect();

    versions.sort();

    if versions.len() <= keep_versions {
        return Ok(());
    }

    let to_remove = versions.len() - keep_versions;
    for old in versions.into_iter().take(to_remove) {
        fs::remove_dir_all(&old)
            .with_context(|| format!("failed removing old version {}", old.display()))?;
    }

    Ok(())
}

fn is_version_dir_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 11
        && bytes[0] == b'v'
        && bytes[5] == b'-'
        && bytes[8] == b'-'
        && bytes[1..5].iter().all(u8::is_ascii_digit)
        && bytes[6..8].iter().all(u8::is_ascii_digit)
        && bytes[9..11].iter().all(u8::is_ascii_digit)
}
