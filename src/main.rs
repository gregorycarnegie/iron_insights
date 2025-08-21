// main.rs - Updated with maud HTML templating
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing;

mod arrow_utils;
mod cache;
mod config;
mod data;
mod filters;
mod handlers;
mod models;
mod percentiles;
mod scoring;
mod share_card;
mod ui;
mod viz;
mod websocket;

use config::AppConfig;
use data::DataProcessor;
use handlers::*;
use models::AppState;
use websocket::{WebSocketState, websocket_handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better observability
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();
    
    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Handle convert command
    if args.len() >= 2 && args[1] == "convert" {
        return handle_convert_command(&args).await;
    }
    
    // Handle update command
    if args.len() >= 2 && args[1] == "update" {
        return handle_update_command().await;
    }
    
    // Normal server startup
    tracing::info!("🚀 Starting Iron Insights - High-Performance Powerlifting Analyzer with DOTS...");
    tracing::info!("🎨 Using maud for compile-time HTML templating");
    
    let config = AppConfig::default();
    let data_processor = DataProcessor::new()
        .with_sample_size(config.sample_size);
    
    // Check for data updates first
    tracing::info!("🔄 Checking for OpenPowerlifting data updates...");
    match data_processor.check_and_update_data().await {
        Ok(updated) => {
            if updated {
                tracing::info!("✅ Data has been updated to latest version!");
            } else {
                tracing::info!("✅ Using current data (already up to date)");
            }
        }
        Err(e) => {
            tracing::warn!("⚠️  Could not check for updates: {}", e);
            tracing::info!("📦 Continuing with existing data...");
        }
    }
    
    let start = std::time::Instant::now();
    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    tracing::info!("📊 Data loaded in {:?}", start.elapsed());
    
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());
    
    // Spawn background task for cache cleanup
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            crate::cache::cleanup_expired(&cleanup_state).await;
            tracing::debug!("🧹 Cache cleanup completed");
        }
    });
    
    let app = Router::new()
        // Main page now returns maud Markup instead of static HTML
        .route("/", get(serve_index))
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route("/api/visualize-arrow", axum::routing::post(create_visualizations_arrow))
        .route("/api/visualize-arrow-stream", axum::routing::post(create_visualizations_arrow_stream))
        .route("/api/stats", get(get_stats))
        .route("/api/share-card", axum::routing::post(generate_share_card))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .layer(tower_http::trace::TraceLayer::new_for_http()) // Add tracing layer
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🌐 Server running on http://localhost:3000");
    tracing::info!("🔗 WebSocket endpoint available at ws://localhost:3000/ws");
    tracing::info!("💡 Data updates are checked automatically on startup");
    tracing::info!("📋 Commands available:");
    tracing::info!("   {} update          - Manually check and download latest OpenPowerlifting data", args[0]);
    tracing::info!("   {} convert <csv>   - Convert CSV to Parquet format for faster loading", args[0]);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_convert_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        println!("❌ Usage: {} convert <input.csv> [output.parquet]", args[0]);
        println!("   If output is not specified, it will auto-generate from input filename");
        println!();
        println!("Examples:");
        println!("   {} convert data/openpowerlifting-2024-01-15.csv", args[0]);
        println!("   {} convert data/openpowerlifting-2024-01-15.csv data/custom.parquet", args[0]);
        std::process::exit(1);
    }
    
    let input_csv = args[2].clone();
    let output_parquet = args.get(3).cloned();
    
    // Validate input file exists
    if !std::path::Path::new(&input_csv).exists() {
        println!("❌ Input file not found: {}", input_csv);
        std::process::exit(1);
    }
    
    tracing::info!("🔄 Converting CSV to Parquet format...");
    
    let data_processor = DataProcessor::new();
    
    // Run conversion in blocking task since it's CPU intensive
    tokio::task::spawn_blocking(move || {
        data_processor.convert_csv_to_parquet(&input_csv, output_parquet.as_deref())
    }).await??;
    
    tracing::info!("✅ Conversion completed successfully!");
    tracing::info!("💡 Next time you start the server, it will automatically use the Parquet file for faster loading");
    
    Ok(())
}

async fn handle_update_command() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🔄 Manually updating OpenPowerlifting data...");
    
    let data_processor = DataProcessor::new();
    
    match data_processor.check_and_update_data().await {
        Ok(updated) => {
            if updated {
                tracing::info!("✅ Data has been updated to latest version!");
                tracing::info!("🚀 You can now restart the server to use the new data.");
            } else {
                tracing::info!("✅ Data is already up to date!");
                tracing::info!("📊 No update needed.");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to update data: {}", e);
            tracing::info!("🌐 Please check your internet connection and try again.");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataProcessor;
    use crate::scoring::calculate_dots_score;
    use crate::percentiles::percentile_rank;
    use crate::filters::apply_filters_lazy;
    
    #[tokio::test]
    async fn test_sample_data_generation() {
        let processor = DataProcessor::new().with_sample_size(100);
        let result = tokio::task::spawn_blocking(move || {
            processor.load_and_preprocess_data()
        }).await;
        
        assert!(result.is_ok());
        let df = result.unwrap().unwrap();
        assert_eq!(df.height(), 100);
        
        // Verify required columns exist
        let column_names: Vec<String> = df.get_column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();
        
        assert!(column_names.contains(&"Name".to_string()));
        assert!(column_names.contains(&"Sex".to_string()));
        assert!(column_names.contains(&"BodyweightKg".to_string()));
        assert!(column_names.contains(&"Best3SquatKg".to_string()));
        assert!(column_names.contains(&"SquatDOTS".to_string()));
        assert!(column_names.contains(&"WeightClassKg".to_string()));
    }
    
    #[test]
    fn test_dots_calculation() {
        // Test realistic values
        let dots_male_100kg_500total = calculate_dots_score(500.0, 100.0, "M");
        let dots_female_60kg_300total = calculate_dots_score(300.0, 60.0, "F");
        
        // DOTS scores should be in reasonable range
        assert!(dots_male_100kg_500total > 200.0 && dots_male_100kg_500total < 800.0);
        assert!(dots_female_60kg_300total > 200.0 && dots_female_60kg_300total < 800.0);
        
        println!("Male 100kg, 500kg total: {:.1} DOTS", dots_male_100kg_500total);
        println!("Female 60kg, 300kg total: {:.1} DOTS", dots_female_60kg_300total);
    }
    
    #[test]
    fn test_percentile_calculation() {
        use polars::prelude::*;
        
        let df = df! {
            "TestColumn" => [100.0f32, 200.0, 300.0, 400.0, 500.0],
        }.unwrap();
        
        let p50 = percentile_rank(&df, "TestColumn", Some(300.0));
        assert_eq!(p50, Some(40.0)); // 2 out of 5 values below 300
        
        let p100 = percentile_rank(&df, "TestColumn", Some(600.0));
        assert_eq!(p100, Some(100.0)); // All values below 600
    }
    
    #[tokio::test]
    async fn test_filter_pipeline() {
        use polars::prelude::*;
        use crate::models::FilterParams;
        
        let df = df! {
            "Sex" => ["M", "F", "M"],
            "Equipment" => ["Raw", "Single-ply", "Raw"],
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
        
        let params = FilterParams {
            sex: Some("M".to_string()),
            equipment: Some(vec!["Raw".to_string()]),
            weight_class: None,
            squat: None,
            bench: None,
            deadlift: None,
            bodyweight: None,
            units: None,
            lift_type: None,
            min_bodyweight: None,
            max_bodyweight: None,
            years_filter: None,
        };
        
        let filtered = apply_filters_lazy(&df, &params)
            .unwrap()
            .collect()
            .unwrap();
        
        // Should only have 2 male lifters with Raw equipment
        assert_eq!(filtered.height(), 2);
    }
    
    #[tokio::test]
    async fn test_server_creation() {
        let config = AppConfig::default();
        let data_processor = DataProcessor::new().with_sample_size(10);
        
        let data = tokio::task::spawn_blocking(move || {
            data_processor.load_and_preprocess_data()
        }).await.unwrap().unwrap();
        
        let state = AppState::new(Arc::new(data), config.cache_config());
        
        // Verify state was created successfully
        assert!(state.data.height() > 0);
        assert_eq!(state.cache.entry_count(), 0); // Empty cache initially
    }
}