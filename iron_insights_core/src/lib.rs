mod published_contract;

pub use published_contract::{
    SliceEntryPaths, SliceKey, entry_paths_from_slice_key, parse_shard_key, parse_slice_key,
};

pub const BINARY_FORMAT_VERSION: u16 = 1;
pub const HISTOGRAM_MAGIC: [u8; 4] = *b"IIH1";
pub const HEATMAP_MAGIC: [u8; 4] = *b"IIM1";
/// Magic for the combined histogram+heatmap binary (IIC1 = Iron Insights Combined v1).
/// Layout: `[IIC1][version u16 LE][hist_len u32 LE][IIH1 blob][IIM1 blob]`
pub const COMBINED_MAGIC: [u8; 4] = *b"IIC1";

/// A parsed histogram binary payload for a single lifter cohort slice.
///
/// `counts[i]` is the number of lifters whose best lift falls in the bin
/// `[min + i * base_bin, min + (i + 1) * base_bin)`.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    /// Lower bound of the first bin (kg).
    pub min: f32,
    /// Upper bound of the last bin (kg).
    pub max: f32,
    /// Width of each bin (kg).
    pub base_bin: f32,
    /// Per-bin lifter counts.
    pub counts: Vec<u32>,
    total: u32,
}

/// A parsed heatmap binary payload mapping bodyweight (x) vs lift (y) cell counts.
///
/// `grid[y * width + x]` is the number of lifters in that (bodyweight, lift) cell.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapBin {
    /// Lower bodyweight bound (kg).
    pub min_x: f32,
    /// Upper bodyweight bound (kg).
    pub max_x: f32,
    /// Lower lift bound (kg).
    pub min_y: f32,
    /// Upper lift bound (kg).
    pub max_y: f32,
    /// Bodyweight bin width (kg).
    pub base_x: f32,
    /// Lift bin width (kg).
    pub base_y: f32,
    /// Number of bodyweight columns.
    pub width: usize,
    /// Number of lift rows.
    pub height: usize,
    /// Row-major grid of lifter counts (`grid[y * width + x]`).
    pub grid: Vec<u32>,
}

/// Descriptive statistics derived from a [`HistogramBin`] for display and QA.
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
    /// Interquartile range (p75 − p25).
    pub iqr: f32,
    /// p10 value (lower bound of the central 80%).
    pub central_80_low: f32,
    /// p90 value (upper bound of the central 80%).
    pub central_80_high: f32,
    pub mode_bin_start: f32,
    pub mode_bin_end: f32,
    pub mode_bin_center: f32,
    pub mode_bin_count: u32,
    /// Number of bins with at least one lifter.
    pub occupied_bins: usize,
    pub total_bins: usize,
    /// Fraction of bins that are empty (0.0 = dense, 1.0 = all empty).
    pub sparsity_score: f32,
    pub total_lifters: u32,
    /// True when the cohort is below [`TINY_COHORT_WARNING_THRESHOLD`].
    pub tiny_sample_warning: bool,
}

/// Local density context around a single histogram bin (e.g. the bin containing a user's lift).
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramDensity {
    /// Human-readable label for this density sample (e.g. `"your lift"`).
    pub label: &'static str,
    pub bin_index: usize,
    pub bin_start: f32,
    pub bin_end: f32,
    pub current_bin_count: u32,
    pub left_bin_count: u32,
    pub right_bin_count: u32,
    /// Sum of current, left, and right bin counts.
    pub neighborhood_count: u32,
    /// `current_bin_count / neighborhood_count`, or 0 if neighborhood is empty.
    pub local_density_ratio: f32,
    /// Fraction of total cohort that falls in the three-bin neighborhood.
    pub neighborhood_share: f32,
}

/// Percentile and rank statistics conditioned on a lifter's bodyweight band.
///
/// Derived from the heatmap by summing counts in the bodyweight column(s) that
/// bracket the lifter's actual bodyweight, then computing a rank within that
/// narrowed distribution.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyweightConditionedStats {
    /// Percentile within the bodyweight-conditioned cohort (0–100).
    pub percentile: f32,
    /// 1-based rank within the bodyweight-conditioned cohort.
    pub rank: usize,
    /// Total lifters in the bodyweight window used for conditioning.
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
const DIAGNOSTIC_PERCENTILES: [f32; 9] = [0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99];

/// IPF men's weight class boundaries and their canonical string labels.
///
/// Each entry is `(upper_bound_kg_exclusive, label)`. The final entry uses
/// `f32::INFINITY` to represent the open-ended top class (`"120+"`).
pub const IPF_MALE_WEIGHT_CLASSES: &[(f32, &str)] = &[
    (53.0, "53"),
    (59.0, "59"),
    (66.0, "66"),
    (74.0, "74"),
    (83.0, "83"),
    (93.0, "93"),
    (105.0, "105"),
    (120.0, "120"),
    (f32::INFINITY, "120+"),
];

/// IPF women's weight class boundaries and their canonical string labels.
///
/// Each entry is `(upper_bound_kg_exclusive, label)`. The final entry uses
/// `f32::INFINITY` to represent the open-ended top class (`"84+"`).
pub const IPF_FEMALE_WEIGHT_CLASSES: &[(f32, &str)] = &[
    (43.0, "43"),
    (47.0, "47"),
    (52.0, "52"),
    (57.0, "57"),
    (63.0, "63"),
    (69.0, "69"),
    (76.0, "76"),
    (84.0, "84"),
    (f32::INFINITY, "84+"),
];

/// Returns the IPF weight class label for a given bodyweight (kg) and sex (`"M"` or `"F"`).
///
/// Returns `None` if the sex string is not `"M"` or `"F"`.
pub fn ipf_weight_class(bodyweight_kg: f32, sex: &str) -> Option<&'static str> {
    let classes = match sex {
        "M" => IPF_MALE_WEIGHT_CLASSES,
        "F" => IPF_FEMALE_WEIGHT_CLASSES,
        _ => return None,
    };
    classes
        .iter()
        .find(|(upper, _)| bodyweight_kg <= *upper)
        .map(|(_, label)| *label)
}

impl HistogramBin {
    pub fn new(min: f32, max: f32, base_bin: f32, counts: Vec<u32>) -> Self {
        let total = counts.iter().copied().sum();
        Self {
            min,
            max,
            base_bin,
            counts,
            total,
        }
    }
}

pub fn parse_hist_bin(bytes: &[u8]) -> Option<HistogramBin> {
    if bytes.len() < 22 || bytes[0..4] != HISTOGRAM_MAGIC {
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

    Some(HistogramBin::new(min, max, base, counts))
}

pub fn parse_heat_bin(bytes: &[u8]) -> Option<HeatmapBin> {
    if bytes.len() < 38 || bytes[0..4] != HEATMAP_MAGIC {
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

/// Parses a combined IIC1 binary payload into a histogram and heatmap.
///
/// The IIC1 format stores both payloads in a single file:
/// `[IIC1][version u16 LE][hist_len u32 LE][IIH1 blob (hist_len bytes)][IIM1 blob (remainder)]`
pub fn parse_combined_bin(bytes: &[u8]) -> Option<(HistogramBin, HeatmapBin)> {
    if bytes.len() < 10 || bytes[0..4] != COMBINED_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    if version != BINARY_FORMAT_VERSION {
        return None;
    }
    let hist_len = u32::from_le_bytes(bytes[6..10].try_into().ok()?) as usize;
    let hist_bytes = bytes.get(10..10 + hist_len)?;
    let heat_bytes = bytes.get(10 + hist_len..)?;
    let hist = parse_hist_bin(hist_bytes)?;
    let heat = parse_heat_bin(heat_bytes)?;
    Some((hist, heat))
}

pub fn percentile_for_value(hist: Option<&HistogramBin>, value: f32) -> Option<(f32, usize, u32)> {
    let hist = hist?;
    if hist.counts.is_empty() {
        return None;
    }

    let total = hist.total;
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
    values_for_percentiles(hist, &[target_pct]).map(|[value]| value)
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

    let total = hist.total;
    if total == 0 {
        return None;
    }

    let [p01, p05, p10, p25, p50, p75, p90, p95, p99] =
        values_for_percentiles(hist, &DIAGNOSTIC_PERCENTILES)?;

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

#[cfg(target_arch = "wasm32")]
pub fn histogram_mean_stddev(hist: Option<&HistogramBin>) -> Option<(f32, f32)> {
    let hist = hist?;

    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let center =
        |idx: usize| -> f64 { hist.min as f64 + (idx as f64 + 0.5) * hist.base_bin as f64 };

    let (total, sum_x, sum_x2) = hist.counts.iter().copied().enumerate().fold(
        (0.0_f64, 0.0_f64, 0.0_f64),
        |(t, sx, sx2), (idx, count)| {
            let x = center(idx);
            let w = count as f64;
            (t + w, sx + w * x, sx2 + w * x * x)
        },
    );

    if total == 0.0 {
        return None;
    }

    let mean = sum_x / total;
    let variance = (sum_x2 / total - mean * mean).max(0.0);

    Some((mean as f32, variance.sqrt() as f32))
}

pub fn histogram_density_for_value(
    hist: Option<&HistogramBin>,
    value: f32,
) -> Option<HistogramDensity> {
    let hist = hist?;
    if hist.counts.is_empty() || hist.base_bin <= 0.0 {
        return None;
    }

    let total = hist.total;
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

fn values_for_percentiles<const N: usize>(
    hist: &HistogramBin,
    target_pcts: &[f32; N],
) -> Option<[f32; N]> {
    if hist.counts.is_empty() || hist.base_bin <= 0.0 || hist.total == 0 {
        return None;
    }

    let targets = target_pcts.map(|pct| pct.clamp(0.0, 1.0) * hist.total as f32);
    let mut values = [hist.max; N];
    let mut target_idx = 0usize;
    let mut below = 0.0f32;

    for (idx, count) in hist.counts.iter().copied().enumerate() {
        let count_f = count as f32;
        let cdf_mid = below + 0.5 * count_f;
        while target_idx < N && cdf_mid >= targets[target_idx] {
            values[target_idx] = hist.min + (idx as f32 + 0.5) * hist.base_bin;
            target_idx += 1;
        }
        if target_idx == N {
            break;
        }
        below += count_f;
    }

    Some(values)
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
        BINARY_FORMAT_VERSION, COMBINED_MAGIC, HEATMAP_MAGIC, HISTOGRAM_MAGIC, HeatmapBin,
        HistogramBin, TINY_COHORT_WARNING_THRESHOLD, bodyweight_conditioned_percentile,
        dots_points, equivalent_value_for_same_percentile, goodlift_points,
        histogram_density_for_value, histogram_diagnostics, parse_combined_bin, parse_heat_bin,
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

    fn make_hist_blob() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&HISTOGRAM_MAGIC);
        b.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
        push_f32(&mut b, 2.5); // base
        push_f32(&mut b, 100.0); // min
        push_f32(&mut b, 105.0); // max
        push_u32(&mut b, 2); // bins count
        push_u32(&mut b, 4); // bin[0]
        push_u32(&mut b, 1); // bin[1]
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
        push_u32(&mut b, 5); // grid[0]
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
}
