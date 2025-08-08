// handlers.rs - Fixed with better DOTS data handling and debugging
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{Html, Json, Response},
};
use polars::prelude::*;
use rayon::prelude::*;
use std::time::Instant;

use crate::models::*;
use crate::scoring::calculate_dots_score;
use crate::share_card::{ShareCardData, CardTheme, generate_themed_share_card_svg};
use crate::ui::HTML_TEMPLATE;

pub async fn create_visualizations(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<VisualizationResponse>, StatusCode> {
    let start = Instant::now();
    
    // Generate cache key
    let cache_key = format!("{:?}", params);
    
    // Check cache first
    if let Some(cached) = state.cache.get(&cache_key).await {
        if cached.computed_at.elapsed().as_secs() < 300 { // 5-minute cache
            let response: VisualizationResponse = 
                serde_json::from_slice(&cached.data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(response));
        }
    }
    
    // Apply filters
    let filtered_data = apply_filters_fast(&state.data, &params);
    
    // Debug: Print filtered data info
    println!("🔍 Filtered data: {} records", filtered_data.height());
    if filtered_data.height() > 0 {
        // Check if DOTS columns exist and have valid data
        for col_name in ["SquatDOTS", "BenchDOTS", "DeadliftDOTS", "TotalDOTS"] {
            if let Ok(col) = filtered_data.column(col_name) {
                if let Ok(f32_series) = col.f32() {
                    let valid_count = f32_series.into_no_null_iter()
                        .filter(|&x| x.is_finite() && x > 0.0)
                        .count();
                    println!("📊 {}: {} valid values", col_name, valid_count);
                } else {
                    println!("❌ {} column is not f32", col_name);
                }
            } else {
                println!("❌ {} column missing", col_name);
            }
        }
    }
    
    // Determine which lift to visualize (default to squat)
    let lift_type = LiftType::from_str(params.lift_type.as_deref().unwrap_or("squat"));
    
    // Generate visualizations
    let histogram_data = create_histogram_data(&filtered_data, &lift_type, false);
    let scatter_data = create_scatter_data(&filtered_data, &lift_type, false);
    let dots_histogram_data = create_histogram_data(&filtered_data, &lift_type, true);
    let dots_scatter_data = create_scatter_data(&filtered_data, &lift_type, true);
    
    // Debug DOTS data
    println!("📈 Raw histogram: {} values", histogram_data.values.len());
    println!("📈 DOTS histogram: {} values", dots_histogram_data.values.len());
    println!("📊 Raw scatter: {} points", scatter_data.x.len());
    println!("📊 DOTS scatter: {} points", dots_scatter_data.x.len());
    
    let user_percentile = calculate_user_percentile(&filtered_data, &params, &lift_type);
    let user_dots_percentile = calculate_user_percentile_dots(&filtered_data, &params, &lift_type);
    
    let response = VisualizationResponse {
        histogram_data,
        scatter_data,
        dots_histogram_data,
        dots_scatter_data,
        user_percentile,
        user_dots_percentile,
        processing_time_ms: start.elapsed().as_millis() as u64,
        total_records: filtered_data.height(),
    };
    
    // Cache the result
    let cached_data = serde_json::to_vec(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.cache.insert(cache_key, CachedResult {
        data: cached_data,
        computed_at: Instant::now(),
    }).await;
    
    Ok(Json(response))
}

pub async fn serve_index() -> Html<&'static str> {
    Html(HTML_TEMPLATE)
}

pub async fn get_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Add more detailed stats including DOTS data health
    let mut dots_stats = std::collections::HashMap::new();
    
    for (lift, col) in [
        ("squat", "SquatDOTS"),
        ("bench", "BenchDOTS"), 
        ("deadlift", "DeadliftDOTS"),
        ("total", "TotalDOTS")
    ] {
        if let Ok(column) = state.data.column(col) {
            if let Ok(f32_series) = column.f32() {
                let valid_count = f32_series.into_no_null_iter()
                    .filter(|&x| x.is_finite() && x > 0.0)
                    .count();
                dots_stats.insert(lift, valid_count);
            }
        }
    }
    
    serde_json::json!({
        "total_records": state.data.height(),
        "cache_size": state.cache.entry_count(),
        "scoring_system": "DOTS",
        "uptime": "running",
        "dots_valid_records": dots_stats
    }).into()
}

fn apply_filters_fast(data: &DataFrame, params: &FilterParams) -> DataFrame {
    let mut df = data.clone().lazy();
    
    // Apply filters using Polars' optimized expressions
    if let Some(sex) = &params.sex {
        if sex != "All" {
            df = df.filter(col("Sex").eq(lit(sex.clone())));
        }
    }
    
    if let Some(equipment) = &params.equipment {
        if !equipment.is_empty() {
            let eq_filter = equipment.iter()
                .map(|eq| col("Equipment").eq(lit(eq.clone())))
                .reduce(|acc, expr| acc.or(expr))
                .unwrap();
            df = df.filter(eq_filter);
        }
    }
    
    // IMPORTANT: Filter out invalid DOTS values that might cause plotting issues
    df = df.filter(
        col("BodyweightKg").gt(0.0)
            .and(col("BodyweightKg").lt(300.0)) // Reasonable bodyweight range
            .and(col("Best3SquatKg").gt(0.0))
            .and(col("Best3BenchKg").gt(0.0))
            .and(col("Best3DeadliftKg").gt(0.0))
    );
    
    let result = df.collect().unwrap_or_else(|e| {
        println!("❌ Filter error: {}", e);
        data.clone()
    });
    
    println!("🔍 Applied filters: {} -> {} records", data.height(), result.height());
    result
}

fn create_histogram_data(data: &DataFrame, lift_type: &LiftType, use_dots: bool) -> HistogramData {
    let column = if use_dots {
        lift_type.dots_column()
    } else {
        lift_type.raw_column()
    };
    
    println!("📊 Creating histogram for column: {}", column);
    
    let values: Vec<f32> = data.column(column)
        .map(|col| {
            col.f32()
                .map(|s| {
                    let filtered: Vec<f32> = s.into_no_null_iter()
                        .filter(|&x| x.is_finite() && x > 0.0)
                        .collect();
                    println!("📈 Column {}: {} valid values (finite and > 0)", column, filtered.len());
                    filtered
                })
                .unwrap_or_else(|e| {
                    println!("❌ Error reading column {}: {}", column, e);
                    vec![]
                })
        })
        .unwrap_or_else(|e| {
            println!("❌ Column {} not found: {}", column, e);
            vec![]
        });
    
    if values.is_empty() {
        println!("⚠️  No valid data for histogram in column: {}", column);
        return HistogramData { 
            values: vec![], 
            counts: vec![], 
            bins: vec![],
            min_val: 0.0,
            max_val: 0.0,
        };
    }
    
    // Fast parallel histogram calculation
    let (min_val, max_val) = values.par_iter()
        .fold(|| (f32::INFINITY, f32::NEG_INFINITY), |acc, &x| (acc.0.min(x), acc.1.max(x)))
        .reduce(|| (f32::INFINITY, f32::NEG_INFINITY), |a, b| (a.0.min(b.0), a.1.max(b.1)));

    println!("📊 Range for {}: {:.1} - {:.1}", column, min_val, max_val);
    
    let num_bins = 50;
    let bin_width = (max_val - min_val) / num_bins as f32;
    let mut bins = vec![0u32; num_bins];
    
    // Sequential binning for correctness
    for &val in &values {
        let bin_idx = ((val - min_val) / bin_width).floor() as usize;
        let bin_idx = bin_idx.min(num_bins - 1);
        bins[bin_idx] += 1;
    }

    let bin_edges: Vec<f32> = (0..=num_bins)
        .map(|i| min_val + i as f32 * bin_width)
        .collect();
    
    println!("✅ Histogram created with {} values, {} bins", values.len(), bins.len());
    
    HistogramData {
        values,
        counts: bins,
        bins: bin_edges,
        min_val,
        max_val,
    }
}

fn create_scatter_data(data: &DataFrame, lift_type: &LiftType, use_dots: bool) -> ScatterData {
    let y_column = if use_dots {
        lift_type.dots_column()
    } else {
        lift_type.raw_column()
    };

    println!("📊 Creating scatter plot for column: {}", y_column);

    let bodyweight: Vec<f32> = data.column("BodyweightKg")
        .map(|col| col.f32().map(|s| s.into_no_null_iter().collect()).unwrap_or_default())
        .unwrap_or_default();

    let y_values: Vec<f32> = data.column(y_column)
        .map(|col| {
            col.f32()
                .map(|s| {
                    let values: Vec<f32> = s.into_no_null_iter()
                        .filter(|&x| x.is_finite() && x > 0.0)
                        .collect();
                    println!("📊 Scatter Y values ({}): {} valid", y_column, values.len());
                    values
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    let sex: Vec<String> = data.column("Sex")
        .map(|col| {
            col.str()
                .map(|s| s.into_no_null_iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    // Ensure all vectors have the same length by taking the minimum
    let min_len = bodyweight.len().min(y_values.len()).min(sex.len());
    
    println!("📊 Scatter data lengths - BW: {}, Y: {}, Sex: {}, Min: {}", 
             bodyweight.len(), y_values.len(), sex.len(), min_len);
    
    ScatterData {
        x: bodyweight.into_iter().take(min_len).collect(),
        y: y_values.into_iter().take(min_len).collect(),
        sex: sex.into_iter().take(min_len).collect(),
    }
}

fn calculate_user_percentile(data: &DataFrame, params: &FilterParams, lift_type: &LiftType) -> Option<f32> {
    let user_lift = get_user_lift_value(params, lift_type)?;
    let column_name = lift_type.raw_column();

    let lift_values: Vec<f32> = data.column(column_name)
        .ok()?
        .f32()
        .ok()?
        .into_no_null_iter()
        .filter(|&x| x > 0.0)
        .collect();

    if lift_values.is_empty() {
        return None;
    }

    let below_count = lift_values.iter()
        .filter(|&&lift| lift < user_lift)
        .count();

    let percentile = (below_count as f32 / lift_values.len() as f32) * 100.0 as f32;
    Some(percentile.round())
}

fn calculate_user_percentile_dots(data: &DataFrame, params: &FilterParams, lift_type: &LiftType) -> Option<f32> {
    let user_bodyweight = params.bodyweight?;
    let user_lift = get_user_lift_value(params, lift_type)?;

    let user_dots = calculate_dots_score(user_lift, user_bodyweight);
    let dots_column = lift_type.dots_column();

    println!("🎯 User DOTS calculation: lift={}, bw={}, dots={:.2}", 
             user_lift, user_bodyweight, user_dots);

    let dots_values: Vec<f32> = data.column(dots_column)
        .ok()?
        .f32()
        .ok()?
        .into_no_null_iter()
        .filter(|&x| x.is_finite() && x > 0.0)
        .collect();

    println!("📊 DOTS percentile calculation: {} valid values", dots_values.len());

    if dots_values.is_empty() {
        return None;
    }

    let below_count = dots_values.iter()
        .filter(|&&dots| dots < user_dots)
        .count();

    let percentile = (below_count as f32 / dots_values.len() as f32) * 100.0 as f32;
    println!("📊 User DOTS percentile: {:.1}%", percentile);
    Some(percentile.round())
}

fn get_user_lift_value(params: &FilterParams, lift_type: &LiftType) -> Option<f32> {
    match lift_type {
        LiftType::Squat => params.squat,
        LiftType::Bench => params.bench,
        LiftType::Deadlift => params.deadlift,
        LiftType::Total => {
            let squat = params.squat.unwrap_or(0.0);
            let bench = params.bench.unwrap_or(0.0);
            let deadlift = params.deadlift.unwrap_or(0.0);
            
            if squat > 0.0 || bench > 0.0 || deadlift > 0.0 {
                Some(squat + bench + deadlift)
            } else {
                None
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ShareCardRequest {
    #[serde(flatten)]
    pub card_data: ShareCardData,
    pub theme: Option<String>,
}

pub async fn generate_share_card(
    State(_state): State<AppState>,
    Json(request): Json<ShareCardRequest>,
) -> Result<Response, StatusCode> {
    // Determine theme based on request or default to standard
    let theme = match request.theme.as_deref() {
        Some("dark") => CardTheme::Dark,
        Some("minimal") => CardTheme::Minimal,
        Some("powerlifting") => CardTheme::Powerlifting,
        _ => CardTheme::Default,
    };
    
    // Generate the SVG
    let svg_content = generate_themed_share_card_svg(&request.card_data, theme);
    
    // Return SVG with proper headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("Content-Disposition", "inline; filename=\"powerlifting-card.svg\"")
        .body(svg_content.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}