// main.rs - Updated with maud HTML templating
use axum::{Router, extract::Request, middleware, response::Response, routing::get};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing;

mod arrow_utils;
mod cache;
mod config;
mod data;
mod duckdb_analytics;
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

    // Normal server startup
    tracing::info!(
        "🚀 Starting Iron Insights - High-Performance Powerlifting Analyzer with DOTS..."
    );
    tracing::info!("🎨 Using maud for compile-time HTML templating");

    let config = AppConfig::default();
    let data_processor = DataProcessor::new().with_sample_size(config.sample_size);

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

    // Get the parquet file path for DuckDB before moving data_processor
    let data =
        tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    tracing::info!("📊 Data loaded in {:?}", start.elapsed());

    // Initialize DuckDB analytics engine with the Parquet file
    let duckdb_start = std::time::Instant::now();
    let parquet_path = DataProcessor::new().get_parquet_path();
    let duckdb_analytics = if let Some(parquet_path) = parquet_path {
        match duckdb_analytics::DuckDBAnalytics::from_parquet(&parquet_path) {
            Ok(analytics) => {
                tracing::info!(
                    "🦆 DuckDB analytics engine initialized in {:?}",
                    duckdb_start.elapsed()
                );
                Some(analytics)
            }
            Err(e) => {
                tracing::warn!("⚠️  Could not initialize DuckDB: {}", e);
                tracing::info!("📊 Continuing with Polars-only analytics...");
                None
            }
        }
    } else {
        tracing::info!("📊 No parquet file found, continuing with Polars-only analytics...");
        None
    };

    let mut state = AppState::new(Arc::new(data), config.cache_config());
    if let Some(duckdb) = duckdb_analytics {
        state = state.with_duckdb(duckdb);
    }
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
        // Rankings page
        .route("/rankings", get(serve_rankings_page))
        // 1RM Calculator page
        .route("/1rm", get(serve_onerepmax_page))
        .route("/about", get(serve_about_page))
        .route("/donate", get(serve_donate_page))
        .route("/sharecard", get(serve_sharecard_page))
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route(
            "/api/visualize-arrow",
            axum::routing::post(create_visualizations_arrow),
        )
        .route(
            "/api/visualize-arrow-stream",
            axum::routing::post(create_visualizations_arrow_stream),
        )
        .route("/api/stats", get(get_stats))
        .route("/api/stats-arrow", get(get_stats_arrow))
        .route("/api/share-card", axum::routing::post(generate_share_card))
        // Rankings API
        .route("/api/rankings", get(get_rankings_api))
        // DuckDB-powered analytics endpoints
        .route("/api/percentiles-duckdb", get(get_percentiles_duckdb))
        .route(
            "/api/weight-distribution-duckdb",
            axum::routing::post(get_weight_distribution_duckdb),
        )
        .route(
            "/api/competitive-analysis-duckdb",
            axum::routing::post(get_competitive_analysis_duckdb),
        )
        .route("/api/summary-stats-duckdb", get(get_summary_stats_duckdb))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        // Optimized compression with Brotli, Gzip, and Zstd support
        // By default, CompressionLayer enables all compression algorithms (br, gzip, zstd, deflate)
        // The client's Accept-Encoding header determines which one is used
        .layer(CompressionLayer::new())
        .layer(tower_http::trace::TraceLayer::new_for_http()) // Add tracing layer
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🌐 HTTP/1.1 server running on http://localhost:3000");
    tracing::info!("🔗 WebSocket endpoint available at ws://localhost:3000/ws");
    tracing::info!(
        "🏹 Arrow IPC binary endpoints: /api/visualize-arrow and /api/visualize-arrow-stream"
    );

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
    tracing::info!(
        "   {} update             - Manually check and download latest OpenPowerlifting data",
        args[0]
    );
    tracing::info!(
        "   {} convert <csv>      - Convert CSV to Parquet format for faster loading",
        args[0]
    );
    tracing::info!("   cargo test benchmarks --ignored - Run performance benchmarks");

    // Set up graceful shutdown signal handler
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
        tracing::info!("🛑 Received shutdown signal (Ctrl+C)");
        tracing::info!("🧹 Initiating graceful shutdown...");
    };

    // Run both servers concurrently with graceful shutdown
    tokio::select! {
        result = axum::serve(listener, app_with_alt_svc)
            .with_graceful_shutdown(shutdown_signal) => {
            if let Err(e) = result {
                tracing::error!("HTTP/1.1 server error: {}", e);
            } else {
                tracing::info!("✅ HTTP/1.1 server shut down gracefully");
            }
        }
        _ = http3_handle => {
            tracing::info!("HTTP/3 QUIC server shutdown");
        }
    }

    tracing::info!("👋 Iron Insights shut down successfully");
    Ok(())
}

// Middleware to add Alt-Svc header for HTTP/3 discovery and resource hints
async fn add_alt_svc_header(request: Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;

    // Add Alt-Svc header to advertise HTTP/3 availability
    response
        .headers_mut()
        .insert("Alt-Svc", "h3=\":3443\"; ma=86400".parse().unwrap());

    // Add Link header for HTTP/2 Server Push hints for critical resources
    // Browsers will preload these resources for better performance
    let link_header = concat!(
        "</static/wasm/iron_insights_wasm_bg.wasm>; rel=preload; as=fetch; crossorigin, ",
        "</static/wasm/iron_insights_wasm.js>; rel=modulepreload, ",
        "</static/js/lazy-loader.js>; rel=modulepreload"
    );

    response
        .headers_mut()
        .insert("Link", link_header.parse().unwrap());

    response
}

async fn handle_convert_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        println!("❌ Usage: {} convert <input.csv> [output.parquet]", args[0]);
        println!("   If output is not specified, it will auto-generate from input filename");
        println!();
        println!("Examples:");
        println!(
            "   {} convert data/openpowerlifting-2024-01-15.csv",
            args[0]
        );
        println!(
            "   {} convert data/openpowerlifting-2024-01-15.csv data/custom.parquet",
            args[0]
        );
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
    })
    .await??;

    tracing::info!("✅ Conversion completed successfully!");
    tracing::info!(
        "💡 Next time you start the server, it will automatically use the Parquet file for faster loading"
    );

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
