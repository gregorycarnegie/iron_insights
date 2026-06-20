use crate::binary::HistogramBin;

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

/// Minimum number of lifters in a cohort before a tiny sample warning is triggered.
pub const TINY_COHORT_WARNING_THRESHOLD: u32 = 250;
const DIAGNOSTIC_PERCENTILES: [f32; 9] = [0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99];

/// Computes the CDF percentile, 1-based rank, and total count for a given value in a histogram.
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

/// Computes the expected value for a given percentile (0.0 to 1.0) using linear interpolation.
pub fn value_for_percentile(hist: Option<&HistogramBin>, target_pct: f32) -> Option<f32> {
    let hist = hist?;
    values_for_percentiles(hist, &[target_pct]).map(|[value]| value)
}

/// Maps a value from a source distribution to the equivalent value at the same percentile in a target distribution.
pub fn equivalent_value_for_same_percentile(
    source_hist: Option<&HistogramBin>,
    target_hist: Option<&HistogramBin>,
    source_value: f32,
) -> Option<(f32, f32)> {
    let source_percentile = percentile_for_value(source_hist, source_value)?.0;
    let target_value = value_for_percentile(target_hist, source_percentile)?;
    Some((source_percentile, target_value))
}

/// Calculates descriptive statistics (percentiles, IQR, mode, sparsity) for a histogram.
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
/// Calculates the estimated mean and standard deviation of the histogram population.
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

/// Evaluates the local density around a specific value to determine how common it is.
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
