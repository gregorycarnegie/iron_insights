// handlers.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use polars::prelude::*;
use rayon::prelude::*;
use std::time::Instant;

use crate::models::*;
use crate::scoring::calculate_dots_score;
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
    
    // Determine which lift to visualize (default to squat)
    let lift_type = LiftType::from_str(params.lift_type.as_deref().unwrap_or("squat"));
    
    // Generate visualizations
    let histogram_data = create_histogram_data(&filtered_data, &lift_type, false);
    let scatter_data = create_scatter_data(&filtered_data, &lift_type, false);
    let dots_histogram_data = create_histogram_data(&filtered_data, &lift_type, true);
    let dots_scatter_data = create_scatter_data(&filtered_data, &lift_type, true);
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
    serde_json::json!({
        "total_records": state.data.height(),
        "cache_size": state.cache.entry_count(),
        "scoring_system": "DOTS",
        "uptime": "running"
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
    
    df.collect().unwrap_or_else(|_| data.clone())
}

fn create_histogram_data(data: &DataFrame, lift_type: &LiftType, use_dots: bool) -> HistogramData {
    let column = if use_dots {
        lift_type.dots_column()
    } else {
        lift_type.raw_column()
    };
    
    let values: Vec<f32> = data.column(column)
        .map(|col| {
            col.f32()
                .map(|s| s.into_no_null_iter().filter(|&x| x > 0.0).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    if values.is_empty() {
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

    let bodyweight: Vec<f32> = data.column("BodyweightKg")
        .map(|col| col.f32().map(|s| s.into_no_null_iter().collect()).unwrap_or_default())
        .unwrap_or_default();

    let y_values: Vec<f32> = data.column(y_column)
        .map(|col| col.f32().map(|s| s.into_no_null_iter().collect()).unwrap_or_default())
        .unwrap_or_default();
    
    let sex: Vec<String> = data.column("Sex")
        .map(|col| {
            col.str()
                .map(|s| s.into_no_null_iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    
    ScatterData {
        x: bodyweight,
        y: y_values,
        sex,
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

    let dots_values: Vec<f32> = data.column(dots_column)
        .ok()?
        .f32()
        .ok()?
        .into_no_null_iter()
        .filter(|&x| x > 0.0)
        .collect();

    if dots_values.is_empty() {
        return None;
    }

    let below_count = dots_values.iter()
        .filter(|&&dots| dots < user_dots)
        .count();

    let percentile = (below_count as f32 / dots_values.len() as f32) * 100.0 as f32;
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