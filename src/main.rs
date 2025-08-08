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
mod websocket;

use config::AppConfig;
use data::DataProcessor;
use handlers::*;
use models::AppState;
use websocket::{WebSocketState, websocket_handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("🚀 Starting Iron Insights - High-Performance Powerlifting Analyzer with DOTS...");
    
    let config = AppConfig::default();
    let data_processor = DataProcessor::new()
        .with_sample_size(config.sample_size);
    
    // Check for data updates first
    println!("🔄 Checking for OpenPowerlifting data updates...");
    match data_processor.check_and_update_data().await {
        Ok(updated) => {
            if updated {
                println!("✅ Data has been updated to latest version!");
            } else {
                println!("✅ Using current data (already up to date)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not check for updates: {}", e);
            println!("📦 Continuing with existing data...");
        }
    }
    
    let start = std::time::Instant::now();
    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    println!("📊 Data loaded in {:?}", start.elapsed());
    
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route("/api/stats", get(get_stats))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Server running on http://localhost:3000");
    println!("🔗 WebSocket endpoint available at ws://localhost:3000/ws");
    println!("💡 Data updates are checked automatically on startup");
    println!("📋 Commands available:");
    println!("   {} update          - Manually check and download latest OpenPowerlifting data", args[0]);
    println!("   {} convert <csv>   - Convert CSV to Parquet format for faster loading", args[0]);
    
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
    
    let input_csv = args[2].clone(); // Clone to own the string
    let output_parquet = args.get(3).cloned(); // Clone Option<String>
    
    // Validate input file exists
    if !std::path::Path::new(&input_csv).exists() {
        println!("❌ Input file not found: {}", input_csv);
        std::process::exit(1);
    }
    
    println!("🔄 Converting CSV to Parquet format...");
    
    let data_processor = DataProcessor::new();
    
    // Run conversion in blocking task since it's CPU intensive
    tokio::task::spawn_blocking(move || {
        data_processor.convert_csv_to_parquet(&input_csv, output_parquet.as_deref())
    }).await??;
    
    println!("✅ Conversion completed successfully!");
    println!("💡 Next time you start the server, it will automatically use the Parquet file for faster loading");
    
    Ok(())
}

async fn handle_update_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Manually updating OpenPowerlifting data...");
    
    let data_processor = DataProcessor::new();
    
    match data_processor.check_and_update_data().await {
        Ok(updated) => {
            if updated {
                println!("✅ Data has been updated to latest version!");
                println!("🚀 You can now restart the server to use the new data.");
            } else {
                println!("✅ Data is already up to date!");
                println!("📊 No update needed.");
            }
        }
        Err(e) => {
            println!("❌ Failed to update data: {}", e);
            println!("🌐 Please check your internet connection and try again.");
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
    
    #[tokio::test]
    async fn test_sample_data_generation() {
        let processor = DataProcessor::new().with_sample_size(100);
        let result = tokio::task::spawn_blocking(move || {
            processor.load_and_preprocess_data()
        }).await;
        
        assert!(result.is_ok());
        let df = result.unwrap().unwrap();
        assert_eq!(df.height(), 100);
        
        // Verify required columns exist - convert to strings for comparison
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
        let dots_male_100kg_500total = calculate_dots_score(500.0, 100.0);
        let dots_female_60kg_300total = calculate_dots_score(300.0, 60.0);
        
        // DOTS scores should be in reasonable range
        assert!(dots_male_100kg_500total > 200.0 && dots_male_100kg_500total < 800.0);
        assert!(dots_female_60kg_300total > 200.0 && dots_female_60kg_300total < 800.0);
        
        println!("Male 100kg, 500kg total: {:.1} DOTS", dots_male_100kg_500total);
        println!("Female 60kg, 300kg total: {:.1} DOTS", dots_female_60kg_300total);
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