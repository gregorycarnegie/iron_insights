// main.rs - Updated with maud HTML templating
use axum::{routing::get, Router, middleware, extract::Request, response::Response};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing;

mod arrow_utils;
mod cache;
mod config;
mod data;
mod filters;
mod handlers;
mod http3_server;
mod models;
mod percentiles;
mod scoring;
mod share_card;
mod ui;
mod viz;
mod websocket;
mod websocket_arrow;

use config::AppConfig;
use data::DataProcessor;
use handlers::*;
use http3_server::Http3Server;
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
    
    // Handle arrow benchmark command  
    if args.len() >= 2 && args[1] == "benchmark-arrow" {
        return handle_benchmark_arrow_command().await;
    }
    
    // Handle websocket benchmark command
    if args.len() >= 2 && args[1] == "benchmark-websocket" {
        return handle_benchmark_websocket_command().await;
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
        // Home page
        .route("/", get(serve_index))
        // Analytics page (moved from root)
        .route("/analytics", get(serve_analytics))
        // 1RM Calculator page
        .route("/1rm", get(serve_onerepmax_page))
        .route("/sharecard", get(serve_sharecard_page))
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route("/api/visualize-arrow", axum::routing::post(create_visualizations_arrow))
        .route("/api/visualize-arrow-stream", axum::routing::post(create_visualizations_arrow_stream))
        .route("/api/stats", get(get_stats))
        .route("/api/stats-arrow", get(get_stats_arrow))
        .route("/api/share-card", axum::routing::post(generate_share_card))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .layer(tower_http::trace::TraceLayer::new_for_http()) // Add tracing layer
        .with_state(state.clone());
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🌐 HTTP/1.1 server running on http://localhost:3000");
    tracing::info!("🔗 WebSocket endpoint available at ws://localhost:3000/ws");
    tracing::info!("🏹 Arrow IPC binary endpoints: /api/visualize-arrow and /api/visualize-arrow-stream");
    
    // Add Alt-Svc header to existing HTTP server for HTTP/3 discovery
    let app_with_alt_svc = app.layer(axum::middleware::from_fn(add_alt_svc_header));
    
    // Start HTTP/3 QUIC server on port 3443 (UDP)
    let http3_state = state.clone();
    let http3_server = Http3Server::new(http3_state, 3443);
    let http3_handle = tokio::spawn(async move {
        if let Err(e) = http3_server.run().await {
            tracing::error!("HTTP/3 QUIC server error: {}", e);
        }
    });
    
    tracing::info!("🚀 HTTP/3 QUIC server running on UDP port 3443");
    tracing::info!("💡 Browsers will discover HTTP/3 via Alt-Svc header on HTTP/1.1 server");
    tracing::info!("📋 Visit http://localhost:3000 and check for 'Alt-Svc: h3=\":3443\"' header");
    tracing::info!("💡 Data updates are checked automatically on startup");
    tracing::info!("📋 Commands available:");
    tracing::info!("   {} update             - Manually check and download latest OpenPowerlifting data", args[0]);
    tracing::info!("   {} convert <csv>      - Convert CSV to Parquet format for faster loading", args[0]);
    tracing::info!("   {} benchmark-arrow    - Benchmark Arrow vs JSON performance", args[0]);
    tracing::info!("   {} benchmark-websocket - Benchmark WebSocket Arrow vs JSON performance", args[0]);
    
    // Run both servers concurrently
    tokio::select! {
        result = axum::serve(listener, app_with_alt_svc) => {
            if let Err(e) = result {
                tracing::error!("HTTP/1.1 server error: {}", e);
            }
        }
        _ = http3_handle => {
            tracing::info!("HTTP/3 QUIC server shutdown");
        }
    }
    
    Ok(())
}


// Middleware to add Alt-Svc header for HTTP/3 discovery
async fn add_alt_svc_header(
    request: Request,
    next: middleware::Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add Alt-Svc header to advertise HTTP/3 availability
    response.headers_mut().insert(
        "Alt-Svc",
        "h3=\":3443\"; ma=86400"
            .parse()
            .unwrap(),
    );
    
    response
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

async fn handle_benchmark_arrow_command() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🏁 Starting Arrow vs JSON benchmark...");
    
    // Load data for benchmarking
    let data_processor = DataProcessor::new().with_sample_size(1000);
    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    let config = AppConfig::default();
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());
    
    // Start HTTP server
    let app = Router::new()
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route("/api/visualize-arrow", axum::routing::post(create_visualizations_arrow))
        .with_state(state);
        
    let _http_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Run benchmark
    let http_client = reqwest::Client::new();
    let params = crate::models::FilterParams::default();
    let iterations = 10;
    
    tracing::info!("🔥 Running {} iterations for each format...", iterations);
    
    // Benchmark JSON
    let json_start = std::time::Instant::now();
    let mut total_json_bytes = 0;
    for _i in 0..iterations {
        let response = http_client
            .post("http://localhost:3001/api/visualize")
            .json(&params)
            .send()
            .await?;
        let bytes = response.bytes().await?;
        total_json_bytes += bytes.len();
    }
    let json_duration = json_start.elapsed();
    
    // Benchmark Arrow
    let arrow_start = std::time::Instant::now();
    let mut total_arrow_bytes = 0;
    for _i in 0..iterations {
        let response = http_client
            .post("http://localhost:3001/api/visualize-arrow")
            .json(&params)
            .send()
            .await?;
        let bytes = response.bytes().await?;
        total_arrow_bytes += bytes.len();
    }
    let arrow_duration = arrow_start.elapsed();
    
    // Calculate results
    let json_ms_per_req = json_duration.as_millis() / iterations as u128;
    let arrow_ms_per_req = arrow_duration.as_millis() / iterations as u128;
    let speed_improvement = json_duration.as_millis() as f64 / arrow_duration.as_millis() as f64;
    let size_reduction = 1.0 - (total_arrow_bytes as f64 / total_json_bytes as f64);
    
    tracing::info!("📊 Benchmark Results:");
    tracing::info!("   JSON:  {} ms/request, {} bytes total", json_ms_per_req, total_json_bytes);
    tracing::info!("   Arrow: {} ms/request, {} bytes total", arrow_ms_per_req, total_arrow_bytes);
    tracing::info!("   Speed improvement: {:.2}x faster", speed_improvement);
    tracing::info!("   Size reduction: {:.1}% smaller", size_reduction * 100.0);
    
    if speed_improvement > 1.0 {
        tracing::info!("✅ Arrow format is {:.2}x faster than JSON!", speed_improvement);
    } else {
        tracing::info!("⚠️  Arrow format performance: {:.2}x relative to JSON", speed_improvement);
    }
    
    if size_reduction > 0.0 {
        tracing::info!("✅ Arrow format is {:.1}% smaller than JSON!", size_reduction * 100.0);
    } else {
        tracing::info!("⚠️  Arrow format size: {:.1}% compared to JSON", (1.0 - size_reduction) * 100.0);
    }
    
    tracing::info!("✅ Benchmark completed!");
    Ok(())
}

async fn handle_benchmark_websocket_command() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🚀 Starting WebSocket Arrow vs JSON benchmark...");
    
    // Load minimal data for benchmarking
    let data_processor = DataProcessor::new().with_sample_size(100);
    let data = tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    let config = AppConfig::default();
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());
    
    // Start server with WebSocket support
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state.clone());
        
    let _server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3002").await.unwrap();
        tracing::info!("🌐 Benchmark server started on ws://127.0.0.1:3002/ws");
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let iterations = 50;
    tracing::info!("🔥 Running {} WebSocket message iterations...", iterations);
    
    // Simulate messages that would be sent via WebSocket
    use crate::websocket::WebSocketMessage;
    use crate::websocket_arrow::{serialize_websocket_message_to_arrow, should_use_arrow_format};
    
    let test_messages = vec![
        WebSocketMessage::StatsUpdate {
            active_users: 25,
            total_connections: 100,
            server_load: 0.75,
        },
        WebSocketMessage::DotsCalculation {
            lift_kg: 200.5,
            bodyweight_kg: 80.0,
            dots_score: 456.7,
            strength_level: "Advanced".to_string(),
            percentile: Some(85.2),
        },
        WebSocketMessage::UserUpdate {
            bodyweight: Some(75.5),
            squat: Some(180.0),
            bench: Some(120.0),
            deadlift: Some(220.0),
            lift_type: "Raw".to_string(),
            sex: Some("M".to_string()),
        },
    ];
    
    let mut total_json_time = std::time::Duration::ZERO;
    let mut total_arrow_time = std::time::Duration::ZERO;
    let mut total_json_bytes = 0usize;
    let mut total_arrow_bytes = 0usize;
    
    for iteration in 0..iterations {
        for message in &test_messages {
            // Benchmark JSON serialization
            let json_start = std::time::Instant::now();
            let json_data = serde_json::to_string(message)?;
            let json_bytes = json_data.len();
            total_json_time += json_start.elapsed();
            total_json_bytes += json_bytes;
            
            // Benchmark Arrow serialization (only for messages that benefit)
            if should_use_arrow_format(message) {
                let arrow_start = std::time::Instant::now();
                let arrow_data = serialize_websocket_message_to_arrow(message).unwrap();
                let arrow_bytes = arrow_data.len();
                total_arrow_time += arrow_start.elapsed();
                total_arrow_bytes += arrow_bytes;
            } else {
                // For messages that don't use Arrow, count JSON time/size for both
                total_arrow_time += json_start.elapsed();
                total_arrow_bytes += json_bytes;
            }
        }
        
        if iteration % 10 == 0 {
            tracing::info!("📊 Completed {} iterations...", iteration + 1);
        }
    }
    
    // Calculate performance metrics
    let json_avg_micros = total_json_time.as_micros() as f64 / (iterations * test_messages.len()) as f64;
    let arrow_avg_micros = total_arrow_time.as_micros() as f64 / (iterations * test_messages.len()) as f64;
    let speed_improvement = json_avg_micros / arrow_avg_micros;
    
    let avg_json_bytes = total_json_bytes as f64 / (iterations * test_messages.len()) as f64;
    let avg_arrow_bytes = total_arrow_bytes as f64 / (iterations * test_messages.len()) as f64;
    let size_ratio = avg_arrow_bytes / avg_json_bytes;
    
    tracing::info!("📊 WebSocket Message Benchmark Results:");
    tracing::info!("   JSON:  {:.2} μs/message, {:.1} bytes average", json_avg_micros, avg_json_bytes);
    tracing::info!("   Arrow: {:.2} μs/message, {:.1} bytes average", arrow_avg_micros, avg_arrow_bytes);
    tracing::info!("   Speed improvement: {:.2}x faster", speed_improvement);
    tracing::info!("   Size ratio: {:.1}% of JSON size", size_ratio * 100.0);
    
    if speed_improvement > 1.0 {
        tracing::info!("✅ Arrow WebSocket messages are {:.2}x faster than JSON!", speed_improvement);
    } else {
        tracing::info!("⚠️  Arrow performance: {:.2}x relative to JSON", speed_improvement);
    }
    
    if size_ratio < 1.0 {
        tracing::info!("✅ Arrow messages are {:.1}% smaller than JSON!", (1.0 - size_ratio) * 100.0);
    } else {
        tracing::info!("⚠️  Arrow messages are {:.1}% larger than JSON", (size_ratio - 1.0) * 100.0);
    }
    
    // Test actual WebSocket connection with both formats
    tracing::info!("🔗 Testing live WebSocket connections...");
    
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
    use futures_util::{SinkExt, StreamExt};
    
    // Test JSON WebSocket
    let json_ws_start = std::time::Instant::now();
    let (json_ws_stream, _) = connect_async("ws://127.0.0.1:3002/ws").await?;
    let (mut json_sink, mut json_stream) = json_ws_stream.split();
    
    // Send connect message without Arrow support
    let connect_msg = WebSocketMessage::Connect {
        session_id: "benchmark_json".to_string(),
        user_agent: Some("benchmark-tool/1.0".to_string()),
        supports_arrow: Some(false),
    };
    json_sink.send(WsMessage::Text(serde_json::to_string(&connect_msg)?.into())).await?;
    let _response = json_stream.next().await;
    let json_connection_time = json_ws_start.elapsed();
    
    // Test Arrow WebSocket  
    let arrow_ws_start = std::time::Instant::now();
    let (arrow_ws_stream, _) = connect_async("ws://127.0.0.1:3002/ws").await?;
    let (mut arrow_sink, mut arrow_stream) = arrow_ws_stream.split();
    
    // Send connect message with Arrow support
    let connect_msg = WebSocketMessage::Connect {
        session_id: "benchmark_arrow".to_string(),
        user_agent: Some("benchmark-tool/1.0".to_string()),
        supports_arrow: Some(true),
    };
    arrow_sink.send(WsMessage::Text(serde_json::to_string(&connect_msg)?.into())).await?;
    let _response = arrow_stream.next().await;
    let arrow_connection_time = arrow_ws_start.elapsed();
    
    tracing::info!("🌐 WebSocket Connection Results:");
    tracing::info!("   JSON connection:  {:?}", json_connection_time);
    tracing::info!("   Arrow connection: {:?}", arrow_connection_time);
    
    tracing::info!("✅ WebSocket benchmark completed!");
    tracing::info!("🚀 Real-time messages using Arrow binary format will be {}x faster", speed_improvement);
    
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
