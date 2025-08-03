// main.rs
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, services::ServeDir};

mod config;
mod data;
mod handlers;
mod models;
mod scoring;
mod ui;

use config::AppConfig;
use data::DataProcessor;
use handlers::*;
use models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Iron Insights - High-Performance Powerlifting Analyzer with DOTS...");
    
    let config = AppConfig::default();
    let data_processor = DataProcessor::new();
    
    let start = std::time::Instant::now();
    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    println!("📊 Data loaded in {:?}", start.elapsed());
    
    let state = AppState::new(Arc::new(data), config.cache_config());
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route("/api/stats", get(get_stats))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Server running on http://localhost:3000");
    println!("💡 Upload your powerlifting CSV to /data/openpowerlifting.csv to get started");
    
    axum::serve(listener, app).await?;
    Ok(())
}