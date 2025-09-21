// src/percentiles.rs - Simple, efficient percentile calculation
use polars::prelude::*;

/// Calculate percentile rank for a value in a column
/// Returns None if column doesn't exist or has no valid values
pub fn percentile_rank(df: &DataFrame, col_name: &str, value: Option<f32>) -> Option<f32> {
    let v = value?;

    let column = df.column(col_name).ok()?;
    let f32_series = column.f32().ok()?;

    let mut n = 0usize;
    let mut below = 0usize;

    for x in f32_series.into_no_null_iter() {
        if x.is_finite() && x > 0.0 {
            n += 1;
            if x < v {
                below += 1;
            }
        }
    }

    if n == 0 {
        return None;
    }

    Some(((below as f32 / n as f32) * 100.0).round())
}

