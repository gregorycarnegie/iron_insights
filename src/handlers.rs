// src/handlers.rs - Refactored with maud HTML templating
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{Json, Response},
};
use maud::Markup;
use tracing::{info, instrument, error};

use crate::{
    arrow_utils::serialize_all_visualization_data,
    cache::{cache_get, cache_put, make_cache_key},
    models::*,
    share_card::{ShareCardData, CardTheme, generate_themed_share_card_svg},
    ui::render_index,
    viz::compute_viz,
};

/// Main HTML page - now using maud templating
#[instrument(skip(_state))]
pub async fn serve_index(State(_state): State<AppState>) -> Markup {
    render_index()
}

/// Main JSON visualization endpoint - thin I/O wrapper
#[instrument(skip(state))]
pub async fn create_visualizations(
    State(state): State<AppState>,
    Json(params): Json<FilterParams>,
) -> Result<Json<VisualizationResponse>, StatusCode> {
    // Generate cache key
    let cache_key = make_cache_key(&params, "json");
    
    // Check cache first
    if let Some(cached) = cache_get::<VisualizationResponse>(&state, &cache_key).await {
        info!("Cache hit for key: {}", cache_key);
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
    
    // Cache the result
    cache_put(&state, &cache_key, &response).await;
    
    Ok(Json(response))
}

/// Arrow IPC visualization endpoint - binary format
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