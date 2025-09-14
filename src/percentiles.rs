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
    
}