use super::{
    BINARY_FORMAT_VERSION, COMBINED_MAGIC, HEATMAP_MAGIC, HISTOGRAM_MAGIC, HeatmapBin,
    HistogramBin, TINY_COHORT_WARNING_THRESHOLD, bodyweight_conditioned_percentile, decode_counts,
    dots_points, encode_counts, equivalent_value_for_same_percentile, goodlift_points,
    histogram_density_for_value, histogram_diagnostics, parse_combined_bin, parse_heat_bin,
    parse_hist_bin, percentile_for_value, rebin_1d, rebin_2d, value_for_percentile, wilks_points,
};
use crate::bodyfat::siri_bf_from_density;
use proptest::prelude::*;

fn push_f32(bytes: &mut Vec<u8>, v: f32) {
    bytes.extend_from_slice(&v.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, v: u32) {
    bytes.extend_from_slice(&v.to_le_bytes());
}

/// Appends a varint/zero-run count payload, as the publisher writes it.
fn push_counts(bytes: &mut Vec<u8>, counts: &[u32]) {
    bytes.extend_from_slice(&encode_counts(counts));
}

fn bounded_count_vec() -> impl Strategy<Value = Vec<u32>> {
    prop::collection::vec(0u32..10_000, 0..64)
}

fn bounded_heat_grid() -> impl Strategy<Value = (Vec<u32>, usize, usize)> {
    (1usize..12, 1usize..12).prop_flat_map(|(width, height)| {
        let cells = width * height;
        prop::collection::vec(0u32..10_000, cells..=cells)
            .prop_map(move |grid| (grid, width, height))
    })
}

#[test]
fn parse_hist_bin_accepts_valid_payload() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"IIH1");
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut bytes, 2.5);
    push_f32(&mut bytes, 100.0);
    push_f32(&mut bytes, 107.5);
    push_u32(&mut bytes, 3);
    push_counts(&mut bytes, &[3, 1, 1]);

    let hist = parse_hist_bin(&bytes).expect("valid payload should parse");
    assert_eq!(hist.base_bin, 2.5);
    assert_eq!(hist.min, 100.0);
    assert_eq!(hist.max, 107.5);
    assert_eq!(hist.counts, vec![3, 1, 1]);
    assert_eq!(hist.total, 5);
}

#[test]
fn parse_hist_bin_rejects_invalid_payload_len() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"IIH1");
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut bytes, 2.5);
    push_f32(&mut bytes, 100.0);
    push_f32(&mut bytes, 105.0);
    push_u32(&mut bytes, 2);
    push_counts(&mut bytes, &[1]); // header claims 2 bins, payload holds 1

    assert!(parse_hist_bin(&bytes).is_none());
}

#[test]
fn parse_heat_bin_accepts_valid_payload() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"IIM1");
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut bytes, 2.5);
    push_f32(&mut bytes, 1.0);
    push_f32(&mut bytes, 100.0);
    push_f32(&mut bytes, 107.5);
    push_f32(&mut bytes, 80.0);
    push_f32(&mut bytes, 82.0);
    push_u32(&mut bytes, 3);
    push_u32(&mut bytes, 2);
    push_counts(&mut bytes, &[3, 1, 0, 0, 0, 1]);

    let heat = parse_heat_bin(&bytes).expect("valid payload should parse");
    assert_eq!(heat.base_x, 2.5);
    assert_eq!(heat.base_y, 1.0);
    assert_eq!(heat.width, 3);
    assert_eq!(heat.height, 2);
    assert_eq!(heat.grid, vec![3, 1, 0, 0, 0, 1]);
}

fn make_hist_blob() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&HISTOGRAM_MAGIC);
    b.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut b, 2.5); // base
    push_f32(&mut b, 100.0); // min
    push_f32(&mut b, 105.0); // max
    push_u32(&mut b, 2); // bins count
    push_counts(&mut b, &[4, 1]);
    b
}

fn make_heat_blob() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&HEATMAP_MAGIC);
    b.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut b, 2.5); // base_x
    push_f32(&mut b, 1.0); // base_y
    push_f32(&mut b, 80.0);
    push_f32(&mut b, 82.5); // min_x, max_x
    push_f32(&mut b, 100.0);
    push_f32(&mut b, 101.0); // min_y, max_y
    push_u32(&mut b, 1); // width
    push_u32(&mut b, 1); // height
    push_counts(&mut b, &[5]);
    b
}

#[test]
fn parse_combined_bin_round_trips_hist_and_heat() {
    let hist_blob = make_hist_blob();
    let heat_blob = make_heat_blob();
    let hist_len = hist_blob.len() as u32;

    let mut combined = Vec::new();
    combined.extend_from_slice(&COMBINED_MAGIC);
    combined.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    combined.extend_from_slice(&hist_len.to_le_bytes());
    combined.extend_from_slice(&hist_blob);
    combined.extend_from_slice(&heat_blob);

    let (hist, heat) = parse_combined_bin(&combined).expect("combined should parse");
    assert_eq!(hist.counts, vec![4, 1]);
    assert_eq!(hist.total, 5);
    assert_eq!(heat.width, 1);
    assert_eq!(heat.height, 1);
    assert_eq!(heat.grid, vec![5]);
}

#[test]
fn parse_combined_bin_rejects_wrong_magic() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BAD!");
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 4]); // hist_len
    assert!(parse_combined_bin(&bytes).is_none());
}

#[test]
fn parse_combined_bin_rejects_truncated_payload() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&COMBINED_MAGIC);
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    // hist_len claims 100 bytes but we provide none
    bytes.extend_from_slice(&100u32.to_le_bytes());
    assert!(parse_combined_bin(&bytes).is_none());
}

#[test]
fn parse_heat_bin_rejects_invalid_payload_len() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"IIM1");
    bytes.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut bytes, 2.5);
    push_f32(&mut bytes, 1.0);
    push_f32(&mut bytes, 100.0);
    push_f32(&mut bytes, 105.0);
    push_f32(&mut bytes, 80.0);
    push_f32(&mut bytes, 82.0);
    push_u32(&mut bytes, 2);
    push_u32(&mut bytes, 2);
    push_counts(&mut bytes, &[1, 2, 3]); // header claims 4 cells, payload holds 3

    assert!(parse_heat_bin(&bytes).is_none());
}

#[test]
fn parse_rejects_unsupported_version() {
    let bad_version = BINARY_FORMAT_VERSION + 1;

    let mut hist_bytes = Vec::new();
    hist_bytes.extend_from_slice(b"IIH1");
    hist_bytes.extend_from_slice(&bad_version.to_le_bytes());
    push_f32(&mut hist_bytes, 2.5);
    push_f32(&mut hist_bytes, 100.0);
    push_f32(&mut hist_bytes, 102.5);
    push_u32(&mut hist_bytes, 1);
    push_counts(&mut hist_bytes, &[1]);
    assert!(parse_hist_bin(&hist_bytes).is_none());

    let mut heat_bytes = Vec::new();
    heat_bytes.extend_from_slice(b"IIM1");
    heat_bytes.extend_from_slice(&bad_version.to_le_bytes());
    push_f32(&mut heat_bytes, 2.5);
    push_f32(&mut heat_bytes, 1.0);
    push_f32(&mut heat_bytes, 100.0);
    push_f32(&mut heat_bytes, 102.5);
    push_f32(&mut heat_bytes, 80.0);
    push_f32(&mut heat_bytes, 81.0);
    push_u32(&mut heat_bytes, 1);
    push_u32(&mut heat_bytes, 1);
    push_counts(&mut heat_bytes, &[1]);
    assert!(parse_heat_bin(&heat_bytes).is_none());
}

#[test]
fn encode_counts_collapses_zero_runs() {
    // 1-byte literal, then a run marker + length, then a literal.
    assert_eq!(encode_counts(&[7, 0, 0, 0, 9]), vec![7, 0, 3, 9]);
    assert_eq!(encode_counts(&[]), Vec::<u8>::new());
    // 300 needs two varint groups: 0xAC 0x02.
    assert_eq!(encode_counts(&[300]), vec![0xAC, 0x02]);
}

#[test]
fn decode_counts_rejects_malformed_payloads() {
    let valid = encode_counts(&[1, 0, 0, 4]);
    assert_eq!(decode_counts(&valid, 4), Some(vec![1, 0, 0, 4]));

    // Truncated mid-payload.
    assert!(decode_counts(&valid[..valid.len() - 1], 4).is_none());
    // Trailing bytes after the declared count.
    let mut trailing = valid.clone();
    trailing.push(1);
    assert!(decode_counts(&trailing, 4).is_none());
    // A zero run that overruns the declared length.
    assert!(decode_counts(&encode_counts(&[0, 0, 0, 0]), 2).is_none());
    // Over-long varint (six groups) is refused rather than wrapping.
    assert!(decode_counts(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01], 1).is_none());
    // A header claiming more cells than any real grid must not drive an alloc.
    assert!(decode_counts(&valid, usize::MAX).is_none());
}

#[test]
fn percentile_for_value_handles_boundaries() {
    let hist = HistogramBin::new(100.0, 110.0, 2.5, vec![10, 20, 30, 40]);

    let low = percentile_for_value(Some(&hist), 80.0).expect("should compute");
    let high = percentile_for_value(Some(&hist), 200.0).expect("should compute");

    assert!(low.0 < high.0);
    assert_eq!(low.2, 100);
    assert_eq!(high.2, 100);
}

#[test]
fn percentile_for_value_returns_none_for_empty_distribution() {
    let empty = HistogramBin::new(0.0, 0.0, 1.0, vec![]);
    assert!(percentile_for_value(Some(&empty), 0.0).is_none());

    let zeroed = HistogramBin::new(0.0, 3.0, 1.0, vec![0, 0, 0]);
    assert!(percentile_for_value(Some(&zeroed), 1.0).is_none());
    assert!(percentile_for_value(None, 1.0).is_none());
}

#[test]
fn percentile_for_value_mid_bin_interpolation_matches_formula() {
    let hist = HistogramBin::new(100.0, 107.5, 2.5, vec![2, 2, 6]);
    let (pct, rank, total) = percentile_for_value(Some(&hist), 104.0).expect("should compute");
    assert!((pct - 0.3).abs() < 1e-6);
    assert_eq!(total, 10);
    assert_eq!(rank, 7);
}

#[test]
fn value_for_percentile_returns_expected_bin_midpoint() {
    let hist = HistogramBin::new(100.0, 107.5, 2.5, vec![2, 2, 6]);
    let value = value_for_percentile(Some(&hist), 0.30).expect("should compute");
    assert!((value - 103.75).abs() < 1e-6);
}

#[test]
fn rebin_1d_preserves_total_with_partial_tail() {
    let counts = vec![1, 2, 3, 4, 5];
    let out = rebin_1d(counts.clone(), 2);
    assert_eq!(out, vec![3, 7, 5]);
    assert_eq!(out.iter().sum::<u32>(), counts.iter().sum::<u32>());
}

#[test]
fn rebin_2d_preserves_total_with_partial_edges() {
    let grid = vec![1, 2, 3, 4, 5, 6];
    let (out, w2, h2) = rebin_2d(grid.clone(), 3, 2, 2, 2);
    assert_eq!((w2, h2), (2, 1));
    assert_eq!(out, vec![12, 9]);
    assert_eq!(out.iter().sum::<u32>(), grid.iter().sum::<u32>());
}

proptest! {
    #[test]
    fn rebin_1d_preserves_total_for_generated_counts(
        counts in bounded_count_vec(),
        k in 0usize..16,
    ) {
        let expected = counts.iter().copied().sum::<u32>();
        let out = rebin_1d(counts, k);

        prop_assert_eq!(out.iter().copied().sum::<u32>(), expected);
    }

    #[test]
    fn rebin_2d_preserves_total_and_shape_for_generated_grids(
        (grid, width, height) in bounded_heat_grid(),
        kx in 0usize..8,
        ky in 0usize..8,
    ) {
        let expected = grid.iter().copied().sum::<u32>();
        let (out, w2, h2) = rebin_2d(grid, width, height, kx, ky);

        prop_assert_eq!(out.iter().copied().sum::<u32>(), expected);
        prop_assert_eq!(out.len(), w2 * h2);
    }

    #[test]
    fn encode_decode_counts_round_trips(counts in prop::collection::vec(
        prop_oneof![Just(0u32), 0u32..5, 0u32..u32::MAX],
        0..256,
    )) {
        let encoded = encode_counts(&counts);
        let decoded = decode_counts(&encoded, counts.len());
        prop_assert_eq!(decoded, Some(counts));
    }

    #[test]
    fn decode_counts_never_panics_on_arbitrary_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..128),
        len in 0usize..256,
    ) {
        // Garbage must come back as None, never a panic or a wrong-length vec.
        if let Some(decoded) = decode_counts(&bytes, len) {
            prop_assert_eq!(decoded.len(), len);
        }
    }

    #[test]
    fn kg_lbs_round_trip_for_generated_values(kg in 0.0f32..1000.0) {
        let round_tripped = lbs_to_kg(kg_to_lbs(kg));

        prop_assert!((round_tripped - kg).abs() <= 1e-4);
    }
}

#[test]
fn score_functions_are_monotonic_for_fixed_bodyweight() {
    let dots_low = dots_points("M", 90.0, 500.0);
    let dots_high = dots_points("M", 90.0, 600.0);
    assert!(dots_high > dots_low);

    let wilks_low = wilks_points("F", 63.0, 350.0);
    let wilks_high = wilks_points("F", 63.0, 420.0);
    assert!(wilks_high > wilks_low);

    let gl_raw = goodlift_points("M", "Raw", 90.0, 700.0);
    let gl_equipped = goodlift_points("M", "Single-ply", 90.0, 700.0);
    assert!(gl_raw.is_finite());
    assert!(gl_equipped.is_finite());
    assert_ne!(gl_raw, gl_equipped);
}

#[test]
fn histogram_diagnostics_reports_expected_ranges() {
    let hist = HistogramBin::new(100.0, 112.0, 2.0, vec![0, 4, 10, 6, 2, 0]);

    let diag = histogram_diagnostics(Some(&hist)).expect("diagnostics should compute");
    assert!(diag.p01 <= diag.p05 && diag.p05 <= diag.p10);
    assert!(diag.p25 <= diag.p50 && diag.p50 <= diag.p75);
    assert!(diag.p90 <= diag.p95 && diag.p95 <= diag.p99);
    assert!((diag.iqr - (diag.p75 - diag.p25)).abs() < 1e-6);
    assert_eq!(diag.mode_bin_count, 10);
    assert_eq!(diag.occupied_bins, 4);
    assert_eq!(diag.total_bins, 6);
    assert!((0.0..=1.0).contains(&diag.sparsity_score));
}

#[test]
fn histogram_diagnostics_flags_tiny_sample() {
    let hist = HistogramBin::new(0.0, 4.0, 1.0, vec![50, 40, 30, 20]);
    let diag = histogram_diagnostics(Some(&hist)).expect("diagnostics should compute");
    assert_eq!(diag.total_lifters, 140);
    assert_eq!(
        diag.tiny_sample_warning,
        140 < TINY_COHORT_WARNING_THRESHOLD
    );
}

#[test]
fn histogram_density_reports_neighbors_and_label() {
    let hist = HistogramBin::new(100.0, 112.0, 2.0, vec![1, 4, 10, 6, 2, 1]);

    let density = histogram_density_for_value(Some(&hist), 104.2).expect("density should compute");
    assert_eq!(density.bin_index, 2);
    assert_eq!(density.current_bin_count, 10);
    assert_eq!(density.left_bin_count, 4);
    assert_eq!(density.right_bin_count, 6);
    assert_eq!(density.neighborhood_count, 20);
    assert_eq!(density.label, "dense middle");
    assert!((0.0..=1.0).contains(&density.local_density_ratio));
    assert!((0.0..=1.0).contains(&density.neighborhood_share));
}

#[test]
fn bodyweight_conditioned_percentile_uses_nearby_rows() {
    let heat = HeatmapBin {
        min_x: 100.0,
        max_x: 115.0,
        min_y: 60.0,
        max_y: 66.0,
        base_x: 5.0,
        base_y: 2.0,
        width: 3,
        height: 3,
        grid: vec![
            1, 2, 1, // y=0
            2, 6, 2, // y=1
            1, 2, 1, // y=2
        ],
    };

    let stats =
        bodyweight_conditioned_percentile(Some(&heat), 106.0, 62.5).expect("should compute");
    assert!((stats.percentile - 0.5).abs() < 1e-6);
    assert_eq!(stats.rank, 9);
    assert_eq!(stats.total_nearby, 18);
    assert_eq!(stats.local_cell_count, 6);
    assert_eq!(stats.neighborhood_count, 18);
    assert!((stats.neighborhood_share - 1.0).abs() < 1e-6);
}

#[test]
fn bodyweight_conditioned_percentile_clamps_edges() {
    let heat = HeatmapBin {
        min_x: 100.0,
        max_x: 110.0,
        min_y: 50.0,
        max_y: 54.0,
        base_x: 5.0,
        base_y: 2.0,
        width: 2,
        height: 2,
        grid: vec![
            10, 0, // low bw
            0, 0, // high bw
        ],
    };
    let stats = bodyweight_conditioned_percentile(Some(&heat), 95.0, 40.0).expect("should compute");
    assert_eq!(stats.bw_bin_index, 0);
    assert_eq!(stats.lift_bin_index, 0);
    assert_eq!(stats.total_nearby, 10);
    assert_eq!(stats.rank, 5);
}

#[test]
fn equivalent_value_for_same_percentile_maps_across_histograms() {
    let source = HistogramBin::new(100.0, 130.0, 10.0, vec![10, 10, 10]);
    let target = HistogramBin::new(200.0, 230.0, 10.0, vec![10, 10, 10]);

    let (pct, equivalent) =
        equivalent_value_for_same_percentile(Some(&source), Some(&target), 115.0)
            .expect("should compute equivalent value");
    assert!((pct - 0.5).abs() < 1e-6);
    assert!((equivalent - 215.0).abs() < 1e-6);
}

#[test]
fn equivalent_value_for_same_percentile_returns_none_without_data() {
    let source = HistogramBin::new(100.0, 120.0, 10.0, vec![5, 5]);

    assert!(equivalent_value_for_same_percentile(None, Some(&source), 105.0).is_none());
    assert!(equivalent_value_for_same_percentile(Some(&source), None, 105.0).is_none());
}

// ===== UNIT CONVERSION =====

use super::{
    IPF_PLATES_KG, JacksonPollock7SiteSkinfolds, KG_PER_LB, bodyfat_category, calc_1rm,
    calc_bodyfat_female, calc_bodyfat_jp3, calc_bodyfat_jp7, calc_bodyfat_male, calc_bodyfat_ymca,
    ipf_weight_class, kg_to_lbs, lbs_to_kg, plates_per_side, tier_for_percentile,
};

#[test]
fn kg_lbs_round_trip() {
    let kg = 100.0f32;
    let converted = lbs_to_kg(kg_to_lbs(kg));
    assert!((converted - kg).abs() < 1e-4);
}

#[test]
fn kg_to_lbs_known_value() {
    let lbs = kg_to_lbs(100.0);
    assert!((lbs - 100.0 / KG_PER_LB).abs() < 1e-4);
}

#[test]
fn lbs_to_kg_known_value() {
    let kg = lbs_to_kg(220.0);
    assert!((kg - 220.0 * KG_PER_LB).abs() < 1e-4);
}

// ===== PERCENTILE TIER =====

#[test]
fn tier_boundaries_are_correct() {
    assert_eq!(tier_for_percentile(0.0), "Novice");
    assert_eq!(tier_for_percentile(0.599), "Novice");
    assert_eq!(tier_for_percentile(0.6), "Intermediate");
    assert_eq!(tier_for_percentile(0.799), "Intermediate");
    assert_eq!(tier_for_percentile(0.8), "Advanced");
    assert_eq!(tier_for_percentile(0.949), "Advanced");
    assert_eq!(tier_for_percentile(0.95), "Elite");
    assert_eq!(tier_for_percentile(0.989), "Elite");
    assert_eq!(tier_for_percentile(0.99), "Legend");
    assert_eq!(tier_for_percentile(1.0), "Legend");
}

// ===== BODYFAT =====

#[test]
fn calc_bodyfat_male_typical_case() {
    let result = calc_bodyfat_male(178.0, 80.0, 38.0, 90.0).expect("should compute");
    assert!(result.body_fat_pct > 0.0 && result.body_fat_pct < 60.0);
    assert!((result.lean_mass_kg + result.fat_mass_kg - 80.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_male_rejects_invalid_inputs() {
    assert!(
        calc_bodyfat_male(0.0, 80.0, 38.0, 90.0).is_none(),
        "zero height"
    );
    assert!(
        calc_bodyfat_male(-5.0, 80.0, 38.0, 90.0).is_none(),
        "negative height"
    );
    assert!(
        calc_bodyfat_male(178.0, 80.0, 0.0, 90.0).is_none(),
        "zero neck"
    );
    assert!(
        calc_bodyfat_male(178.0, 80.0, 95.0, 90.0).is_none(),
        "waist <= neck"
    );
}

#[test]
fn calc_bodyfat_male_mass_components_sum_to_weight() {
    let result = calc_bodyfat_male(175.0, 90.0, 36.0, 85.0).expect("should compute");
    assert!((result.lean_mass_kg + result.fat_mass_kg - 90.0).abs() < 1e-3);
    assert!(result.body_fat_pct >= 2.0 && result.body_fat_pct <= 60.0);
}

#[test]
fn calc_bodyfat_female_typical_case() {
    let result = calc_bodyfat_female(165.0, 65.0, 33.0, 75.0, 95.0).expect("should compute");
    assert!(result.body_fat_pct >= 8.0 && result.body_fat_pct <= 60.0);
    assert!((result.lean_mass_kg + result.fat_mass_kg - 65.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_female_rejects_invalid_inputs() {
    assert!(
        calc_bodyfat_female(0.0, 65.0, 33.0, 75.0, 95.0).is_none(),
        "zero height"
    );
    assert!(
        calc_bodyfat_female(165.0, 65.0, 0.0, 75.0, 95.0).is_none(),
        "zero neck"
    );
    assert!(
        calc_bodyfat_female(165.0, 65.0, 200.0, 75.0, 95.0).is_none(),
        "diff <= 0"
    );
}

#[test]
fn bodyfat_category_male_boundaries() {
    assert_eq!(bodyfat_category(5.9, true), "Essential");
    assert_eq!(bodyfat_category(6.0, true), "Elite Athlete");
    assert_eq!(bodyfat_category(10.9, true), "Elite Athlete");
    assert_eq!(bodyfat_category(11.0, true), "Athlete");
    assert_eq!(bodyfat_category(14.9, true), "Athlete");
    assert_eq!(bodyfat_category(15.0, true), "Fitness");
    assert_eq!(bodyfat_category(19.9, true), "Fitness");
    assert_eq!(bodyfat_category(20.0, true), "Average");
    assert_eq!(bodyfat_category(24.9, true), "Average");
    assert_eq!(bodyfat_category(25.0, true), "Obese");
}

#[test]
fn bodyfat_category_female_boundaries() {
    assert_eq!(bodyfat_category(13.9, false), "Essential");
    assert_eq!(bodyfat_category(14.0, false), "Elite Athlete");
    assert_eq!(bodyfat_category(17.9, false), "Elite Athlete");
    assert_eq!(bodyfat_category(18.0, false), "Athlete");
    assert_eq!(bodyfat_category(21.9, false), "Athlete");
    assert_eq!(bodyfat_category(22.0, false), "Fitness");
    assert_eq!(bodyfat_category(25.9, false), "Fitness");
    assert_eq!(bodyfat_category(26.0, false), "Average");
    assert_eq!(bodyfat_category(31.9, false), "Average");
    assert_eq!(bodyfat_category(32.0, false), "Obese");
}

// ===== BODYFAT: ADDITIONAL METHODS =====

#[test]
fn siri_bf_from_density_known_values() {
    // Density 1.05 g/cc → ~21.43% body fat.
    assert!((siri_bf_from_density(1.05) - 21.4286).abs() < 0.01);
    // Density 1.10 g/cc → ~0% body fat (essentially fat-free).
    assert!(siri_bf_from_density(1.10).abs() < 0.05);
}

#[test]
fn calc_bodyfat_ymca_typical_male() {
    let r = calc_bodyfat_ymca(85.0, 85.0, true).expect("should compute");
    // 85 kg ≈ 187.39 lb, 85 cm ≈ 33.46 in.
    // BF = (-98.42 + 4.15*33.46 - 0.082*187.39) / 187.39 * 100 ≈ 13.5%
    assert!(
        (r.body_fat_pct - 13.5).abs() < 1.0,
        "got {}",
        r.body_fat_pct
    );
    assert!((r.lean_mass_kg + r.fat_mass_kg - 85.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_ymca_typical_female() {
    let r = calc_bodyfat_ymca(65.0, 75.0, false).expect("should compute");
    // 65 kg ≈ 143.30 lb, 75 cm ≈ 29.527 in.
    // BF = (-76.76 + 4.15*29.527 - 0.082*143.30) / 143.30 * 100 ≈ 23.75%
    assert!(
        (r.body_fat_pct - 23.75).abs() < 0.5,
        "got {}",
        r.body_fat_pct
    );
    assert!((r.lean_mass_kg + r.fat_mass_kg - 65.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_ymca_rejects_invalid_inputs() {
    assert!(calc_bodyfat_ymca(0.0, 85.0, true).is_none());
    assert!(calc_bodyfat_ymca(-5.0, 85.0, true).is_none());
    assert!(calc_bodyfat_ymca(85.0, 0.0, true).is_none());
    assert!(calc_bodyfat_ymca(85.0, -1.0, false).is_none());
}

#[test]
fn calc_bodyfat_ymca_clamps_minimum_by_sex() {
    // Very low waist → formula returns negative; clamp to sex-specific floor.
    let lean_male = calc_bodyfat_ymca(100.0, 40.0, true).expect("should compute");
    assert!((lean_male.body_fat_pct - 2.0).abs() < 1e-3);
    let lean_female = calc_bodyfat_ymca(100.0, 40.0, false).expect("should compute");
    assert!((lean_female.body_fat_pct - 8.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_jp3_typical_male() {
    // chest 10, abdomen 20, thigh 15, age 30, weight 85 kg.
    let r = calc_bodyfat_jp3(30.0, 85.0, true, 10.0, 20.0, 15.0).expect("should compute");
    assert!(r.body_fat_pct >= 2.0 && r.body_fat_pct <= 60.0);
    // Hand-computed: sum=45, BD ≈ 1.0677; Siri → ~13.6%
    assert!(
        (r.body_fat_pct - 13.6).abs() < 0.5,
        "got {}",
        r.body_fat_pct
    );
    assert!((r.lean_mass_kg + r.fat_mass_kg - 85.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_jp3_typical_female() {
    // tricep 15, suprailiac 12, thigh 20, age 30, weight 65 kg.
    let r = calc_bodyfat_jp3(30.0, 65.0, false, 15.0, 12.0, 20.0).expect("should compute");
    assert!(r.body_fat_pct >= 8.0 && r.body_fat_pct <= 60.0);
    // Hand-computed: sum=47, BD≈1.056_39; Siri → ~20.7%
    assert!(
        (r.body_fat_pct - 20.7).abs() < 1.0,
        "got {}",
        r.body_fat_pct
    );
    assert!((r.lean_mass_kg + r.fat_mass_kg - 65.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_jp3_rejects_invalid_inputs() {
    assert!(
        calc_bodyfat_jp3(0.0, 85.0, true, 10.0, 20.0, 15.0).is_none(),
        "zero age"
    );
    assert!(
        calc_bodyfat_jp3(200.0, 85.0, true, 10.0, 20.0, 15.0).is_none(),
        "age too high"
    );
    assert!(
        calc_bodyfat_jp3(30.0, 0.0, true, 10.0, 20.0, 15.0).is_none(),
        "zero weight"
    );
    assert!(
        calc_bodyfat_jp3(30.0, 85.0, true, 0.0, 20.0, 15.0).is_none(),
        "zero site a"
    );
    assert!(
        calc_bodyfat_jp3(30.0, 85.0, true, 10.0, -1.0, 15.0).is_none(),
        "negative site b"
    );
    assert!(
        calc_bodyfat_jp3(30.0, 85.0, true, 10.0, 20.0, 0.0).is_none(),
        "zero site c"
    );
}

#[test]
fn calc_bodyfat_jp3_clamps_lean_floor() {
    // Tiny skinfolds → BF pinned to sex-specific floor.
    let lean_male = calc_bodyfat_jp3(25.0, 80.0, true, 2.0, 2.0, 2.0).expect("should compute");
    assert!((lean_male.body_fat_pct - 2.0).abs() < 1e-3);
    let lean_female = calc_bodyfat_jp3(25.0, 65.0, false, 2.0, 2.0, 2.0).expect("should compute");
    assert!((lean_female.body_fat_pct - 8.0).abs() < 1e-3);
}

fn jp7_sites(
    [
        chest_mm,
        midaxillary_mm,
        tricep_mm,
        subscapular_mm,
        abdomen_mm,
        suprailiac_mm,
        thigh_mm,
    ]: [f32; 7],
) -> JacksonPollock7SiteSkinfolds {
    JacksonPollock7SiteSkinfolds {
        chest_mm,
        midaxillary_mm,
        tricep_mm,
        subscapular_mm,
        abdomen_mm,
        suprailiac_mm,
        thigh_mm,
    }
}

#[test]
fn calc_bodyfat_jp7_typical_male() {
    // 7 sites at 10 mm each, age 30, weight 85 kg.
    let r = calc_bodyfat_jp7(30.0, 85.0, true, jp7_sites([10.0; 7])).expect("should compute");
    assert!(r.body_fat_pct >= 2.0 && r.body_fat_pct <= 60.0);
    // Hand-computed: sum=70, BD≈1.073_25; Siri → ~11.2%
    assert!(
        (r.body_fat_pct - 11.2).abs() < 1.0,
        "got {}",
        r.body_fat_pct
    );
    assert!((r.lean_mass_kg + r.fat_mass_kg - 85.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_jp7_typical_female() {
    let r = calc_bodyfat_jp7(
        30.0,
        65.0,
        false,
        jp7_sites([12.0, 12.0, 15.0, 12.0, 15.0, 12.0, 20.0]),
    )
    .expect("should compute");
    assert!(r.body_fat_pct >= 8.0 && r.body_fat_pct <= 60.0);
    assert!((r.lean_mass_kg + r.fat_mass_kg - 65.0).abs() < 1e-3);
}

#[test]
fn calc_bodyfat_jp7_rejects_invalid_inputs() {
    // age invalid
    assert!(calc_bodyfat_jp7(0.0, 85.0, true, jp7_sites([10.0; 7])).is_none());
    // weight invalid
    assert!(calc_bodyfat_jp7(30.0, 0.0, true, jp7_sites([10.0; 7])).is_none());
    // any site zero or negative
    assert!(
        calc_bodyfat_jp7(
            30.0,
            85.0,
            true,
            jp7_sites([0.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0])
        )
        .is_none()
    );
    assert!(
        calc_bodyfat_jp7(
            30.0,
            85.0,
            true,
            jp7_sites([10.0, 10.0, 10.0, 10.0, 10.0, 10.0, -1.0])
        )
        .is_none()
    );
}

// ===== 1RM FORMULAS =====

#[test]
fn calc_1rm_returns_weight_for_one_rep() {
    assert_eq!(calc_1rm(100.0, 1.0, "epley"), 100.0);
    assert_eq!(calc_1rm(100.0, 0.5, "brzycki"), 100.0);
    assert_eq!(calc_1rm(100.0, 1.0, "lander"), 100.0);
}

#[test]
fn calc_1rm_epley_formula() {
    // w * (1 + r/30)
    let expected = 100.0 * (1.0 + 5.0 / 30.0);
    assert!((calc_1rm(100.0, 5.0, "epley") - expected).abs() < 1e-4);
    assert!(
        (calc_1rm(100.0, 5.0, "unknown") - expected).abs() < 1e-4,
        "defaults to epley"
    );
}

#[test]
fn calc_1rm_brzycki_formula() {
    // w / (1.0278 - 0.0278 * r)
    let expected = 100.0 / (1.0278 - 0.0278 * 5.0);
    assert!((calc_1rm(100.0, 5.0, "brzycki") - expected).abs() < 1e-3);
}

#[test]
fn calc_1rm_lander_formula() {
    // (100 * w) / (101.3 - 2.67123 * r)
    let expected = (100.0 * 100.0) / (101.3 - 2.67123 * 5.0);
    assert!((calc_1rm(100.0, 5.0, "lander") - expected).abs() < 1e-3);
}

#[test]
fn calc_1rm_mayhew_formula() {
    // (100 * w) / (52.2 + 41.9 * e^(-0.055 * r))
    let expected = (100.0 * 100.0) / (52.2 + 41.9 * (-0.055_f32 * 5.0).exp());
    assert!((calc_1rm(100.0, 5.0, "mayhew") - expected).abs() < 1e-3);
}

#[test]
fn calc_1rm_lombardi_formula() {
    // w * r^0.1
    let expected = 100.0f32 * 5.0f32.powf(0.1);
    assert!((calc_1rm(100.0, 5.0, "lombardi") - expected).abs() < 1e-4);
}

#[test]
fn calc_1rm_oconner_formula() {
    // w * (1 + r/40)
    let expected = 100.0 * (1.0 + 5.0 / 40.0);
    assert!((calc_1rm(100.0, 5.0, "oconner") - expected).abs() < 1e-4);
}

#[test]
fn calc_1rm_is_monotonic_with_reps() {
    for formula in &[
        "epley", "brzycki", "mayhew", "lander", "lombardi", "oconner",
    ] {
        let low = calc_1rm(100.0, 3.0, formula);
        let high = calc_1rm(100.0, 10.0, formula);
        assert!(high > low, "formula {formula} should increase with reps");
    }
}

// ===== PLATE CALCULATOR =====

#[test]
fn plates_per_side_exact_fit() {
    // 80 kg per side = 3×25 + 1×5
    let (plates, remainder) = plates_per_side(80.0);
    assert!(remainder < 1e-3);
    let total: f32 = plates.iter().map(|(w, c)| w * *c as f32).sum();
    assert!((total - 80.0).abs() < 1e-3);
}

#[test]
fn plates_per_side_zero_returns_empty() {
    let (plates, remainder) = plates_per_side(0.0);
    assert!(plates.is_empty());
    assert!(remainder < 1e-4);
}

#[test]
fn plates_per_side_negative_treated_as_zero() {
    let (plates, remainder) = plates_per_side(-10.0);
    assert!(plates.is_empty());
    assert!(remainder < 1e-4);
}

#[test]
fn plates_per_side_greedy_prefers_large_plates() {
    let (plates, _) = plates_per_side(50.0);
    assert!(!plates.is_empty());
    assert_eq!(plates[0].0, 25.0, "largest plate should come first");
}

#[test]
fn plates_per_side_remainder_is_non_negative() {
    for per_side in [1.0f32, 7.5, 12.3, 47.5, 100.0] {
        let (_, remainder) = plates_per_side(per_side);
        assert!(
            remainder >= 0.0,
            "remainder must be non-negative for {per_side}"
        );
    }
}

#[test]
fn plates_per_side_180kg_total_standard_bar() {
    // 180 total, 20kg bar → 80kg per side → 3×25 + 1×5
    let per_side = (180.0 - 20.0) / 2.0;
    let (plates, remainder) = plates_per_side(per_side);
    assert!(remainder < 1e-3);
    let total: f32 = plates.iter().map(|(w, c)| w * *c as f32).sum();
    assert!((total - 80.0).abs() < 1e-3);
}

#[test]
fn ipf_plates_kg_is_sorted_descending() {
    for window in IPF_PLATES_KG.windows(2) {
        assert!(window[0] > window[1], "plates should be descending");
    }
}

// ===== IPF WEIGHT CLASS =====

#[test]
fn ipf_weight_class_male_boundaries() {
    assert_eq!(ipf_weight_class(53.0, "M"), Some("53"));
    assert_eq!(ipf_weight_class(53.1, "M"), Some("59"));
    assert_eq!(ipf_weight_class(120.0, "M"), Some("120"));
    assert_eq!(ipf_weight_class(120.1, "M"), Some("120+"));
    assert_eq!(ipf_weight_class(200.0, "M"), Some("120+"));
}

#[test]
fn ipf_weight_class_female_boundaries() {
    assert_eq!(ipf_weight_class(43.0, "F"), Some("43"));
    assert_eq!(ipf_weight_class(43.1, "F"), Some("47"));
    assert_eq!(ipf_weight_class(84.0, "F"), Some("84"));
    assert_eq!(ipf_weight_class(84.1, "F"), Some("84+"));
}

#[test]
fn ipf_weight_class_invalid_sex_returns_none() {
    assert!(ipf_weight_class(80.0, "X").is_none());
    assert!(ipf_weight_class(80.0, "").is_none());
}
