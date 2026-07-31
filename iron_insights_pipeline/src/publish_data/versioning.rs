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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn version_dirs(names: &[&str]) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        for name in names {
            fs::create_dir_all(temp.path().join(name)).expect("create version dir");
        }
        temp
    }

    fn remaining(temp: &TempDir) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(temp.path())
            .expect("read temp dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn version_dir_names_must_match_the_exact_shape() {
        assert!(is_version_dir_name("v2026-07-31"));

        // Each conjunct has to be independently necessary, or a directory that
        // is not a published version could be selected for deletion.
        for bad in [
            "2026-07-31",   // no leading v
            "v2026-07-3",   // too short
            "v2026-07-311", // too long
            "vX026-07-31",  // non-digit in the year
            "v2026-0X-31",  // non-digit in the month
            "v2026-07-3X",  // non-digit in the day
            "v2026_07-31",  // wrong first separator
            "v2026-07_31",  // wrong second separator
            "",
        ] {
            assert!(!is_version_dir_name(bad), "{bad} should be rejected");
        }
    }

    #[test]
    fn placeholder_versions_are_not_treated_as_real() {
        assert!(is_valid_effective_version("v2026-07-31"));
        // Stage 1 emits these when it has nothing better; publishing under them
        // would create a directory that never gets superseded.
        assert!(!is_valid_effective_version("vYYYY-MM-DD"));
        assert!(!is_valid_effective_version("v0000-00-00"));
        assert!(!is_valid_effective_version("nonsense"));
    }

    #[test]
    fn resolve_version_prefers_cli_then_metadata_then_today() {
        let meta = |version: &str| BuildMetadata {
            built_at_utc: "2026-07-31T00:00:00Z".to_string(),
            dataset_version: version.to_string(),
            dataset_revision: None,
            source_zip_url: "https://example.test/data.zip".to_string(),
        };

        // CLI wins outright, and gets the `v` prefix normalised on.
        assert_eq!(
            resolve_version(Some("2026-01-02".into()), Some(&meta("v2026-07-31"))),
            "v2026-01-02"
        );
        // Otherwise a usable metadata version is taken as-is.
        assert_eq!(
            resolve_version(None, Some(&meta("2026-07-31"))),
            "v2026-07-31"
        );
        // A placeholder in metadata must fall through to today rather than be
        // published verbatim.
        let today = format!("v{}", Utc::now().format("%Y-%m-%d"));
        assert_eq!(resolve_version(None, Some(&meta("vYYYY-MM-DD"))), today);
        assert_eq!(resolve_version(None, None), today);
    }

    #[test]
    fn build_metadata_is_read_back_when_present() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("build_metadata.json");

        assert!(
            read_optional_build_metadata(&path)
                .expect("missing file is not an error")
                .is_none(),
            "a missing metadata file should be None, not an error"
        );

        fs::write(
            &path,
            br#"{"built_at_utc":"2026-07-31T00:00:00Z","dataset_version":"v2026-07-31","dataset_revision":"abc123","source_zip_url":"https://example.test/data.zip"}"#,
        )
        .expect("write metadata");

        let meta = read_optional_build_metadata(&path)
            .expect("valid metadata should parse")
            .expect("metadata should be present");
        assert_eq!(meta.dataset_version, "v2026-07-31");
        assert_eq!(meta.dataset_revision.as_deref(), Some("abc123"));
    }

    #[test]
    fn prune_removes_the_oldest_and_keeps_exactly_keep_versions() {
        // Five versions keeping two removes three. Two versions keeping one
        // would not distinguish `len - keep` from `len / keep`.
        let temp = version_dirs(&[
            "v2026-07-27",
            "v2026-07-28",
            "v2026-07-29",
            "v2026-07-30",
            "v2026-07-31",
        ]);

        prune_old_versions(temp.path(), 2).expect("prune should succeed");

        assert_eq!(remaining(&temp), ["v2026-07-30", "v2026-07-31"]);
    }

    #[test]
    fn prune_leaves_everything_that_is_not_a_version_directory() {
        let temp = version_dirs(&["v2026-07-30", "v2026-07-31", "scratch", "notes"]);
        fs::write(temp.path().join("latest.json"), b"{}").expect("write latest");

        prune_old_versions(temp.path(), 1).expect("prune should succeed");

        // Only the older version goes; `latest.json` and unrelated directories
        // share the data dir and must survive.
        assert_eq!(
            remaining(&temp),
            ["latest.json", "notes", "scratch", "v2026-07-31"]
        );
    }

    #[test]
    fn prune_is_a_no_op_when_there_is_nothing_to_remove() {
        let temp = version_dirs(&["v2026-07-30", "v2026-07-31"]);

        prune_old_versions(temp.path(), 4).expect("prune should succeed");

        assert_eq!(remaining(&temp), ["v2026-07-30", "v2026-07-31"]);
    }
}
