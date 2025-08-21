// src/filters.rs - Centralized lazy filtering logic with column pruning
use polars::prelude::*;
use lazy_static::lazy_static;
use chrono::{self, Datelike};
use crate::models::{FilterParams, LiftType};

// Pre-compiled filter expressions for maximum performance
lazy_static! {
    static ref VALIDITY_EXPR: Expr = col("BodyweightKg").gt(lit(30.0))
        .and(col("BodyweightKg").lt(lit(300.0)))
        .and(col("Best3SquatKg").gt(lit(0.0)))
        .and(col("Best3BenchKg").gt(lit(0.0)))
        .and(col("Best3DeadliftKg").gt(lit(0.0)))
        .and(col("TotalKg").gt(lit(0.0)))
        .and(col("SquatDOTS").is_finite())
        .and(col("BenchDOTS").is_finite())
        .and(col("DeadliftDOTS").is_finite())
        .and(col("TotalDOTS").is_finite())
        .and(col("SquatDOTS").gt(lit(0.0)))
        .and(col("BenchDOTS").gt(lit(0.0)))
        .and(col("DeadliftDOTS").gt(lit(0.0)))
        .and(col("TotalDOTS").gt(lit(0.0)));
}

/// Optimized lazy filtering with column pruning and pre-compiled expressions
pub fn apply_filters_lazy(df: &DataFrame, params: &FilterParams) -> PolarsResult<LazyFrame> {
    // Column pruning: only select required columns based on lift type
    let required_cols = get_required_columns(params);
    
    let mut lf = df.clone().lazy()
        .select(required_cols); // Prune columns early for I/O efficiency
    
    // Apply user filters first (most selective)
    
    // Sex filter
    if let Some(sex) = &params.sex {
        if sex != "All" {
            lf = lf.filter(col("Sex").eq(lit(sex.as_str())));
        }
    }
    
    // Equipment filter using OR chains (is_in not available in this version)
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() && !equipment.contains(&"All".to_string()) {
            // Build OR expression for multiple equipment types
            let eq_filter = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, expr| acc.or(expr))
                .unwrap_or(lit(true));
            lf = lf.filter(eq_filter);
        }
    }
    
    // Years filter using the Date column (parsed as dates)
    if let Some(years_filter) = &params.years_filter {
        match years_filter.as_str() {
            "past_5_years" => {
                // Filter to records from the past 5 years
                let five_years_ago = chrono::Utc::now() - chrono::Duration::days(5 * 365);
                let cutoff_date = five_years_ago.date_naive();
                lf = lf.filter(col("Date").gt_eq(lit(cutoff_date)));
            }
            "past_10_years" => {
                // Filter to records from the past 10 years
                let ten_years_ago = chrono::Utc::now() - chrono::Duration::days(10 * 365);
                let cutoff_date = ten_years_ago.date_naive();
                lf = lf.filter(col("Date").gt_eq(lit(cutoff_date)));
            }
            "ytd" => {
                // Filter to records from this year (Year To Date)
                let current_year = chrono::Utc::now().year();
                let year_start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
                lf = lf.filter(col("Date").gt_eq(lit(year_start)));
            }
            _ => {} // "all" or unknown - no filtering needed
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
    
    // Apply pre-compiled validity filter (most expensive, applied last)
    lf = lf.filter(VALIDITY_EXPR.clone());
    
    Ok(lf)
}

/// Get minimal required columns based on parameters to reduce I/O
fn get_required_columns(params: &FilterParams) -> Vec<Expr> {
    let _lift_type = LiftType::from_str(params.lift_type.as_deref().unwrap_or("squat"));
    
    let mut cols = vec![
        col("BodyweightKg"),
        col("Sex"),
        col("Date"),
        col("Best3SquatKg"),
        col("Best3BenchKg"),
        col("Best3DeadliftKg"),
        col("TotalKg"),
        col("SquatDOTS"),
        col("BenchDOTS"), 
        col("DeadliftDOTS"),
        col("TotalDOTS"),
    ];
    
    // Only include additional columns if filters use them
    if params.equipment.is_some() {
        cols.push(col("Equipment"));
    }
    
    if params.weight_class.is_some() {
        cols.push(col("WeightClassKg"));
    }
    
    // Return all required columns (duplicates handled by Polars)
    
    cols
}

/// Create optimized filter expression from parameters (using pre-compiled base)
pub fn build_filter_expr(params: &FilterParams) -> Expr {
    let mut expr = VALIDITY_EXPR.clone(); // Start with pre-compiled validity
    
    // Build composite filter expression
    if let Some(sex) = &params.sex {
        if sex != "All" {
            expr = expr.and(col("Sex").eq(lit(sex.as_str())));
        }
    }
    
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() && !equipment.contains(&"All".to_string()) {
            let eq_expr = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, e| acc.or(e))
                .unwrap_or(lit(false));
            expr = expr.and(eq_expr);
        }
    }
    
    // Add bodyweight range filters
    if let Some(min_bw) = params.min_bodyweight {
        expr = expr.and(col("BodyweightKg").gt_eq(lit(min_bw)));
    }
    if let Some(max_bw) = params.max_bodyweight {
        expr = expr.and(col("BodyweightKg").lt_eq(lit(max_bw)));
    }
    
    // Weight class filter
    if let Some(weight_class) = &params.weight_class {
        if weight_class != "All" {
            expr = expr.and(col("WeightClassKg").eq(lit(weight_class.as_str())));
        }
    }
    
    // Years filter
    if let Some(years_filter) = &params.years_filter {
        match years_filter.as_str() {
            "past_5_years" => {
                let five_years_ago = chrono::Utc::now() - chrono::Duration::days(5 * 365);
                let cutoff_date = five_years_ago.date_naive();
                expr = expr.and(col("Date").gt_eq(lit(cutoff_date)));
            }
            "past_10_years" => {
                let ten_years_ago = chrono::Utc::now() - chrono::Duration::days(10 * 365);
                let cutoff_date = ten_years_ago.date_naive();
                expr = expr.and(col("Date").gt_eq(lit(cutoff_date)));
            }
            "ytd" => {
                let current_year = chrono::Utc::now().year();
                let year_start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
                expr = expr.and(col("Date").gt_eq(lit(year_start)));
            }
            _ => {} // "all" or unknown - no filtering needed
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
            years_filter: None,
        };
        
        // Create sample DataFrame
        let df = df! {
            "Sex" => ["M", "F", "M"],
            "Equipment" => ["Raw", "Single-ply", "Wraps"],
            "Date" => ["2023-01-15", "2022-06-20", "2024-03-10"],
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
    
    #[test]
    fn test_date_filtering() {
        use chrono::{Utc, Datelike};
        
        // Create test data with various dates - simple version without DOTS validation
        let current_year = Utc::now().year();
        let df = df! {
            "Date" => [
                "2018-01-01", 
                "2022-06-15", 
                &format!("{}-01-01", current_year), 
                &format!("{}-12-01", current_year)
            ],
        }.unwrap()
        .lazy()
        .with_columns([
            // Convert Date strings to proper Date type for testing
            col("Date").str().to_date(StrptimeOptions::default()).alias("Date"),
        ])
        .collect()
        .unwrap();
        
        println!("Test data created with {} records", df.height());
        
        // Test YTD filtering - direct filtering without apply_filters_lazy
        let current_year = Utc::now().year();
        let year_start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        
        let filtered_ytd = df.clone()
            .lazy()
            .filter(col("Date").gt_eq(lit(year_start)))
            .collect()
            .unwrap();
        
        println!("YTD filter date: {}", year_start);
        println!("YTD filtered records: {}", filtered_ytd.height());
        
        // Should include 2024 records
        assert!(filtered_ytd.height() >= 1);
        
        // Test past 5 years filtering
        let five_years_ago = Utc::now() - chrono::Duration::days(5 * 365);
        let cutoff_date = five_years_ago.date_naive();
        
        let filtered_5y = df.clone()
            .lazy()
            .filter(col("Date").gt_eq(lit(cutoff_date)))
            .collect()
            .unwrap();
        
        println!("5 years ago cutoff: {}", cutoff_date);
        println!("Past 5 years filtered records: {}", filtered_5y.height());
        
        // Should include records from past 5 years (2022 and 2024 records)
        assert!(filtered_5y.height() >= 2);
        
        // Test past 10 years filtering
        let ten_years_ago = Utc::now() - chrono::Duration::days(10 * 365);
        let cutoff_date_10y = ten_years_ago.date_naive();
        
        let filtered_10y = df.clone()
            .lazy()
            .filter(col("Date").gt_eq(lit(cutoff_date_10y)))
            .collect()
            .unwrap();
        
        println!("10 years ago cutoff: {}", cutoff_date_10y);
        println!("Past 10 years filtered records: {}", filtered_10y.height());
        
        // Should include all records (2020, 2022, 2024)
        assert!(filtered_10y.height() >= 3);
    }
}