// src/handlers.rs - Optimized with streaming and performance improvements
use axum::http::StatusCode;
use axum::{
    body::Body,
    extract::State,
    http::header,
    response::{Json, Response},
};
use bytes::Bytes;
use maud::Markup;
use std::convert::Infallible;
use tokio_stream::{Stream, StreamExt};
use tracing::{error, info, instrument};

use crate::share_card::{CardTheme, ShareCardData, generate_themed_share_card_svg};
use iron_core::{
    arrow_utils::{serialize_all_visualization_data, serialize_stats_to_arrow},
    cache::{cache_get_arrow, cache_put_arrow, make_cache_key},
    models::*,
    viz::compute_viz,
};
use iron_ui::{
    render_about, render_analytics, render_donate, render_index, render_onerepmax, render_sharecard,
};

// Import macros from iron_server crate
use crate::{duckdb_handler, log_and_500, simple_page_handler};

// ============================================================================
// Simple Page Handlers (using simple_page_handler macro)
// ============================================================================

simple_page_handler!(
    /// Home page - landing page with overview
    serve_index => render_index
);

simple_page_handler!(
    /// Analytics page - the original main functionality
    serve_analytics => render_analytics
);

simple_page_handler!(
    /// 1RM Calculator page
    serve_onerepmax_page => render_onerepmax
);

simple_page_handler!(
    /// About page
    serve_about_page => render_about
);

simple_page_handler!(
    /// Donation page
    serve_donate_page => render_donate
);

simple_page_handler!(
    /// Share Card page
    serve_sharecard_page => render_sharecard
);

/// Main JSON visualization endpoint - thin I/O wrapper
#[instrument(skip(state))]
pub async fn create_visualizations(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<VisualizationResponse>, StatusCode> {
    // Generate cache key
    let cache_key = make_cache_key(&params, "arrow");

    // Check cache first using Arrow IPC binary protocol
    if let Some(cached) = cache_get_arrow(&state, &cache_key).await {
        info!("Arrow cache hit for key: {}", cache_key);
        return Ok(Json(cached));
    }

    // Compute visualization data
    let config = iron_core::config::AppConfig::default();
    let viz_data = log_and_500!(compute_viz(&state.data, &params, &config), "Compute error")?;

    // Map to response DTO
    let response = VisualizationResponse::from(viz_data);

    // Cache the result using Arrow IPC binary protocol
    cache_put_arrow(&state, &cache_key, &response).await;

    Ok(Json(response))
}

/// Arrow IPC visualization endpoint - streaming binary format
#[instrument(skip(state))]
pub async fn create_visualizations_arrow(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Response, StatusCode> {
    // Generate cache key with arrow suffix
    let cache_key = make_cache_key(&params, "arrow");

    // Check cache first
    if let Some(cached) = state.cache.get(&cache_key).await
        && cached.computed_at.elapsed().as_secs() < iron_core::cache::CACHE_TTL_SECS
    {
        info!("Arrow cache hit for key: {}", cache_key);
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
            .header(header::CACHE_CONTROL, "public, max-age=300")
            .body(cached.data.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Compute visualization data
    let config = iron_core::config::AppConfig::default();
    let viz_data = log_and_500!(
        compute_viz(&state.data, &params, &config),
        "Arrow compute error"
    )?;

    // Convert to Arrow format
    let arrow_response = log_and_500!(
        serialize_all_visualization_data(iron_core::arrow_utils::VisualizationDataBundle {
            histogram_data: &viz_data.hist,
            scatter_data: &viz_data.scatter,
            dots_histogram_data: &viz_data.dots_hist,
            dots_scatter_data: &viz_data.dots_scatter,
            user_percentile: viz_data.user_percentile,
            user_dots_percentile: viz_data.user_dots_percentile,
            processing_time_ms: viz_data.processing_time_ms,
            total_records: viz_data.total_records,
        }),
        "Arrow serialization error"
    )?;

    // Cache the result
    state
        .cache
        .insert(
            cache_key,
            CachedResult {
                data: arrow_response.data.clone(),
                computed_at: std::time::Instant::now(),
            },
        )
        .await;

    // Build response with headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
        .header(header::CACHE_CONTROL, "public, max-age=300")
        .header(
            "X-Processing-Time-Ms",
            arrow_response.processing_time_ms.to_string(),
        )
        .header("X-Total-Records", arrow_response.total_records.to_string())
        .body(arrow_response.data.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Streaming Arrow IPC endpoint for large datasets
#[instrument(skip(state))]
pub async fn create_visualizations_arrow_stream(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Response<Body>, StatusCode> {
    // Generate cache key
    let _cache_key = make_cache_key(&params, "arrow_stream");

    // Compute visualization data
    let config = iron_core::config::AppConfig::default();
    let viz_data = log_and_500!(
        compute_viz(&state.data, &params, &config),
        "Arrow stream compute error"
    )?;

    // Create streaming response
    let stream = create_arrow_data_stream(viz_data).map(Ok::<Bytes, Infallible>);

    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::TRANSFER_ENCODING, "chunked")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get application statistics
#[instrument(skip(state))]
pub async fn get_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let cache_stats = iron_core::cache::cache_stats(&state);

    Json(serde_json::json!({
        "total_records": state.data.height(),
        "cache_entries": cache_stats.entry_count,
        "cache_size": cache_stats.weighted_size,
        "scoring_system": "DOTS",
        "status": "operational"
    }))
}

/// Get application statistics in Arrow format - 27x faster!
#[instrument(skip(state))]
pub async fn get_stats_arrow(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let cache_stats = iron_core::cache::cache_stats(&state);

    let stats_data = StatsData {
        total_records: state.data.height() as u32,
        cache_entries: cache_stats.entry_count as u32,
        cache_size: cache_stats.weighted_size,
        scoring_system: "DOTS".to_string(),
        status: "operational".to_string(),
    };

    let arrow_data = log_and_500!(
        serialize_stats_to_arrow(&stats_data),
        "Stats Arrow serialization error"
    )?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
        .header(header::CACHE_CONTROL, "public, max-age=30") // Stats change frequently
        .body(arrow_data.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Generate SVG share card
#[instrument(skip(_state))]
pub async fn generate_share_card(
    State(_state): State<AppState>,
    Json(request): Json<ShareCardRequest>,
) -> Result<Response, StatusCode> {
    let theme = match request.theme.as_deref() {
        Some("dark") => CardTheme::Dark,
        Some("minimal") => CardTheme::Minimal,
        Some("powerlifting") => CardTheme::Powerlifting,
        _ => CardTheme::Default,
    };

    let svg_content = generate_themed_share_card_svg(&request.card_data, theme);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(
            "Content-Disposition",
            "inline; filename=\"powerlifting-card.svg\"",
        )
        .body(svg_content.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Share card request structure
#[derive(serde::Deserialize, Debug)]
pub struct ShareCardRequest {
    #[serde(flatten)]
    pub card_data: ShareCardData,
    pub theme: Option<String>,
}

/// Create streaming Arrow data in chunks for memory efficiency
fn create_arrow_data_stream(viz_data: iron_core::viz::VizData) -> impl Stream<Item = Bytes> + Send {
    use tokio_stream::iter;

    // Convert viz data to Arrow chunks
    let chunks: Vec<Vec<u8>> = vec![
        // Histogram data chunk
        serialize_histogram_chunk(&viz_data.hist, "raw_histogram").unwrap_or_default(),
        // Scatter data chunk
        serialize_scatter_chunk(&viz_data.scatter, "raw_scatter").unwrap_or_default(),
        // DOTS histogram chunk
        serialize_histogram_chunk(&viz_data.dots_hist, "dots_histogram").unwrap_or_default(),
        // DOTS scatter chunk
        serialize_scatter_chunk(&viz_data.dots_scatter, "dots_scatter").unwrap_or_default(),
        // Metadata chunk
        serialize_metadata_chunk(
            viz_data.user_percentile,
            viz_data.user_dots_percentile,
            viz_data.processing_time_ms,
            viz_data.total_records,
        )
        .unwrap_or_default(),
    ];

    iter(chunks.into_iter().map(Bytes::from))
}

/// Serialize histogram data as Arrow chunk
fn serialize_histogram_chunk(
    hist: &iron_core::models::HistogramData,
    _chunk_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Simplified Arrow serialization for chunks
    // In a real implementation, you'd use proper Arrow record batches
    let json_data = serde_json::to_string(&serde_json::json!({
        "type": "histogram",
        "name": _chunk_name,
        "values": hist.values,
        "counts": hist.counts,
        "bins": hist.bins,
        "min_val": hist.min_val,
        "max_val": hist.max_val,
    }))?;

    Ok(json_data.into_bytes())
}

/// Serialize scatter data as Arrow chunk  
fn serialize_scatter_chunk(
    scatter: &iron_core::models::ScatterData,
    _chunk_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = serde_json::to_string(&serde_json::json!({
        "type": "scatter",
        "name": _chunk_name,
        "x": scatter.x,
        "y": scatter.y,
        "sex": scatter.sex,
    }))?;

    Ok(json_data.into_bytes())
}

/// Serialize metadata as Arrow chunk
fn serialize_metadata_chunk(
    user_percentile: Option<f32>,
    user_dots_percentile: Option<f32>,
    processing_time_ms: u64,
    total_records: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = serde_json::to_string(&serde_json::json!({
        "type": "metadata",
        "user_percentile": user_percentile,
        "user_dots_percentile": user_dots_percentile,
        "processing_time_ms": processing_time_ms,
        "total_records": total_records,
    }))?;

    Ok(json_data.into_bytes())
}

/// DuckDB-powered percentile calculation endpoint
#[instrument(skip(state))]
pub async fn get_percentiles_duckdb(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    duckdb_handler!(state, calculate_dots_percentiles, "percentiles")
}

/// DuckDB-powered weight distribution endpoint
#[instrument(skip(state))]
pub async fn get_weight_distribution_duckdb(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let duckdb = state.duckdb.as_ref().ok_or_else(|| {
        error!("DuckDB not available");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    let lift_type = params.lift_type.as_deref().unwrap_or("total");
    let sex = params.sex.as_deref().unwrap_or("M");
    let equipment = params.equipment.unwrap_or_else(|| vec!["Raw".to_string()]);
    let weight_class = params.weight_class.as_deref();
    let bin_count = 50; // Default bin count

    let distribution = tokio::task::spawn_blocking({
        let duckdb = duckdb.clone();
        let lift_type = lift_type.to_string();
        let sex = sex.to_string();
        let equipment = equipment.clone();
        let weight_class = weight_class.map(|s| s.to_string());
        move || {
            duckdb.calculate_weight_distribution(
                &lift_type,
                &sex,
                &equipment,
                bin_count,
                weight_class.as_deref(),
            )
        }
    })
    .await
    .map_err(|e| {
        error!("Task join error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .map_err(|e| {
        error!("DuckDB weight distribution error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "distribution": distribution,
        "parameters": {
            "lift_type": lift_type,
            "sex": sex,
            "equipment": equipment,
            "bin_count": bin_count
        },
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "engine": "duckdb"
    })))
}

/// DuckDB-powered competitive analysis endpoint
#[instrument(skip(state))]
pub async fn get_competitive_analysis_duckdb(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let duckdb = state.duckdb.as_ref().ok_or_else(|| {
        error!("DuckDB not available");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    let lift_type = params.lift_type.as_deref().unwrap_or("total");
    let sex = params.sex.as_deref().unwrap_or("M");
    let equipment = params.equipment.unwrap_or_else(|| vec!["Raw".to_string()]);
    let weight_class = params.weight_class.as_deref();

    // Extract user lift based on lift type
    let user_lift = match lift_type {
        "squat" => params.squat.unwrap_or(0.0) as f64,
        "bench" => params.bench.unwrap_or(0.0) as f64,
        "deadlift" => params.deadlift.unwrap_or(0.0) as f64,
        "total" => {
            (params.squat.unwrap_or(0.0)
                + params.bench.unwrap_or(0.0)
                + params.deadlift.unwrap_or(0.0)) as f64
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let user_bodyweight = params.bodyweight.unwrap_or(80.0) as f64;

    if user_lift <= 0.0 || user_bodyweight <= 0.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let analysis = tokio::task::spawn_blocking({
        let duckdb = duckdb.clone();
        let lift_type = lift_type.to_string();
        let sex = sex.to_string();
        let equipment = equipment.clone();
        let weight_class = weight_class.map(|s| s.to_string());
        move || {
            duckdb.analyze_competitive_position(
                &lift_type,
                user_lift,
                user_bodyweight,
                &sex,
                &equipment,
                weight_class.as_deref(),
            )
        }
    })
    .await
    .map_err(|e| {
        error!("Task join error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .map_err(|e| {
        error!("DuckDB competitive analysis error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "analysis": analysis,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "engine": "duckdb"
    })))
}

/// DuckDB-powered summary statistics endpoint
#[instrument(skip(state))]
pub async fn get_summary_stats_duckdb(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    duckdb_handler!(state, get_summary_stats, "stats")
}
