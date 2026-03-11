pub const BINARY_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    pub min: f32,
    pub max: f32,
    pub base_bin: f32,
    pub counts: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapBin {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub width: usize,
    pub height: usize,
    pub grid: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramDiagnostics {
    pub p01: f32,
    pub p05: f32,
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    pub p95: f32,
    pub p99: f32,
    pub iqr: f32,
    pub central_80_low: f32,
    pub central_80_high: f32,
    pub mode_bin_start: f32,
    pub mode_bin_end: f32,
    pub mode_bin_center: f32,
    pub mode_bin_count: u32,
    pub occupied_bins: usize,
    pub total_bins: usize,
    pub sparsity_score: f32,
    pub total_lifters: u32,
    pub tiny_sample_warning: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramDensity {
    pub label: &'static str,
    pub bin_index: usize,
    pub bin_start: f32,
    pub bin_end: f32,
    pub current_bin_count: u32,
    pub left_bin_count: u32,
    pub right_bin_count: u32,
    pub neighborhood_count: u32,
    pub local_density_ratio: f32,
    pub neighborhood_share: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyweightConditionedStats {
    pub percentile: f32,
    pub rank: usize,
    pub total_nearby: u32,
    pub bw_bin_index: usize,
    pub bw_bin_low: f32,
    pub bw_bin_high: f32,
    pub bw_window_low: f32,
    pub bw_window_high: f32,
    pub lift_bin_index: usize,
    pub lift_bin_low: f32,
    pub lift_bin_high: f32,
    pub local_cell_count: u32,
    pub neighborhood_count: u32,
    pub neighborhood_share: f32,
}

pub const TINY_COHORT_WARNING_THRESHOLD: u32 = 250;

pub fn parse_hist_bin(bytes: &[u8]) -> Option<HistogramBin> {
    if bytes.len() < 22 || &bytes[0..4] != b"IIH1" {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }

    let base = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let min = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let max = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let bins = u32::from_le_bytes(bytes[18..22].try_into().ok()?) as usize;

    let payload = bytes.get(22..)?;
    if payload.len() != bins * 4 {
        return None;
    }

    let mut counts = Vec::with_capacity(bins);
    for chunk in payload.chunks_exact(4) {
        counts.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HistogramBin {
        min,
        max,
        base_bin: base,
        counts,
    })
}

pub fn parse_heat_bin(bytes: &[u8]) -> Option<HeatmapBin> {
    if bytes.len() < 38 || &bytes[0..4] != b"IIM1" {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }

    let base_x = f32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let base_y = f32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let min_x = f32::from_le_bytes(bytes[14..18].try_into().ok()?);
    let max_x = f32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let min_y = f32::from_le_bytes(bytes[22..26].try_into().ok()?);
    let max_y = f32::from_le_bytes(bytes[26..30].try_into().ok()?);
    let width = u32::from_le_bytes(bytes[30..34].try_into().ok()?) as usize;
    let height = u32::from_le_bytes(bytes[34..38].try_into().ok()?) as usize;

    let payload = bytes.get(38..)?;
    if payload.len() != width * height * 4 {
        return None;
    }

    let mut grid = Vec::with_capacity(width * height);
    for chunk in payload.chunks_exact(4) {
        grid.push(u32::from_le_bytes(chunk.try_into().ok()?));
    }

    Some(HeatmapBin {
        min_x,
        max_x,
        min_y,
        max_y,
        base_x,
        base_y,
        width,
        height,
        grid,
    })
}

pub fn percentile_for_value(hist: Option<&HistogramBin>, value: f32) -> Option<(f32, usize, u32)> {
    let hist = hist?;
    if hist.counts.is_empty() {
        return None;
    }

    let total: u32 = hist.counts.iter().copied().sum();
    if total == 0 {
        return None;
    }

    let bin_idx = ((value - hist.min) / hist.base_bin)
        .floor()
        .clamp(0.0, (hist.counts.len() - 1) as f32) as usize;

    let below: u32 = hist.counts.iter().take(bin_idx).copied().sum();
    let current = hist.counts[bin_idx] as f32;
    let cdf = below as f32 + 0.5 * current;
    let pct = cdf / total as f32;
    let rank = ((1.0 - pct) * total as f32).round().max(1.0) as usize;

    Some((pct, rank, total))
}

pub fn value_for_percentile(hist: Option<&HistogramBin>, target_pct: f32) -> Option<f32> {
    let hist = hist?;
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let total: u32 = hist.counts.iter().copied().sum();
    if total == 0 {
        return None;
    }

    let target_cdf = target_pct.clamp(0.0, 1.0) * total as f32;
    let mut below = 0.0f32;

    for (idx, count) in hist.counts.iter().copied().enumerate() {
        let count_f = count as f32;
        let cdf_mid = below + 0.5 * count_f;
        if cdf_mid >= target_cdf {
            return Some(hist.min + (idx as f32 + 0.5) * hist.base_bin);
        }
        below += count_f;
    }

    Some(hist.max)
}

pub fn equivalent_value_for_same_percentile(
    source_hist: Option<&HistogramBin>,
    target_hist: Option<&HistogramBin>,
    source_value: f32,
) -> Option<(f32, f32)> {
    let source_percentile = percentile_for_value(source_hist, source_value)?.0;
    let target_value = value_for_percentile(target_hist, source_percentile)?;
    Some((source_percentile, target_value))
}

pub fn histogram_diagnostics(hist: Option<&HistogramBin>) -> Option<HistogramDiagnostics> {
    let hist = hist?;
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let total: u32 = hist.counts.iter().copied().sum();
    if total == 0 {
        return None;
    }

    let q = |pct: f32| value_for_percentile(Some(hist), pct);

    let p01 = q(0.01)?;
    let p05 = q(0.05)?;
    let p10 = q(0.10)?;
    let p25 = q(0.25)?;
    let p50 = q(0.50)?;
    let p75 = q(0.75)?;
    let p90 = q(0.90)?;
    let p95 = q(0.95)?;
    let p99 = q(0.99)?;

    let (mode_idx, mode_count) = hist
        .counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(idx, count)| (idx, *count))?;
    let mode_bin_start = hist.min + mode_idx as f32 * hist.base_bin;
    let mode_bin_end = mode_bin_start + hist.base_bin;

    let occupied_bins = hist.counts.iter().filter(|&&count| count > 0).count();
    let total_bins = hist.counts.len();
    let sparsity_score = 1.0 - (occupied_bins as f32 / total_bins as f32);

    Some(HistogramDiagnostics {
        p01,
        p05,
        p10,
        p25,
        p50,
        p75,
        p90,
        p95,
        p99,
        iqr: p75 - p25,
        central_80_low: p10,
        central_80_high: p90,
        mode_bin_start,
        mode_bin_end,
        mode_bin_center: mode_bin_start + 0.5 * hist.base_bin,
        mode_bin_count: mode_count,
        occupied_bins,
        total_bins,
        sparsity_score,
        total_lifters: total,
        tiny_sample_warning: total < TINY_COHORT_WARNING_THRESHOLD,
    })
}

pub fn histogram_density_for_value(
    hist: Option<&HistogramBin>,
    value: f32,
) -> Option<HistogramDensity> {
    let hist = hist?;
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let total: u32 = hist.counts.iter().copied().sum();
    if total == 0 {
        return None;
    }

    let bin_index = ((value - hist.min) / hist.base_bin)
        .floor()
        .clamp(0.0, (hist.counts.len() - 1) as f32) as usize;
    let current_bin_count = hist.counts[bin_index];
    let left_bin_count = if bin_index > 0 {
        hist.counts[bin_index - 1]
    } else {
        0
    };
    let right_bin_count = if bin_index + 1 < hist.counts.len() {
        hist.counts[bin_index + 1]
    } else {
        0
    };
    let neighborhood_count = left_bin_count + current_bin_count + right_bin_count;
    let mode_count = hist.counts.iter().copied().max().unwrap_or(0).max(1);
    let local_density_ratio = current_bin_count as f32 / mode_count as f32;
    let neighborhood_share = neighborhood_count as f32 / total as f32;

    let label = if local_density_ratio >= 0.65 {
        "dense middle"
    } else if local_density_ratio >= 0.30 {
        "moderately common"
    } else if local_density_ratio >= 0.10 {
        "rare air"
    } else {
        "extreme tail"
    };

    let bin_start = hist.min + bin_index as f32 * hist.base_bin;
    let bin_end = bin_start + hist.base_bin;

    Some(HistogramDensity {
        label,
        bin_index,
        bin_start,
        bin_end,
        current_bin_count,
        left_bin_count,
        right_bin_count,
        neighborhood_count,
        local_density_ratio,
        neighborhood_share,
    })
}

pub fn bodyweight_conditioned_percentile(
    heat: Option<&HeatmapBin>,
    user_lift: f32,
    user_bw: f32,
) -> Option<BodyweightConditionedStats> {
    let heat = heat?;
    if heat.width == 0 || heat.height == 0 || heat.grid.len() != heat.width * heat.height {
        return None;
    }
    if heat.base_x <= 0.0 || heat.base_y <= 0.0 {
        return None;
    }

    let total_heat: u32 = heat.grid.iter().copied().sum();
    if total_heat == 0 {
        return None;
    }

    let lift_bin_index = ((user_lift - heat.min_x) / heat.base_x)
        .floor()
        .clamp(0.0, (heat.width - 1) as f32) as usize;
    let bw_bin_index = ((user_bw - heat.min_y) / heat.base_y)
        .floor()
        .clamp(0.0, (heat.height - 1) as f32) as usize;

    let row_lo = bw_bin_index.saturating_sub(1);
    let row_hi = (bw_bin_index + 1).min(heat.height - 1);

    let mut nearby_counts = vec![0u32; heat.width];
    for y in row_lo..=row_hi {
        for (x, sum) in nearby_counts.iter_mut().enumerate() {
            let idx = y * heat.width + x;
            *sum = sum.saturating_add(heat.grid[idx]);
        }
    }

    let total_nearby: u32 = nearby_counts.iter().copied().sum();
    if total_nearby == 0 {
        return None;
    }

    let below: u32 = nearby_counts.iter().take(lift_bin_index).copied().sum();
    let current = nearby_counts[lift_bin_index] as f32;
    let cdf = below as f32 + 0.5 * current;
    let percentile = cdf / total_nearby as f32;
    let rank = ((1.0 - percentile) * total_nearby as f32).round().max(1.0) as usize;

    let mut neighborhood_count = 0u32;
    let x_lo = lift_bin_index.saturating_sub(1);
    let x_hi = (lift_bin_index + 1).min(heat.width - 1);
    for y in row_lo..=row_hi {
        for x in x_lo..=x_hi {
            neighborhood_count = neighborhood_count.saturating_add(heat.grid[y * heat.width + x]);
        }
    }
    let neighborhood_share = neighborhood_count as f32 / total_heat as f32;

    Some(BodyweightConditionedStats {
        percentile,
        rank,
        total_nearby,
        bw_bin_index,
        bw_bin_low: heat.min_y + bw_bin_index as f32 * heat.base_y,
        bw_bin_high: heat.min_y + (bw_bin_index as f32 + 1.0) * heat.base_y,
        bw_window_low: heat.min_y + row_lo as f32 * heat.base_y,
        bw_window_high: heat.min_y + (row_hi as f32 + 1.0) * heat.base_y,
        lift_bin_index,
        lift_bin_low: heat.min_x + lift_bin_index as f32 * heat.base_x,
        lift_bin_high: heat.min_x + (lift_bin_index as f32 + 1.0) * heat.base_x,
        local_cell_count: heat.grid[bw_bin_index * heat.width + lift_bin_index],
        neighborhood_count,
        neighborhood_share,
    })
}

#[allow(clippy::excessive_precision)]
pub fn dots_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(40.0, 150.0),
        _ => bodyweight_kg.clamp(40.0, 210.0),
    };
    let denom = if sex == "F" {
        -57.96288 + 13.6175032 * bw - 0.1126655495 * bw.powi(2) + 0.0005158568 * bw.powi(3)
            - 0.0000010706 * bw.powi(4)
    } else {
        -307.75076 + 24.0900756 * bw - 0.1918759221 * bw.powi(2) + 0.0007391293 * bw.powi(3)
            - 0.0000010930 * bw.powi(4)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

#[allow(clippy::excessive_precision)]
pub fn wilks_points(sex: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let bw = match sex {
        "F" => bodyweight_kg.clamp(26.51, 154.53),
        _ => bodyweight_kg.clamp(40.0, 201.9),
    };
    let denom = if sex == "F" {
        594.31747775582 - 27.23842536447 * bw + 0.82112226871 * bw.powi(2)
            - 0.00930733913 * bw.powi(3)
            + 0.00004731582 * bw.powi(4)
            - 0.00000009054 * bw.powi(5)
    } else {
        -216.0475144 + 16.2606339 * bw - 0.002388645 * bw.powi(2) - 0.00113732 * bw.powi(3)
            + 0.00000701863 * bw.powi(4)
            - 0.00000001291 * bw.powi(5)
    };
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 500.0 / denom
    }
}

#[allow(clippy::excessive_precision)]
pub fn goodlift_points(sex: &str, equipment: &str, bodyweight_kg: f32, total_kg: f32) -> f32 {
    let classic = matches!(equipment, "Raw" | "Wraps" | "Straps");
    let (a, b, c) = match (sex, classic) {
        ("F", true) => (610.32796, 1045.59282, 0.03048),
        ("F", false) => (758.63878, 949.31382, 0.02435),
        ("M", true) => (1199.72839, 1025.18162, 0.00921),
        _ => (1236.25115, 1449.21864, 0.01644),
    };
    let denom = a - (b * (-c * bodyweight_kg).exp());
    if denom <= 0.0 {
        0.0
    } else {
        total_kg * 100.0 / denom
    }
}

pub fn rebin_1d(counts: Vec<u32>, k: usize) -> Vec<u32> {
    if k <= 1 {
        return counts;
    }
    counts
        .chunks(k)
        .map(|chunk| chunk.iter().copied().sum())
        .collect()
}

pub fn rebin_2d(
    grid: Vec<u32>,
    width: usize,
    height: usize,
    kx: usize,
    ky: usize,
) -> (Vec<u32>, usize, usize) {
    if kx <= 1 && ky <= 1 {
        return (grid, width, height);
    }

    let w2 = width.div_ceil(kx.max(1));
    let h2 = height.div_ceil(ky.max(1));
    let mut out = vec![0u32; w2 * h2];

    for y in 0..height {
        for x in 0..width {
            let src = y * width + x;
            let dst = (y / ky.max(1)) * w2 + (x / kx.max(1));
            out[dst] = out[dst].saturating_add(grid[src]);
        }
    }

    (out, w2, h2)
}

#[cfg(test)]
mod tests {
    use super::{
        BINARY_FORMAT_VERSION, HeatmapBin, HistogramBin, TINY_COHORT_WARNING_THRESHOLD,
        bodyweight_conditioned_percentile, dots_points, equivalent_value_for_same_percentile,
        goodlift_points, histogram_density_for_value, histogram_diagnostics, parse_heat_bin,
        parse_hist_bin, percentile_for_value, rebin_1d, rebin_2d, value_for_percentile,
        wilks_points,
    };

    fn push_f32(bytes: &mut Vec<u8>, v: f32) {
        bytes.extend_from_slice(&v.to_le_bytes());
    }

    fn push_u32(bytes: &mut Vec<u8>, v: u32) {
        bytes.extend_from_slice(&v.to_le_bytes());
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
        push_u32(&mut bytes, 3);
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);

        let hist = parse_hist_bin(&bytes).expect("valid payload should parse");
        assert_eq!(hist.base_bin, 2.5);
        assert_eq!(hist.min, 100.0);
        assert_eq!(hist.max, 107.5);
        assert_eq!(hist.counts, vec![3, 1, 1]);
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
        push_u32(&mut bytes, 1);

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
        for v in [3, 1, 0, 0, 0, 1] {
            push_u32(&mut bytes, v);
        }

        let heat = parse_heat_bin(&bytes).expect("valid payload should parse");
        assert_eq!(heat.base_x, 2.5);
        assert_eq!(heat.base_y, 1.0);
        assert_eq!(heat.width, 3);
        assert_eq!(heat.height, 2);
        assert_eq!(heat.grid.len(), 6);
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
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 2);
        push_u32(&mut bytes, 3);

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
        push_u32(&mut hist_bytes, 1);
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
        push_u32(&mut heat_bytes, 1);
        assert!(parse_heat_bin(&heat_bytes).is_none());
    }

    #[test]
    fn percentile_for_value_handles_boundaries() {
        let hist = HistogramBin {
            min: 100.0,
            max: 110.0,
            base_bin: 2.5,
            counts: vec![10, 20, 30, 40],
        };

        let low = percentile_for_value(Some(&hist), 80.0).expect("should compute");
        let high = percentile_for_value(Some(&hist), 200.0).expect("should compute");

        assert!(low.0 < high.0);
        assert_eq!(low.2, 100);
        assert_eq!(high.2, 100);
    }

    #[test]
    fn percentile_for_value_returns_none_for_empty_distribution() {
        let empty = HistogramBin {
            min: 0.0,
            max: 0.0,
            base_bin: 1.0,
            counts: vec![],
        };
        assert!(percentile_for_value(Some(&empty), 0.0).is_none());

        let zeroed = HistogramBin {
            min: 0.0,
            max: 3.0,
            base_bin: 1.0,
            counts: vec![0, 0, 0],
        };
        assert!(percentile_for_value(Some(&zeroed), 1.0).is_none());
        assert!(percentile_for_value(None, 1.0).is_none());
    }

    #[test]
    fn percentile_for_value_mid_bin_interpolation_matches_formula() {
        let hist = HistogramBin {
            min: 100.0,
            max: 107.5,
            base_bin: 2.5,
            counts: vec![2, 2, 6],
        };
        let (pct, rank, total) = percentile_for_value(Some(&hist), 104.0).expect("should compute");
        assert!((pct - 0.3).abs() < 1e-6);
        assert_eq!(total, 10);
        assert_eq!(rank, 7);
    }

    #[test]
    fn value_for_percentile_returns_expected_bin_midpoint() {
        let hist = HistogramBin {
            min: 100.0,
            max: 107.5,
            base_bin: 2.5,
            counts: vec![2, 2, 6],
        };
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
        let hist = HistogramBin {
            min: 100.0,
            max: 112.0,
            base_bin: 2.0,
            counts: vec![0, 4, 10, 6, 2, 0],
        };

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
        let hist = HistogramBin {
            min: 0.0,
            max: 4.0,
            base_bin: 1.0,
            counts: vec![50, 40, 30, 20],
        };
        let diag = histogram_diagnostics(Some(&hist)).expect("diagnostics should compute");
        assert_eq!(diag.total_lifters, 140);
        assert_eq!(
            diag.tiny_sample_warning,
            140 < TINY_COHORT_WARNING_THRESHOLD
        );
    }

    #[test]
    fn histogram_density_reports_neighbors_and_label() {
        let hist = HistogramBin {
            min: 100.0,
            max: 112.0,
            base_bin: 2.0,
            counts: vec![1, 4, 10, 6, 2, 1],
        };

        let density =
            histogram_density_for_value(Some(&hist), 104.2).expect("density should compute");
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
        let stats =
            bodyweight_conditioned_percentile(Some(&heat), 95.0, 40.0).expect("should compute");
        assert_eq!(stats.bw_bin_index, 0);
        assert_eq!(stats.lift_bin_index, 0);
        assert_eq!(stats.total_nearby, 10);
        assert_eq!(stats.rank, 5);
    }

    #[test]
    fn equivalent_value_for_same_percentile_maps_across_histograms() {
        let source = HistogramBin {
            min: 100.0,
            max: 130.0,
            base_bin: 10.0,
            counts: vec![10, 10, 10],
        };
        let target = HistogramBin {
            min: 200.0,
            max: 230.0,
            base_bin: 10.0,
            counts: vec![10, 10, 10],
        };

        let (pct, equivalent) =
            equivalent_value_for_same_percentile(Some(&source), Some(&target), 115.0)
                .expect("should compute equivalent value");
        assert!((pct - 0.5).abs() < 1e-6);
        assert!((equivalent - 215.0).abs() < 1e-6);
    }

    #[test]
    fn equivalent_value_for_same_percentile_returns_none_without_data() {
        let source = HistogramBin {
            min: 100.0,
            max: 120.0,
            base_bin: 10.0,
            counts: vec![5, 5],
        };

        assert!(equivalent_value_for_same_percentile(None, Some(&source), 105.0).is_none());
        assert!(equivalent_value_for_same_percentile(Some(&source), None, 105.0).is_none());
    }
}
