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

/// Calculate percentile rank with custom filter
pub fn percentile_rank_filtered<F>(
    df: &DataFrame,
    col_name: &str,
    value: Option<f32>,
    filter: F,
) -> Option<f32>
where
    F: Fn(f32) -> bool,
{
    let v = value?;
    
    let column = df.column(col_name).ok()?;
    let f32_series = column.f32().ok()?;
    
    let mut n = 0usize;
    let mut below = 0usize;
    
    for x in f32_series.into_no_null_iter() {
        if filter(x) {
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

/// Get quantiles for a column (useful for box plots or summaries)
pub fn get_quantiles(df: &DataFrame, col_name: &str, quantiles: &[f64]) -> Vec<Option<f32>> {
    df.column(col_name)
        .ok()
        .and_then(|col| col.f32().ok())
        .map(|series| {
            quantiles.iter()
                .map(|&q| {
                    series.quantile(q, QuantileMethod::Linear)
                        .unwrap_or(None)
                        .map(|v| v as f32)
                })
                .collect()
        })
        .unwrap_or_else(|| vec![None; quantiles.len()])
}

/// Calculate multiple percentile ranks at once (batch operation)
pub fn percentile_ranks_batch(
    df: &DataFrame,
    col_name: &str,
    values: &[f32],
) -> Vec<Option<f32>> {
    let column = match df.column(col_name) {
        Ok(col) => col,
        Err(_) => return vec![None; values.len()],
    };
    
    let f32_series = match column.f32() {
        Ok(series) => series,
        Err(_) => return vec![None; values.len()],
    };
    
    // Collect valid values once
    let valid_values: Vec<f32> = f32_series
        .into_no_null_iter()
        .filter(|&x| x.is_finite() && x > 0.0)
        .collect();
    
    if valid_values.is_empty() {
        return vec![None; values.len()];
    }
    
    let n = valid_values.len() as f32;
    
    // Calculate percentile for each input value
    values.iter()
        .map(|&v| {
            let below = valid_values.iter().filter(|&&x| x < v).count() as f32;
            Some((below / n * 100.0).round())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentile_rank() {
        let df = df! {
            "values" => [100.0f32, 150.0, 200.0, 250.0, 300.0],
        }.unwrap();
        
        // Test middle value
        let p = percentile_rank(&df, "values", Some(200.0));
        assert_eq!(p, Some(40.0)); // 2 out of 5 values are below 200
        
        // Test lowest value
        let p = percentile_rank(&df, "values", Some(50.0));
        assert_eq!(p, Some(0.0)); // No values below 50
        
        // Test highest value
        let p = percentile_rank(&df, "values", Some(350.0));
        assert_eq!(p, Some(100.0)); // All values below 350
    }
    
    #[test]
    fn test_percentile_with_invalid_values() {
        let df = df! {
            "values" => [100.0f32, f32::NAN, 200.0, f32::INFINITY, 300.0, -50.0, 0.0],
        }.unwrap();
        
        // Should only count valid positive finite values (100, 200, 300)
        let p = percentile_rank(&df, "values", Some(250.0));
        assert_eq!(p, Some(67.0)); // 2 out of 3 valid values are below 250
    }
    
    #[test]
    fn test_batch_percentiles() {
        let df = df! {
            "values" => [100.0f32, 200.0, 300.0, 400.0, 500.0],
        }.unwrap();
        
        let test_values = vec![150.0, 350.0, 600.0];
        let results = percentile_ranks_batch(&df, "values", &test_values);
        
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Some(20.0)); // 1/5 below 150
        assert_eq!(results[1], Some(60.0)); // 3/5 below 350
        assert_eq!(results[2], Some(100.0)); // 5/5 below 600
    }
    
    #[test]
    fn test_quantiles() {
        let df = df! {
            "values" => [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        }.unwrap();
        
        let quantiles = get_quantiles(&df, "values", &[0.25, 0.5, 0.75]);
        assert_eq!(quantiles.len(), 3);
        
        // Approximate values for quartiles
        assert!(quantiles[0].unwrap() > 2.0 && quantiles[0].unwrap() < 4.0); // Q1
        assert!(quantiles[1].unwrap() > 4.0 && quantiles[1].unwrap() < 6.0); // Median
        assert!(quantiles[2].unwrap() > 6.0 && quantiles[2].unwrap() < 9.0); // Q3
    }
}