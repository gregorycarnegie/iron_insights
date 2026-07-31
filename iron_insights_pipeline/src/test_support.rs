//! Fixtures shared by the stage tests.
//!
//! Stages 3 and 4 both need a published tree to work against, and building one
//! means running stage 3 for real. Keeping the fixture here means both test
//! modules exercise the same publish path rather than two hand-rolled
//! approximations of it.

use std::path::{Path, PathBuf};

use polars::prelude::*;
use tempfile::TempDir;

use crate::publish_data::{PublishArgs, publish};

/// One row of the per-lifter table stage 2 hands to stage 3.
#[derive(Clone)]
pub(crate) struct RecordRow {
    pub(crate) sex: &'static str,
    pub(crate) equipment: &'static str,
    pub(crate) weight_class: &'static str,
    pub(crate) age_class: &'static str,
    pub(crate) best_lift: f32,
    pub(crate) bodyweight: f32,
    pub(crate) date: &'static str,
}

impl RecordRow {
    pub(crate) fn new(sex: &'static str, weight_class: &'static str, best_lift: f32, bodyweight: f32) -> Self {
        Self {
            sex,
            equipment: "Raw",
            weight_class,
            age_class: "24-34",
            best_lift,
            bodyweight,
            date: "2026-03-07",
        }
    }
}

pub(crate) fn write_records_parquet(path: &Path, rows: &[RecordRow]) {
    std::fs::create_dir_all(path.parent().expect("records path has a parent"))
        .expect("create records dir");

    let mut df = df!(
        "Sex" => rows.iter().map(|r| r.sex).collect::<Vec<_>>(),
        "Equipment" => rows.iter().map(|r| r.equipment).collect::<Vec<_>>(),
        "IpfWeightClass" => rows.iter().map(|r| r.weight_class).collect::<Vec<_>>(),
        "AgeClassBucket" => rows.iter().map(|r| r.age_class).collect::<Vec<_>>(),
        "best_lift" => rows.iter().map(|r| r.best_lift).collect::<Vec<_>>(),
        "bodyweight_at_best" => rows.iter().map(|r| r.bodyweight).collect::<Vec<_>>(),
        "date_at_best" => rows.iter().map(|r| r.date).collect::<Vec<_>>(),
    )
    .expect("records frame should build");

    let mut file = std::fs::File::create(path).expect("create records parquet");
    ParquetWriter::new(&mut file)
        .finish(&mut df)
        .expect("write records parquet");
}

/// Runs a real stage 3 publish over `lifts` and returns the temp dir (kept alive
/// by the caller) plus the data dir it published into.
pub(crate) fn publish_tree(version: &str, lifts: &[(&str, Vec<RecordRow>)]) -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let records_dir = temp.path().join("records");
    let data_dir = temp.path().join("data");

    for (lift, rows) in lifts {
        write_records_parquet(&records_dir.join("all").join(format!("{lift}.parquet")), rows);
    }

    publish(&PublishArgs {
        records_dir,
        build_metadata_path: temp.path().join("missing_metadata.json"),
        data_dir: data_dir.clone(),
        version: Some(version.to_string()),
        keep_versions: 4,
    })
    .expect("publish should succeed");

    (temp, data_dir)
}

/// A cohort wide enough that its payloads exceed `INLINE_THRESHOLD` and get
/// written as real `.bin` files.
///
/// Stage 4 reads figures off disk by path, so a fixture small enough to be
/// inlined into the index would leave it with nothing to find — and the test
/// would pass while asserting only the degraded, statless path.
pub(crate) fn wide_cohort(sex: &'static str, weight_class: &'static str, base_lift: f32) -> Vec<RecordRow> {
    (0..160u16)
        .map(|i| {
            RecordRow::new(
                sex,
                weight_class,
                base_lift + f32::from(i % 40) * 2.5,
                60.0 + f32::from(i % 25),
            )
        })
        .collect()
}
