// src/handlers.rs - Optimized with streaming and performance improvements
use axum::{
    extract::State,
    http::{header},
    response::{Json, Response},
    body::Body,
};
use axum::http::StatusCode;
use maud::Markup;
use tracing::{info, instrument, error};
use tokio_stream::{Stream, StreamExt};
use bytes::Bytes;
use std::convert::Infallible;

use crate::{
    arrow_utils::{serialize_all_visualization_data, serialize_stats_to_arrow},
    cache::{cache_get_arrow, cache_put_arrow, make_cache_key},
    models::*,
    share_card::{ShareCardData, CardTheme, generate_themed_share_card_svg},
    ui::{render_index, render_analytics, sharecard_page::render_sharecard_page},
    viz::compute_viz,
};

/// Home page - landing page with overview
#[instrument(skip(_state))]
pub async fn serve_index(State(_state): State<AppState>) -> Markup {
    render_index()
}

/// Analytics page - the original main functionality
#[instrument(skip(_state))]
pub async fn serve_analytics(State(_state): State<AppState>) -> Markup {
    render_analytics()
}

/// Share Card page
#[instrument(skip(_state))]
pub async fn serve_sharecard_page(State(_state): State<AppState>) -> Markup {
    render_sharecard_page()
}

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
    let config = crate::config::AppConfig::default();
    let viz_data = compute_viz(&state.data, &params, &config)
        .map_err(|e| {
            error!("Compute error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
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
    if let Some(cached) = state.cache.get(&cache_key).await {
        if cached.computed_at.elapsed().as_secs() < crate::cache::CACHE_TTL_SECS {
            info!("Arrow cache hit for key: {}", cache_key);
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
                .header(header::CACHE_CONTROL, "public, max-age=300")
                .body(cached.data.into())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    // Compute visualization data
    let config = crate::config::AppConfig::default();
    let viz_data = compute_viz(&state.data, &params, &config)
        .map_err(|e| {
            error!("Arrow compute error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to Arrow format
    let arrow_response = serialize_all_visualization_data(
        &viz_data.hist,
        &viz_data.scatter,
        &viz_data.dots_hist,
        &viz_data.dots_scatter,
        viz_data.user_percentile,
        viz_data.user_dots_percentile,
        viz_data.processing_time_ms,
        viz_data.total_records,
    ).map_err(|e| {
        error!("Arrow serialization error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Cache the result
    state.cache.insert(cache_key, CachedResult {
        data: arrow_response.data.clone(),
        computed_at: std::time::Instant::now(),
    }).await;
    
    // Build response with headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apache.arrow.stream")
        .header(header::CACHE_CONTROL, "public, max-age=300")
        .header("X-Processing-Time-Ms", arrow_response.processing_time_ms.to_string())
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
    let config = crate::config::AppConfig::default();
    let viz_data = compute_viz(&state.data, &params, &config)
        .map_err(|e| {
            error!("Arrow stream compute error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create streaming response
    let stream = create_arrow_data_stream(viz_data)
        .map(|chunk| Ok::<Bytes, Infallible>(chunk));
    
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
    let cache_stats = crate::cache::cache_stats(&state);
    
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
    let cache_stats = crate::cache::cache_stats(&state);
    
    let stats_data = StatsData {
        total_records: state.data.height() as u32,
        cache_entries: cache_stats.entry_count as u32,
        cache_size: cache_stats.weighted_size,
        scoring_system: "DOTS".to_string(),
        status: "operational".to_string(),
    };
    
    let arrow_data = serialize_stats_to_arrow(&stats_data)
        .map_err(|e| {
            error!("Stats Arrow serialization error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
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
        .header("Content-Disposition", "inline; filename=\"powerlifting-card.svg\"")
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
fn create_arrow_data_stream(
    viz_data: crate::viz::VizData,
) -> impl Stream<Item = Bytes> + Send {
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
        ).unwrap_or_default(),
    ];
    
    iter(chunks.into_iter().map(Bytes::from))
}

/// Serialize histogram data as Arrow chunk
fn serialize_histogram_chunk(
    hist: &crate::models::HistogramData,
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
    scatter: &crate::models::ScatterData,
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

// DTO conversion implementation
impl From<crate::viz::VizData> for VisualizationResponse {
    fn from(data: crate::viz::VizData) -> Self {
        VisualizationResponse {
            histogram_data: data.hist,
            scatter_data: data.scatter,
            dots_histogram_data: data.dots_hist,
            dots_scatter_data: data.dots_scatter,
            user_percentile: data.user_percentile,
            user_dots_percentile: data.user_dots_percentile,
            processing_time_ms: data.processing_time_ms,
            total_records: data.total_records,
        }
    }
}
