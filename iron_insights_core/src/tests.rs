use super::{
    BINARY_FORMAT_VERSION, COMBINED_MAGIC, HEATMAP_MAGIC, HISTOGRAM_MAGIC, HistogramBin,
    decode_counts, dots_points, encode_counts, equivalent_value_for_same_percentile,
    goodlift_points, parse_combined_bin, percentile_for_value, rebin_1d, rebin_2d,
    value_for_percentile, wilks_points,
};
use crate::binary::{parse_heat_bin, parse_hist_bin};
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

/// A header with no count payload: the shortest input each parser must accept.
fn minimal_hist_header(bins: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&HISTOGRAM_MAGIC);
    b.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    push_f32(&mut b, 2.5);
    push_f32(&mut b, 100.0);
    push_f32(&mut b, 100.0);
    push_u32(&mut b, bins);
    b
}

fn minimal_heat_header(width: u32, height: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&HEATMAP_MAGIC);
    b.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    for value in [2.5f32, 1.0, 100.0, 100.0, 80.0, 80.0] {
        push_f32(&mut b, value);
    }
    push_u32(&mut b, width);
    push_u32(&mut b, height);
    b
}

#[test]
fn parsers_accept_a_header_of_exactly_the_minimum_length() {
    // The guards are `len < N`, so a payload of exactly N must parse. Off-by-one
    // there would reject the smallest real cohorts rather than fail loudly.
    let hist = minimal_hist_header(0);
    assert_eq!(hist.len(), 22);
    assert!(parse_hist_bin(&hist).is_some());

    let heat = minimal_heat_header(0, 0);
    assert_eq!(heat.len(), 38);
    assert!(parse_heat_bin(&heat).is_some());
}

#[test]
fn parsers_reject_truncated_headers_without_panicking() {
    // Shorter than the magic itself: the length guard is the only thing between
    // this and an out-of-bounds slice.
    for len in [0usize, 1, 3, 4, 10, 21] {
        assert!(parse_hist_bin(&vec![0u8; len]).is_none(), "hist len {len}");
    }
    for len in [0usize, 4, 20, 37] {
        assert!(parse_heat_bin(&vec![0u8; len]).is_none(), "heat len {len}");
    }
    for len in [0usize, 4, 9] {
        assert!(
            parse_combined_bin(&vec![0u8; len]).is_none(),
            "combined len {len}"
        );
    }
}

#[test]
fn parsers_reject_the_right_length_with_the_wrong_magic() {
    // Length and magic are checked in one `||`. If that became `&&`, a
    // long-enough buffer of anything at all would be parsed as a payload.
    let mut hist = minimal_hist_header(0);
    hist[0..4].copy_from_slice(b"XXXX");
    assert!(parse_hist_bin(&hist).is_none());

    let mut heat = minimal_heat_header(0, 0);
    heat[0..4].copy_from_slice(b"XXXX");
    assert!(parse_heat_bin(&heat).is_none());

    let mut combined = Vec::new();
    combined.extend_from_slice(b"XXXX");
    combined.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
    combined.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(combined.len(), 10);
    assert!(parse_combined_bin(&combined).is_none());
}

#[test]
fn decode_counts_rejects_a_run_that_overruns_the_declared_length() {
    // Two literals then a run of three, but only four counts declared. The
    // remaining space is two, so the run must be refused. Comparing against
    // `len + out.len()` instead of `len - out.len()` would accept it and return
    // more counts than the header promised.
    let payload = encode_counts(&[1, 1, 0, 0, 0]);
    assert_eq!(payload, vec![1, 1, 0, 3]);
    assert!(decode_counts(&payload, 4).is_none());
}

#[test]
fn decode_counts_accepts_exactly_the_maximum_cell_count() {
    // The cap is `len > MAX_CELLS`, so the maximum itself is still valid. A zero
    // run makes a grid this size only a few bytes on the wire, which is what
    // makes the boundary cheap to test at all.
    const MAX_CELLS: usize = 1 << 22;
    let payload = encode_counts(&vec![0u32; MAX_CELLS]);

    assert_eq!(
        decode_counts(&payload, MAX_CELLS).map(|counts| counts.len()),
        Some(MAX_CELLS)
    );
    assert!(decode_counts(&payload, MAX_CELLS + 1).is_none());
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

use super::{
    IPF_PLATES_KG, JacksonPollock7SiteSkinfolds, bodyfat_category, calc_1rm, calc_bodyfat_female,
    calc_bodyfat_jp3, calc_bodyfat_jp7, calc_bodyfat_male, calc_bodyfat_ymca, ipf_weight_class,
    plates_per_side, tier_for_percentile,
};

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
    assert_eq!(calc_1rm(100.0, 1.0, "lombardi"), 100.0);
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
fn calc_1rm_is_monotonic_with_reps() {
    for formula in &["epley", "brzycki", "mayhew", "lombardi"] {
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

// ===== SCORING COEFFICIENT PINS =====
//
// Characterisation tests: they lock in the values the transcribed polynomial
// coefficients currently produce, so an accidental edit to any one of them
// fails loudly. They are NOT independently verified against the federations'
// published tables. The point is to detect drift, not to certify correctness.
//
// Mutation testing found 74 surviving mutants in `scoring.rs`: the other tests
// assert only that scores rise with total and stay positive, which almost any
// corrupted coefficient still satisfies. If a formula is deliberately updated,
// regenerate these numbers in the same commit.

/// Tight enough to catch a single-coefficient change, loose enough to survive
/// f32 rounding differences between platforms.
fn assert_score(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 0.01,
        "{what}: expected {expected}, got {actual}"
    );
}

#[test]
fn dots_matches_pinned_coefficients() {
    assert_score(dots_points("M", 93.0, 700.0), 445.3757, "dots M");
    assert_score(dots_points("F", 63.0, 400.0), 430.206, "dots F");
}

#[test]
fn wilks_matches_pinned_coefficients() {
    assert_score(wilks_points("M", 93.0, 700.0), 528.0868, "wilks M");
    assert_score(wilks_points("F", 63.0, 400.0), 447.7654, "wilks F");
}

#[test]
fn goodlift_matches_pinned_coefficients_per_equipment() {
    // Equipment splits classic from equipped, so all four combinations differ.
    assert_score(goodlift_points("M", "Raw", 93.0, 700.0), 91.5748, "gl M raw");
    assert_score(
        goodlift_points("M", "Single-ply", 93.0, 700.0),
        75.9133,
        "gl M equipped",
    );
    assert_score(goodlift_points("F", "Raw", 63.0, 400.0), 87.5133, "gl F raw");
    assert_score(
        goodlift_points("F", "Single-ply", 63.0, 400.0),
        72.214,
        "gl F equipped",
    );

    // Wraps and Straps count as classic, so they must match Raw exactly.
    for classic in ["Wraps", "Straps"] {
        assert_eq!(
            goodlift_points("M", classic, 93.0, 700.0),
            goodlift_points("M", "Raw", 93.0, 700.0),
            "{classic} should score as classic"
        );
    }
}

#[test]
fn bodyweight_is_clamped_to_each_formulas_valid_range() {
    // Outside the range the score must saturate rather than extrapolate off the
    // end of a 4th or 5th order polynomial.
    assert_score(dots_points("M", 30.0, 500.0), 635.5549, "dots below range");
    assert_score(dots_points("M", 250.0, 500.0), 247.8104, "dots above range");
    assert_score(wilks_points("F", 20.0, 300.0), 918.7693, "wilks below range");

    // Clamping means anything past the bound scores identically.
    assert_eq!(dots_points("M", 10.0, 500.0), dots_points("M", 40.0, 500.0));
    assert_eq!(dots_points("M", 400.0, 500.0), dots_points("M", 210.0, 500.0));

    // The female curve clamps at 150 kg, not the default 210. A heavyweight
    // woman is the only input where the two ranges disagree, so without this
    // the sex-specific arm could be deleted entirely and every other assertion
    // would still pass.
    assert_eq!(dots_points("F", 200.0, 400.0), dots_points("F", 150.0, 400.0));
    assert!(
        dots_points("F", 200.0, 400.0) > dots_points("M", 200.0, 400.0),
        "the female clamp must keep her score above the male curve at 200 kg"
    );
}

// ===== JACKSON-POLLOCK 7-SITE PINS =====
//
// Same reasoning as the scoring pins above: the body-density polynomial has its
// own set of transcribed coefficients that the range assertions elsewhere cannot
// distinguish from a corrupted one.

fn jp7_reference_sites() -> JacksonPollock7SiteSkinfolds {
    JacksonPollock7SiteSkinfolds {
        chest_mm: 12.0,
        midaxillary_mm: 10.0,
        tricep_mm: 14.0,
        subscapular_mm: 16.0,
        abdomen_mm: 20.0,
        suprailiac_mm: 18.0,
        thigh_mm: 22.0,
    }
}

#[test]
fn navy_and_ymca_match_pinned_coefficients() {
    // The Navy formulas are log-based and the YMCA ones linear; each has its own
    // set of transcribed constants that the range assertions elsewhere cannot
    // tell apart from a corrupted set.
    let male = calc_bodyfat_male(178.0, 85.0, 38.0, 90.0).expect("navy male");
    assert_score(male.body_fat_pct, 20.1466, "navy male pct");
    assert_score(male.lean_mass_kg, 67.8754, "navy male lean");
    assert_score(male.fat_mass_kg, 17.1246, "navy male fat");

    // The female formula adds the hip measurement and uses different constants.
    let female = calc_bodyfat_female(165.0, 65.0, 33.0, 75.0, 95.0).expect("navy female");
    assert_score(female.body_fat_pct, 26.9171, "navy female pct");
    assert_score(female.lean_mass_kg, 47.5039, "navy female lean");
    assert_score(female.fat_mass_kg, 17.4961, "navy female fat");

    assert_score(
        calc_bodyfat_ymca(85.0, 90.0, true).expect("ymca male").body_fat_pct,
        17.7493,
        "ymca male",
    );
    assert_score(
        calc_bodyfat_ymca(65.0, 75.0, false).expect("ymca female").body_fat_pct,
        23.7464,
        "ymca female",
    );
}

#[test]
fn jp3_matches_pinned_coefficients_and_bounds() {
    assert_score(
        calc_bodyfat_jp3(30.0, 85.0, true, 12.0, 20.0, 22.0)
            .expect("jp3 male")
            .body_fat_pct,
        16.2414,
        "jp3 male",
    );
    assert_score(
        calc_bodyfat_jp3(30.0, 65.0, false, 14.0, 18.0, 22.0)
            .expect("jp3 female")
            .body_fat_pct,
        22.1452,
        "jp3 female",
    );

    // Inclusive upper age bound, as in the 7-site version.
    assert!(calc_bodyfat_jp3(120.0, 85.0, true, 12.0, 20.0, 22.0).is_some());
    assert!(calc_bodyfat_jp3(121.0, 85.0, true, 12.0, 20.0, 22.0).is_none());
}

#[test]
fn jp7_matches_pinned_coefficients() {
    let male = calc_bodyfat_jp7(30.0, 85.0, true, jp7_reference_sites()).expect("male result");
    assert_score(male.body_fat_pct, 16.3070, "jp7 male pct");
    assert_score(male.lean_mass_kg, 71.1391, "jp7 male lean");
    assert_score(male.fat_mass_kg, 13.8609, "jp7 male fat");

    // The female branch uses different coefficients entirely.
    let female = calc_bodyfat_jp7(30.0, 85.0, false, jp7_reference_sites()).expect("female result");
    assert_score(female.body_fat_pct, 22.5227, "jp7 female pct");
    assert_score(female.lean_mass_kg, 65.8557, "jp7 female lean");
    assert_score(female.fat_mass_kg, 19.1443, "jp7 female fat");
}

#[test]
fn jp7_accepts_the_oldest_valid_age_and_rejects_past_it() {
    // The bound is inclusive; only an exact-120 case separates `>` from `>=`.
    assert!(calc_bodyfat_jp7(120.0, 85.0, true, jp7_reference_sites()).is_some());
    assert!(calc_bodyfat_jp7(121.0, 85.0, true, jp7_reference_sites()).is_none());
    assert!(calc_bodyfat_jp7(0.0, 85.0, true, jp7_reference_sites()).is_none());
}

#[test]
fn jp7_rejects_a_zero_reading_at_any_single_site() {
    // A caliper reading of exactly zero is impossible and would skew the sum.
    // Each site needs its own case: with six other positive readings, a single
    // dropped bound is invisible unless that site is the one set to zero.
    let sites = jp7_reference_sites();
    let zeroed: [(&str, JacksonPollock7SiteSkinfolds); 7] = [
        ("chest", JacksonPollock7SiteSkinfolds { chest_mm: 0.0, ..sites }),
        ("midaxillary", JacksonPollock7SiteSkinfolds { midaxillary_mm: 0.0, ..sites }),
        ("tricep", JacksonPollock7SiteSkinfolds { tricep_mm: 0.0, ..sites }),
        ("subscapular", JacksonPollock7SiteSkinfolds { subscapular_mm: 0.0, ..sites }),
        ("abdomen", JacksonPollock7SiteSkinfolds { abdomen_mm: 0.0, ..sites }),
        ("suprailiac", JacksonPollock7SiteSkinfolds { suprailiac_mm: 0.0, ..sites }),
        ("thigh", JacksonPollock7SiteSkinfolds { thigh_mm: 0.0, ..sites }),
    ];

    for (site, skinfolds) in zeroed {
        assert!(
            calc_bodyfat_jp7(30.0, 85.0, true, skinfolds).is_none(),
            "a zero {site} reading should be rejected"
        );
    }
}


