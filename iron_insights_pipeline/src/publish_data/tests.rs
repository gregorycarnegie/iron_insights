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
