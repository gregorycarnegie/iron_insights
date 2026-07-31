use super::{
    accumulation::{AccumulationRow, MetricPublisher},
    constants::{BW_BIN_BASE_KG, LIFT_BIN_BASE_KG},
    histogram::{build_combined_bytes, build_heatmap, build_histogram},
    metric::Metric,
    trends::{parse_year_bucket, quantile_sorted},
};
use iron_insights_core::{dots_points, goodlift_points, parse_combined_bin, wilks_points};
use proptest::prelude::*;
use std::collections::BTreeMap;

fn generated_lift_values() -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(0.0f32..1000.0, 1..96)
}

fn generated_heat_points() -> impl Strategy<Value = Vec<(f32, f32)>> {
    prop::collection::vec((0.0f32..1000.0, 0.0f32..250.0), 1..96)
}

#[test]
fn dots_points_increase_with_total() {
    let low = dots_points("M", 90.0, 500.0);
    let high = dots_points("M", 90.0, 600.0);
    assert!(high > low);
    assert!(low > 0.0);
}

#[test]
fn wilks_points_increase_with_total() {
    let low = wilks_points("F", 63.0, 350.0);
    let high = wilks_points("F", 63.0, 420.0);
    assert!(high > low);
    assert!(low > 0.0);
}

#[test]
fn goodlift_points_differs_by_equipment() {
    let raw = goodlift_points("M", "Raw", 90.0, 700.0);
    let equipped = goodlift_points("M", "Single-ply", 90.0, 700.0);
    assert!(raw.is_finite());
    assert!(equipped.is_finite());
    assert_ne!(raw, equipped);
}

/// A corrupt Wilks score (bodyweight typo) must not reach any accumulator —
/// heatmap and histogram axes are sized from observed min/max, so one such row
/// stretched a published grid to 112,513 columns / 90 MB.
#[test]
fn accumulate_row_rejects_implausible_values() {
    let mut pubr = MetricPublisher::new(Metric::Wilks, "tested", "total");
    let mut trends = BTreeMap::new();
    let row = |x: f32, bw: Option<f32>| AccumulationRow {
        sex: "F",
        equipment: "Raw",
        weight_class: "63",
        age_class: "24-34",
        year: Some(2024),
        x_value: x,
        valid_bw: bw,
    };

    pubr.accumulate_row(row(281_280.0, Some(63.0)), &mut trends); // corrupt score
    pubr.accumulate_row(row(f32::NAN, Some(63.0)), &mut trends);
    pubr.accumulate_row(row(0.0, Some(63.0)), &mut trends);
    assert!(pubr.slices.is_empty(), "implausible rows must be dropped");
    assert!(trends.is_empty(), "implausible rows must not reach trends");

    pubr.accumulate_row(row(450.0, Some(2000.0)), &mut trends); // good score, bad bw
    let acc = pubr
        .slices
        .get(&("F", "Raw", "63", "24-34"))
        .expect("valid row should accumulate");
    assert_eq!(acc.lift_values, vec![450.0]);
    assert!(
        acc.heat_points.is_empty(),
        "implausible bodyweight must not size the heatmap"
    );

    pubr.accumulate_row(row(450.0, Some(63.0)), &mut trends);
    let acc = pubr.slices.get(&("F", "Raw", "63", "24-34")).unwrap();
    assert_eq!(acc.heat_points, vec![(450.0, 63.0)]);
}

#[test]
fn build_histogram_uses_expected_edges_and_total() {
    let values = vec![100.0, 101.0, 102.4, 104.9, 105.0];
    let hist = build_histogram(&values, LIFT_BIN_BASE_KG).expect("histogram should build");

    assert_eq!(hist.min, 100.0);
    assert_eq!(hist.max, 107.5);
    assert_eq!(hist.counts, vec![3, 1, 1]);
    assert_eq!(hist.total, 5);
    assert_eq!(
        hist.counts.iter().copied().map(u64::from).sum::<u64>(),
        hist.total
    );
}

#[test]
fn build_heatmap_empty_is_zero_shape() {
    let heat = build_heatmap(&[], LIFT_BIN_BASE_KG, BW_BIN_BASE_KG).expect("heatmap should build");
    assert_eq!(heat.width, 0);
    assert_eq!(heat.height, 0);
    assert!(heat.grid.is_empty());
}

#[test]
fn build_heatmap_bins_points_and_preserves_total() {
    let points = vec![
        (100.0, 80.0),
        (101.0, 80.2),
        (102.4, 80.9),
        (104.9, 81.1),
        (105.0, 81.9),
    ];
    let heat =
        build_heatmap(&points, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG).expect("heatmap should build");

    assert_eq!(heat.min_x, 100.0);
    assert_eq!(heat.max_x, 107.5);
    assert_eq!(heat.min_y, 80.0);
    assert_eq!(heat.max_y, 82.0);
    assert_eq!(heat.width, 3);
    assert_eq!(heat.height, 2);
    assert_eq!(
        heat.grid.iter().copied().map(u64::from).sum::<u64>(),
        points.len() as u64
    );
}

proptest! {
    #[test]
    fn build_histogram_preserves_total_for_generated_values(values in generated_lift_values()) {
        let hist = build_histogram(&values, LIFT_BIN_BASE_KG).expect("histogram should build");
        let counted = hist.counts.iter().copied().map(u64::from).sum::<u64>();

        prop_assert_eq!(hist.total, values.len() as u64);
        prop_assert_eq!(counted, hist.total);
        prop_assert!(hist.min <= values.iter().copied().fold(f32::INFINITY, f32::min));
        prop_assert!(hist.max > values.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    }

    #[test]
    fn build_heatmap_preserves_total_and_shape_for_generated_points(
        points in generated_heat_points(),
    ) {
        let heat = build_heatmap(&points, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG)
            .expect("heatmap should build");
        let counted = heat.grid.iter().copied().map(u64::from).sum::<u64>();

        prop_assert_eq!(counted, points.len() as u64);
        prop_assert_eq!(heat.grid.len(), heat.width * heat.height);
        prop_assert!(heat.min_x <= points.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min));
        prop_assert!(heat.max_x > points.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max));
        prop_assert!(heat.min_y <= points.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min));
        prop_assert!(heat.max_y > points.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max));
    }

    #[test]
    fn combined_payload_round_trips_generated_histogram_and_heatmap(
        values in generated_lift_values(),
        points in generated_heat_points(),
    ) {
        let hist = build_histogram(&values, LIFT_BIN_BASE_KG).expect("histogram should build");
        let heat = build_heatmap(&points, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG)
            .expect("heatmap should build");
        let bytes = build_combined_bytes(&hist, &heat, LIFT_BIN_BASE_KG, BW_BIN_BASE_KG);
        let (parsed_hist, parsed_heat) = parse_combined_bin(&bytes).expect("payload should parse");

        prop_assert_eq!(parsed_hist.counts, hist.counts);
        prop_assert_eq!(parsed_heat.grid, heat.grid);
        prop_assert_eq!(parsed_heat.width, heat.width);
        prop_assert_eq!(parsed_heat.height, heat.height);
    }
}

#[test]
fn parse_year_bucket_accepts_valid_dates() {
    assert_eq!(parse_year_bucket(Some("2026-03-07")), Some(2026));
    assert_eq!(parse_year_bucket(Some("1999-12-31")), Some(1999));
}

#[test]
fn parse_year_bucket_rejects_invalid_dates() {
    assert_eq!(parse_year_bucket(Some("bad")), None);
    assert_eq!(parse_year_bucket(Some("1800-01-01")), None);
    assert_eq!(parse_year_bucket(None), None);
}

#[test]
fn quantile_sorted_uses_nearest_rank_on_sorted_data() {
    let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    assert_eq!(quantile_sorted(&values, 0.0), 10.0);
    assert_eq!(quantile_sorted(&values, 0.5), 30.0);
    assert_eq!(quantile_sorted(&values, 0.9), 50.0);
}

// ===== END-TO-END PUBLISH =====
//
// Everything above tests one function. These drive the whole of stage 3 against
// a temp directory: synthetic records parquet in, published tree out, then read
// the tree back the way the web app does — raw JSON off disk, payload through
// `iron_insights_core::parse_combined_bin`. Asserting the wire format rather
// than our own serializable types is deliberate: a rename that breaks the app
// should fail here, and it would not if both sides shared a struct.

use super::{Args, publish};
use crate::test_support::{RecordRow, publish_tree};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::Value;
use std::{fs, path::Path};
use tempfile::TempDir;

/// Publishes a fixed three-lifter cohort and returns the temp dir plus data dir.
fn publish_fixture(version: &str) -> (TempDir, std::path::PathBuf) {
    // Two men and one woman, all raw, all in the same age class.
    publish_tree(
        version,
        &[(
            "squat",
            vec![
                RecordRow::new("M", "93", 200.0, 92.0),
                RecordRow::new("M", "93", 220.0, 91.0),
                RecordRow::new("F", "63", 120.0, 62.0),
            ],
        )],
    )
}

fn read_json(path: &Path) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Resolves a slice entry's payload the way the app does: inline base64 first,
/// otherwise the `.bin` file it points at.
fn payload_bytes(version_dir: &Path, entry: &Value) -> Vec<u8> {
    let inline = entry["inline"].as_str().unwrap_or_default();
    if !inline.is_empty() {
        return BASE64.decode(inline).expect("inline payload is valid base64");
    }
    let bin = entry["bin"].as_str().expect("entry has bin or inline");
    assert!(!bin.is_empty(), "entry has neither bin nor inline");
    fs::read(version_dir.join(bin)).expect("read .bin")
}

#[test]
fn publish_writes_a_readable_version_tree() {
    let (_temp, data_dir) = publish_fixture("v2026-07-31");

    let latest = read_json(&data_dir.join("latest.json"));
    assert_eq!(latest["version"], "v2026-07-31");

    let version_dir = data_dir.join("v2026-07-31");
    let index = read_json(&version_dir.join("index.json"));

    // Each lifter is counted into both their own equipment and the "All"
    // aggregate, so both shards must exist and be reachable from the root.
    for shard_key in ["sex=M|equip=Raw", "sex=M|equip=All", "sex=F|equip=Raw"] {
        let rel = index["shards"][shard_key]
            .as_str()
            .unwrap_or_else(|| panic!("root index is missing shard {shard_key}"));
        assert!(
            version_dir.join(rel).is_file(),
            "shard {shard_key} points at a missing file: {rel}"
        );
    }

    // No meta/ tree is written any more; the summary rides in the index.
    assert!(!version_dir.join("meta").exists());
}

#[test]
fn published_slices_round_trip_through_the_core_parser() {
    let (_temp, data_dir) = publish_fixture("v2026-07-31");
    let version_dir = data_dir.join("v2026-07-31");

    let index = read_json(&version_dir.join("index.json"));
    let shard_rel = index["shards"]["sex=M|equip=Raw"]
        .as_str()
        .expect("male raw shard");
    let shard = read_json(&version_dir.join(shard_rel));

    // The widest male cohort: both men, aggregated across every filter.
    let key = "sex=M|equip=Raw|wc=All|age=All Ages|tested=All|lift=S|metric=Kg";
    let entry = &shard["slices"][key];
    assert!(!entry.is_null(), "shard is missing slice {key}");

    assert_eq!(entry["summary"]["total"], 2, "both men should be counted");

    let (hist, heat) = parse_combined_bin(&payload_bytes(&version_dir, entry))
        .expect("published payload should parse with the shipped parser");

    assert_eq!(hist.total, 2);
    assert_eq!(hist.counts.iter().sum::<u32>(), 2);
    assert_eq!(heat.grid.iter().sum::<u32>(), 2, "both men have bodyweight");

    // The 200 kg and 220 kg squats must land inside the histogram's own bounds.
    assert!(hist.min <= 200.0 && hist.max >= 220.0, "{hist:?}");

    // Every key in the shard must parse with the contract the app uses.
    for raw_key in shard["slices"].as_object().expect("slices object").keys() {
        assert!(
            iron_insights_core::parse_slice_key(raw_key).is_some(),
            "app could not parse published key: {raw_key}"
        );
    }
}

#[test]
fn republishing_a_version_replaces_rather_than_merges() {
    let (temp, data_dir) = publish_fixture("v2026-07-31");
    let version_dir = data_dir.join("v2026-07-31");

    // A file the fresh index will not reference.
    let orphan = version_dir.join("bin").join("orphan.bin");
    fs::create_dir_all(orphan.parent().expect("bin dir")).expect("create bin dir");
    fs::write(&orphan, b"stale").expect("write orphan");

    publish(&Args {
        records_dir: temp.path().join("records"),
        build_metadata_path: temp.path().join("missing_metadata.json"),
        data_dir,
        version: Some("v2026-07-31".to_string()),
        keep_versions: 4,
    })
    .expect("republish should succeed");

    assert!(
        !orphan.exists(),
        "republish left an orphaned file behind: {}",
        orphan.display()
    );
}

#[test]
fn prune_keeps_only_the_newest_versions() {
    let (temp, data_dir) = publish_fixture("v2026-07-29");
    let records_dir = temp.path().join("records");

    for version in ["v2026-07-30", "v2026-07-31"] {
        publish(&Args {
            records_dir: records_dir.clone(),
            build_metadata_path: temp.path().join("missing_metadata.json"),
            data_dir: data_dir.clone(),
            version: Some(version.to_string()),
            keep_versions: 2,
        })
        .expect("publish should succeed");
    }

    assert!(!data_dir.join("v2026-07-29").exists(), "oldest should prune");
    assert!(data_dir.join("v2026-07-30").exists());
    assert!(data_dir.join("v2026-07-31").exists());
}
