// tests/benchmarks.rs - Performance benchmarks moved from main.rs
use axum::{Router, routing::get};
use futures_util::{SinkExt, StreamExt};
use iron_insights::{
    config::AppConfig,
    data::DataProcessor,
    handlers::{create_visualizations, create_visualizations_arrow},
    models::{AppState, FilterParams},
    websocket::{WebSocketMessage, WebSocketState, websocket_handler},
    websocket_arrow::{serialize_websocket_message_to_arrow, should_use_arrow_format},
};
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

/// Benchmark Arrow vs JSON API performance
#[tokio::test]
#[ignore] // Use `cargo test --ignored` to run benchmarks
async fn benchmark_arrow_vs_json() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🏁 Starting Arrow vs JSON benchmark...");

    // Load data for benchmarking
    let data_processor = DataProcessor::new().with_sample_size(1000);
    let data =
        tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    let config = AppConfig::default();
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());

    // Start HTTP server
    let app = Router::new()
        .route("/api/visualize", axum::routing::post(create_visualizations))
        .route(
            "/api/visualize-arrow",
            axum::routing::post(create_visualizations_arrow),
        )
        .with_state(state);

    let _http_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Run benchmark
    let http_client = reqwest::Client::new();
    let params = FilterParams::default();
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
    tracing::info!(
        "   JSON:  {} ms/request, {} bytes total",
        json_ms_per_req,
        total_json_bytes
    );
    tracing::info!(
        "   Arrow: {} ms/request, {} bytes total",
        arrow_ms_per_req,
        total_arrow_bytes
    );
    tracing::info!("   Speed improvement: {:.2}x faster", speed_improvement);
    tracing::info!("   Size reduction: {:.1}% smaller", size_reduction * 100.0);

    if speed_improvement > 1.0 {
        tracing::info!(
            "✅ Arrow format is {:.2}x faster than JSON!",
            speed_improvement
        );
    } else {
        tracing::info!(
            "⚠️  Arrow format performance: {:.2}x relative to JSON",
            speed_improvement
        );
    }

    if size_reduction > 0.0 {
        tracing::info!(
            "✅ Arrow format is {:.1}% smaller than JSON!",
            size_reduction * 100.0
        );
    } else {
        tracing::info!(
            "⚠️  Arrow format size: {:.1}% compared to JSON",
            (1.0 - size_reduction) * 100.0
        );
    }

    tracing::info!("✅ Benchmark completed!");

    // Assert performance expectations
    assert!(
        speed_improvement > 0.5,
        "Arrow should be reasonably performant compared to JSON"
    );
    assert!(
        size_reduction > -0.5,
        "Arrow shouldn't be dramatically larger than JSON"
    );

    Ok(())
}

/// Benchmark WebSocket Arrow vs JSON message performance
#[tokio::test]
#[ignore] // Use `cargo test --ignored` to run benchmarks
async fn benchmark_websocket_messages() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🚀 Starting WebSocket Arrow vs JSON benchmark...");

    // Load minimal data for benchmarking
    let data_processor = DataProcessor::new().with_sample_size(100);
    let data =
        tokio::task::spawn_blocking(move || data_processor.load_and_preprocess_data()).await??;
    let config = AppConfig::default();
    let mut state = AppState::new(Arc::new(data), config.cache_config());
    state.websocket_state = Some(WebSocketState::new());

    // Start server with WebSocket support
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state.clone());

    let _server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3002")
            .await
            .unwrap();
        tracing::info!("🌐 Benchmark server started on ws://127.0.0.1:3002/ws");
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let iterations = 50;
    tracing::info!("🔥 Running {} WebSocket message iterations...", iterations);

    // Simulate messages that would be sent via WebSocket
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
    let json_avg_micros =
        total_json_time.as_micros() as f64 / (iterations * test_messages.len()) as f64;
    let arrow_avg_micros =
        total_arrow_time.as_micros() as f64 / (iterations * test_messages.len()) as f64;
    let speed_improvement = json_avg_micros / arrow_avg_micros;

    let avg_json_bytes = total_json_bytes as f64 / (iterations * test_messages.len()) as f64;
    let avg_arrow_bytes = total_arrow_bytes as f64 / (iterations * test_messages.len()) as f64;
    let size_ratio = avg_arrow_bytes / avg_json_bytes;

    tracing::info!("📊 WebSocket Message Benchmark Results:");
    tracing::info!(
        "   JSON:  {:.2} μs/message, {:.1} bytes average",
        json_avg_micros,
        avg_json_bytes
    );
    tracing::info!(
        "   Arrow: {:.2} μs/message, {:.1} bytes average",
        arrow_avg_micros,
        avg_arrow_bytes
    );
    tracing::info!("   Speed improvement: {:.2}x faster", speed_improvement);
    tracing::info!("   Size ratio: {:.1}% of JSON size", size_ratio * 100.0);

    if speed_improvement > 1.0 {
        tracing::info!(
            "✅ Arrow WebSocket messages are {:.2}x faster than JSON!",
            speed_improvement
        );
    } else {
        tracing::info!(
            "⚠️  Arrow performance: {:.2}x relative to JSON",
            speed_improvement
        );
    }

    if size_ratio < 1.0 {
        tracing::info!(
            "✅ Arrow messages are {:.1}% smaller than JSON!",
            (1.0 - size_ratio) * 100.0
        );
    } else {
        tracing::info!(
            "⚠️  Arrow messages are {:.1}% larger than JSON",
            (size_ratio - 1.0) * 100.0
        );
    }

    // Test actual WebSocket connection with both formats
    tracing::info!("🔗 Testing live WebSocket connections...");

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
    json_sink
        .send(WsMessage::Text(serde_json::to_string(&connect_msg)?.into()))
        .await?;
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
    arrow_sink
        .send(WsMessage::Text(serde_json::to_string(&connect_msg)?.into()))
        .await?;
    let _response = arrow_stream.next().await;
    let arrow_connection_time = arrow_ws_start.elapsed();

    tracing::info!("🌐 WebSocket Connection Results:");
    tracing::info!("   JSON connection:  {:?}", json_connection_time);
    tracing::info!("   Arrow connection: {:?}", arrow_connection_time);

    tracing::info!("✅ WebSocket benchmark completed!");
    tracing::info!(
        "🚀 Real-time messages using Arrow binary format will be {}x faster",
        speed_improvement
    );

    // Assert performance expectations
    assert!(
        speed_improvement > 0.5,
        "Arrow WebSocket should be reasonably performant"
    );
    assert!(
        size_ratio < 2.0,
        "Arrow messages shouldn't be dramatically larger"
    );

    Ok(())
}
