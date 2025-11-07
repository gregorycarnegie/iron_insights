// src/viz.rs - Centralized visualization computation with SIMD optimizations
use crate::{
    config::AppConfig,
    filters::apply_filters_lazy,
    models::{FilterParams, HistogramData, LiftType, ScatterData},
    percentiles::percentile_rank,
};
use iron_scoring::calculate_dots_score;
use polars::prelude::*;
use std::time::Instant;
use wide::f32x4;

/// Unified data carrier for all visualization data
pub struct VizData {
    pub hist: HistogramData,
    pub scatter: ScatterData,
    pub dots_hist: HistogramData,
    pub dots_scatter: ScatterData,
    pub user_percentile: Option<f32>,
    pub user_dots_percentile: Option<f32>,
    pub total_records: usize,
    pub processing_time_ms: u64,
}

/// Optimized compute function with zero-copy batch processing and aggressive lazy evaluation
pub fn compute_viz(
    data: &DataFrame,
    params: &FilterParams,
    config: &AppConfig,
) -> PolarsResult<VizData> {
    let t0 = Instant::now();

    // Determine lift type early
    let lift_type = LiftType::parse(params.lift_type.as_deref().unwrap_or("squat"));

    // Keep lazy frame as long as possible for optimization
    let lazy_filtered = apply_filters_lazy(data, params)?;

    // Use scan aggregations for statistics before collecting full data
    // This is more efficient than collecting all data first
    let raw_col = lift_type.raw_column();
    let dots_col = lift_type.dots_column();

    // Parallel stats computation using lazy evaluation
    let stats_df = lazy_filtered
        .clone()
        .select([
            col(raw_col).min().alias("raw_min"),
            col(raw_col).max().alias("raw_max"),
            col(dots_col).min().alias("dots_min"),
            col(dots_col).max().alias("dots_max"),
            col("BodyweightKg").count().alias("total_count"),
        ])
        .collect()?;

    let total_records = stats_df.column("total_count")?.u32()?.get(0).unwrap_or(0) as usize;

    // Only collect full data once for visualization processing
    let filtered = lazy_filtered.collect()?;

    println!("🔍 Filtered to {} records", total_records);

    // Batch process all visualizations in single pass (zero-copy)
    let (hist, scatter, dots_hist, dots_scatter) =
        create_all_viz_data_batch(&filtered, &lift_type, config)?;

    // Calculate percentiles if user data provided
    let user_percentile = if let Some(lift_value) = get_user_lift_value(params, &lift_type) {
        percentile_rank(&filtered, lift_type.raw_column(), Some(lift_value))
    } else {
        None
    };

    let user_dots_percentile = if let (Some(bw), Some(lift)) =
        (params.bodyweight, get_user_lift_value(params, &lift_type))
    {
        let user_sex = params.sex.as_deref().unwrap_or("M"); // Default to male if not specified
        let user_dots = calculate_dots_score(lift, bw, user_sex);
        percentile_rank(&filtered, lift_type.dots_column(), Some(user_dots))
    } else {
        None
    };

    Ok(VizData {
        hist,
        scatter,
        dots_hist,
        dots_scatter,
        user_percentile,
        user_dots_percentile,
        total_records,
        processing_time_ms: t0.elapsed().as_millis() as u64,
    })
}

/// Helper to select the right column based on DOTS flag
pub fn y_col(lift: &LiftType, use_dots: bool) -> &str {
    if use_dots {
        lift.dots_column()
    } else {
        lift.raw_column()
    }
}

/// Zero-copy batch processing for all visualizations
fn create_all_viz_data_batch(
    df: &DataFrame,
    lift_type: &LiftType,
    config: &AppConfig,
) -> PolarsResult<(HistogramData, ScatterData, HistogramData, ScatterData)> {
    // Extract all required columns in single pass
    let raw_col = y_col(lift_type, false);
    let dots_col = y_col(lift_type, true);

    let bodyweight = df.column("BodyweightKg")?.f32()?;
    let raw_values = df.column(raw_col)?.f32()?;
    let dots_values = df.column(dots_col)?.f32()?;
    let sex_col = df.column("Sex")?;
    let sex_series = match sex_col.dtype() {
        DataType::String => sex_col.clone(),
        DataType::Categorical(..) => sex_col.cast(&DataType::String)?,
        other => {
            return Err(PolarsError::ComputeError(
                format!("unexpected dtype for Sex column: {:?}", other).into(),
            ));
        }
    };
    let sex = sex_series.str()?;

    // Collect valid data in single pass with safe access
    let mut valid_bw = Vec::new();
    let mut valid_raw = Vec::new();
    let mut valid_dots = Vec::new();
    let mut valid_sex = Vec::new();

    for i in 0..df.height() {
        if let (Some(bw), Some(raw), Some(dots), Some(s)) = (
            bodyweight.get(i),
            raw_values.get(i),
            dots_values.get(i),
            sex.get(i),
        ) && bw.is_finite()
            && raw.is_finite()
            && dots.is_finite()
            && bw > 0.0
            && raw > 0.0
            && dots > 0.0
        {
            valid_bw.push(bw);
            valid_raw.push(raw);
            valid_dots.push(dots);
            valid_sex.push(s.to_string());
        }
    }

    // Create histograms with SIMD
    let hist = create_histogram_simd(&valid_raw, config.histogram_bins, "raw");
    let dots_hist = create_histogram_simd(&valid_dots, config.histogram_bins, "dots");

    // Create scatter data (already filtered)
    let scatter = ScatterData {
        x: valid_bw.clone(),
        y: valid_raw,
        sex: valid_sex.clone(),
    };

    let dots_scatter = ScatterData {
        x: valid_bw,
        y: valid_dots,
        sex: valid_sex,
    };

    Ok((hist, scatter, dots_hist, dots_scatter))
}

/// SIMD-accelerated histogram binning
fn create_histogram_simd(values: &[f32], bin_count: usize, _label: &str) -> HistogramData {
    if values.is_empty() {
        return HistogramData {
            values: vec![],
            counts: vec![],
            bins: vec![],
            min_val: 0.0,
            max_val: 0.0,
        };
    }

    // Calculate range using SIMD
    let (min_val, max_val) = find_min_max_simd(values);

    let bin_width = (max_val - min_val) / bin_count as f32;
    let inv_bin_width = if bin_width > 0.0 {
        1.0 / bin_width
    } else {
        0.0
    };

    let mut bins = vec![0u32; bin_count];

    // SIMD processing
    let chunks = values.chunks_exact(4);
    let remainder = chunks.remainder();

    // Process 4 values at once with SIMD
    for chunk in chunks {
        let vals = f32x4::from([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let normalized = (vals - f32x4::splat(min_val)) * f32x4::splat(inv_bin_width);
        let indices = normalized.floor();

        let indices_array = indices.to_array();
        for &idx in &indices_array {
            let bin_idx = (idx as usize).min(bin_count - 1);
            bins[bin_idx] += 1;
        }
    }

    // Handle remainder
    for &val in remainder {
        let bin_idx = ((val - min_val) * inv_bin_width).floor() as usize;
        bins[bin_idx.min(bin_count - 1)] += 1;
    }

    let bin_edges: Vec<f32> = (0..=bin_count)
        .map(|i| min_val + i as f32 * bin_width)
        .collect();

    HistogramData {
        values: values.to_vec(),
        counts: bins,
        bins: bin_edges,
        min_val,
        max_val,
    }
}

/// SIMD min/max finding
fn find_min_max_simd(values: &[f32]) -> (f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0);
    }

    let chunks = values.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut min_vec = f32x4::splat(f32::INFINITY);
    let mut max_vec = f32x4::splat(f32::NEG_INFINITY);

    // SIMD processing
    for chunk in chunks {
        let vals = f32x4::from([chunk[0], chunk[1], chunk[2], chunk[3]]);
        min_vec = min_vec.min(vals);
        max_vec = max_vec.max(vals);
    }

    // Extract SIMD results
    let min_array = min_vec.to_array();
    let max_array = max_vec.to_array();

    let mut min_val = min_array.iter().cloned().fold(f32::INFINITY, f32::min);
    let mut max_val = max_array.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    // Handle remainder
    for &val in remainder {
        min_val = min_val.min(val);
        max_val = max_val.max(val);
    }

    (min_val, max_val)
}

/// Get user's lift value based on lift type
fn get_user_lift_value(params: &FilterParams, lift_type: &LiftType) -> Option<f32> {
    match lift_type {
        LiftType::Squat => params.squat,
        LiftType::Bench => params.bench,
        LiftType::Deadlift => params.deadlift,
        LiftType::Total => {
            let s = params.squat.unwrap_or(0.0);
            let b = params.bench.unwrap_or(0.0);
            let d = params.deadlift.unwrap_or(0.0);
            if s > 0.0 || b > 0.0 || d > 0.0 {
                Some(s + b + d)
            } else {
                None
            }
        }
    }
}
