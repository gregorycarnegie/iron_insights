use crate::binary::HistogramBin;

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
