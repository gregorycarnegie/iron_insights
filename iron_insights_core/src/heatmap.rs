use crate::binary::HeatmapBin;

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

/// Computes statistics for a lift within a narrowed bodyweight band using the 2D heatmap.
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
