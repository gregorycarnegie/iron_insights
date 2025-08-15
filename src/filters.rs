// src/filters.rs - Centralized lazy filtering logic
use polars::prelude::*;
use crate::models::FilterParams;

/// Apply all filters lazily - single collect() point for performance
pub fn apply_filters_lazy(df: &DataFrame, params: &FilterParams) -> PolarsResult<LazyFrame> {
    let mut lf = df.clone().lazy();
    
    // Sex filter
    if let Some(sex) = &params.sex {
        if sex != "All" {
            lf = lf.filter(col("Sex").eq(lit(sex.as_str())));
        }
    }
    
    // Equipment filter using is_in() for efficiency
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() {
            // Build OR expression for multiple equipment types
            let eq_filter = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, expr| acc.or(expr))
                .unwrap_or(lit(true));
            lf = lf.filter(eq_filter);
        }
    }
    
    // Bodyweight range filter (if specified)
    if let Some(min_bw) = params.min_bodyweight {
        lf = lf.filter(col("BodyweightKg").gt_eq(lit(min_bw)));
    }
    if let Some(max_bw) = params.max_bodyweight {
        lf = lf.filter(col("BodyweightKg").lt_eq(lit(max_bw)));
    }
    
    // Weight class filter
    if let Some(weight_class) = &params.weight_class {
        if weight_class != "All" {
            lf = lf.filter(col("WeightClassKg").eq(lit(weight_class.as_str())));
        }
    }
    
    // Single validity gate - all records must have reasonable values
    lf = lf.filter(
        col("BodyweightKg").gt(lit(30.0))
            .and(col("BodyweightKg").lt(lit(300.0)))
            .and(col("Best3SquatKg").gt(lit(0.0)))
            .and(col("Best3BenchKg").gt(lit(0.0)))
            .and(col("Best3DeadliftKg").gt(lit(0.0)))
            .and(col("TotalKg").gt(lit(0.0)))
            // Ensure DOTS values are valid
            .and(col("SquatDOTS").is_finite())
            .and(col("BenchDOTS").is_finite())
            .and(col("DeadliftDOTS").is_finite())
            .and(col("TotalDOTS").is_finite())
            .and(col("SquatDOTS").gt(lit(0.0)))
            .and(col("BenchDOTS").gt(lit(0.0)))
            .and(col("DeadliftDOTS").gt(lit(0.0)))
            .and(col("TotalDOTS").gt(lit(0.0)))
    );
    
    Ok(lf)
}

/// Create a filter expression from parameters (for advanced use)
pub fn build_filter_expr(params: &FilterParams) -> Expr {
    let mut expr = lit(true);
    
    // Build composite filter expression
    if let Some(sex) = &params.sex {
        if sex != "All" {
            expr = expr.and(col("Sex").eq(lit(sex.as_str())));
        }
    }
    
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() {
            let eq_expr = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, e| acc.or(e))
                .unwrap_or(lit(false));
            expr = expr.and(eq_expr);
        }
    }
    
    expr
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_construction() {
        let params = FilterParams {
            sex: Some("M".to_string()),
            equipment: Some(vec!["Raw".to_string(), "Wraps".to_string()]),
            weight_class: None,
            squat: None,
            bench: None,
            deadlift: None,
            bodyweight: Some(75.0),
            units: None,
            lift_type: None,
            min_bodyweight: Some(60.0),
            max_bodyweight: Some(90.0),
        };
        
        // Create sample DataFrame
        let df = df! {
            "Sex" => ["M", "F", "M"],
            "Equipment" => ["Raw", "Single-ply", "Wraps"],
            "BodyweightKg" => [75.0f32, 65.0, 85.0],
            "Best3SquatKg" => [180.0f32, 120.0, 200.0],
            "Best3BenchKg" => [120.0f32, 70.0, 140.0],
            "Best3DeadliftKg" => [220.0f32, 140.0, 240.0],
            "TotalKg" => [520.0f32, 330.0, 580.0],
            "SquatDOTS" => [300.0f32, 280.0, 320.0],
            "BenchDOTS" => [200.0f32, 160.0, 220.0],
            "DeadliftDOTS" => [360.0f32, 320.0, 380.0],
            "TotalDOTS" => [860.0f32, 760.0, 920.0],
            "WeightClassKg" => ["74kg", "63kg", "83kg"],
        }.unwrap();
        
        let result = apply_filters_lazy(&df, &params);
        assert!(result.is_ok());
        
        let filtered = result.unwrap().collect();
        assert!(filtered.is_ok());
        
        let df_filtered = filtered.unwrap();
        // Should filter to only Male with Raw or Wraps equipment in 60-90kg range
        assert!(df_filtered.height() <= 2); // At most 2 records match
    }
}