// src/filters.rs - Centralized lazy filtering logic with column pruning
use crate::models::{FilterParams, LiftType};
use chrono::{self, Datelike};
use lazy_static::lazy_static;
use polars::prelude::*;

// Pre-compiled filter expressions for maximum performance
lazy_static! {
    static ref VALIDITY_EXPR: Expr = col("BodyweightKg")
        .gt(lit(30.0))
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

    let mut lf = df.clone().lazy().select(required_cols); // Prune columns early for I/O efficiency

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
            let eq_filter = equipment
                .iter()
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
            // Convert dropdown value (e.g., "74") to database format (e.g., "74kg")
            let db_weight_class = if weight_class.ends_with('+') {
                format!("{}kg+", weight_class.trim_end_matches('+'))
            } else {
                format!("{}kg", weight_class)
            };
            lf = lf.filter(col("WeightClassKg").eq(lit(db_weight_class.as_str())));
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
