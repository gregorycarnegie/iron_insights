// src/filters.rs - Centralized lazy filtering logic with column pruning
use crate::models::{FilterParams, LiftType};
use chrono::{self, Datelike};
use lazy_static::lazy_static;
use polars::prelude::*;
use tracing::warn;

fn parse_weight_class(value: &str) -> Option<(&'static str, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("All") {
        return None;
    }

    let (system, class_raw) = if let Some((prefix, class)) = trimmed.split_once(':') {
        (prefix.to_lowercase(), class)
    } else {
        ("ipf".to_string(), trimmed)
    };

    let column = match system.as_str() {
        "para" => "ParaWeightClassKg",
        "wp" => "WPWeightClassKg",
        _ => "IPFWeightClassKg",
    };

    let class_clean = class_raw.trim();
    if class_clean.is_empty() {
        return None;
    }

    let db_value = if class_clean.ends_with('+') {
        format!("{}kg+", class_clean.trim_end_matches('+'))
    } else {
        format!("{}kg", class_clean)
    };

    Some((column, db_value))
}

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
    let has_federation = df
        .get_column_names()
        .iter()
        .any(|name| name.as_str() == "Federation");

    // Column pruning: only select required columns based on lift type
    let required_cols = get_required_columns(params, has_federation);

    let mut lf = df.clone().lazy().select(required_cols); // Prune columns early for I/O efficiency

    // Apply user filters first (most selective)

    // Sex filter
    if let Some(sex) = &params.sex
        && sex != "All"
    {
        lf = lf.filter(col("Sex").eq(lit(sex.as_str())));
    }

    // Equipment filter using OR chains (is_in not available in this version)
    if let Some(equipment) = &params.equipment
        && !equipment.is_empty()
        && !equipment.contains(&"All".to_string())
    {
        // Build OR expression for multiple equipment types
        let eq_filter = equipment
            .iter()
            .map(|eq| col("Equipment").eq(lit(eq.clone())))
            .reduce(|acc, expr| acc.or(expr))
            .unwrap_or(lit(true));
        lf = lf.filter(eq_filter);
    }

    // Years filter using the Date column (parsed as dates)
    if let Some(years_filter) = &params.years_filter {
        match years_filter.as_str() {
            "last_5_years" => {
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
            "last_12_months" => {
                // Filter to records from the past 12 months
                let twelve_months_ago = chrono::Utc::now() - chrono::Duration::days(365);
                let cutoff_date = twelve_months_ago.date_naive();
                lf = lf.filter(col("Date").gt_eq(lit(cutoff_date)));
            }
            "current_year" => {
                // Filter to records from current year
                let current_year = chrono::Utc::now().year();
                let year_start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
                let year_end = chrono::NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap();
                lf = lf.filter(
                    col("Date")
                        .gt_eq(lit(year_start))
                        .and(col("Date").lt_eq(lit(year_end))),
                );
            }
            "previous_year" => {
                // Filter to records from previous year
                let previous_year = chrono::Utc::now().year() - 1;
                let year_start = chrono::NaiveDate::from_ymd_opt(previous_year, 1, 1).unwrap();
                let year_end = chrono::NaiveDate::from_ymd_opt(previous_year, 12, 31).unwrap();
                lf = lf.filter(
                    col("Date")
                        .gt_eq(lit(year_start))
                        .and(col("Date").lt_eq(lit(year_end))),
                );
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

    // Federation filter
    if let Some(federation) = &params.federation
        && federation != "all"
    {
        if has_federation {
            lf = lf.filter(col("Federation").eq(lit(federation.to_uppercase().as_str())));
        } else {
            warn!(
                "Federation filter requested but backing data has no Federation column; skipping filter"
            );
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
    if let Some(weight_class) = &params.weight_class
        && weight_class != "All"
    {
        if let Some((column, db_weight_class)) = parse_weight_class(weight_class) {
            lf = lf.filter(col(column).eq(lit(db_weight_class.as_str())));
        }
    }

    // Apply pre-compiled validity filter (most expensive, applied last)
    lf = lf.filter(VALIDITY_EXPR.clone());

    Ok(lf)
}

/// Get minimal required columns based on parameters to reduce I/O
fn get_required_columns(params: &FilterParams, has_federation: bool) -> Vec<Expr> {
    let _lift_type = LiftType::parse(params.lift_type.as_deref().unwrap_or("squat"));

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
        if let Some((column, _)) =
            params.weight_class.as_deref().and_then(|wc| parse_weight_class(wc))
        {
            cols.push(col(column));
        } else {
            cols.push(col("IPFWeightClassKg"));
            cols.push(col("ParaWeightClassKg"));
            cols.push(col("WPWeightClassKg"));
        }
    }

    if has_federation
        && let Some(federation) = params.federation.as_ref()
        && federation != "all"
    {
        cols.push(col("Federation"));
    }

    // Return all required columns (duplicates handled by Polars)
    cols
}
