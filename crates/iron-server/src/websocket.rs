// websocket.rs - Real-time WebSocket support for Iron Insights
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use dashmap::DashMap;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::websocket_arrow::{serialize_websocket_message_to_arrow, should_use_arrow_format};
use iron_core::models::AppState;
use iron_scoring::calculate_dots_score;

/// WebSocket connection manager
pub type Connections = Arc<DashMap<String, ConnectionInfo>>;
pub type Broadcaster = Arc<broadcast::Sender<BroadcastMessage>>;

/// Connection information for each WebSocket client
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub last_activity: u64,
    pub user_agent: Option<String>,
}

/// Messages sent between server and clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Client connects and provides session info
    Connect {
        session_id: String,
        user_agent: Option<String>,
        supports_arrow: Option<bool>, // Client indicates Arrow support
    },
    /// Client updates their lift data
    UserUpdate {
        bodyweight: Option<f32>,
        squat: Option<f32>,
        bench: Option<f32>,
        deadlift: Option<f32>,
        lift_type: String,
        sex: Option<String>,
    },
    /// Server sends real-time stats
    StatsUpdate {
        active_users: usize,
        total_connections: usize,
        server_load: f32,
    },
    /// Server broadcasts live user activity
    ActivityFeed {
        activity_type: String,
        data: serde_json::Value,
        timestamp: u64,
    },
    /// Real-time DOTS calculation results
    DotsCalculation {
        lift_kg: f32,
        bodyweight_kg: f32,
        dots_score: f32,
        strength_level: String,
        percentile: Option<f32>,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Heartbeat to keep connection alive
    Ping,
    Pong,
}

/// Broadcast messages sent to all connected clients
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum BroadcastMessage {
    /// User activity update
    UserActivity {
        user_count: usize,
        recent_calculations: Vec<RecentCalculation>,
    },
    /// Server metrics update
    #[allow(dead_code)]
    ServerMetrics {
        active_connections: usize,
        calculations_per_minute: usize,
        data_freshness: String,
    },
    /// Live leaderboard updates
    #[allow(dead_code)]
    LeaderboardUpdate {
        top_dots: Vec<LeaderboardEntry>,
        recent_prs: Vec<PersonalRecord>,
    },
}

/// Recent DOTS calculation for activity feed
#[derive(Debug, Clone, Serialize)]
pub struct RecentCalculation {
    pub dots_score: f32,
    pub lift_type: String,
    pub strength_level: String,
    pub timestamp: u64,
}

/// Leaderboard entry for live updates
#[derive(Debug, Clone, Serialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub dots_score: f32,
    pub lift_type: String,
    pub anonymous_id: String,
}

/// Personal record for activity feed
#[derive(Debug, Clone, Serialize)]
pub struct PersonalRecord {
    pub improvement: f32,
    pub lift_type: String,
    pub new_dots: f32,
    pub timestamp: u64,
}

/// Reconnection backoff delays in seconds
#[allow(dead_code)]
const RECONNECT_BACKOFF: [u64; 6] = [1, 2, 5, 10, 30, 60];

/// Shared WebSocket state
#[derive(Clone)]
pub struct WebSocketState {
    pub connections: Connections,
    pub broadcaster: Broadcaster,
    pub stats: Arc<WebSocketStats>,
}

/// Real-time statistics
#[derive(Debug, Default)]
pub struct WebSocketStats {
    pub total_connections: AtomicUsize,
    pub active_connections: AtomicUsize,
    pub calculations_performed: AtomicUsize,
    pub recent_calculations: Arc<DashMap<String, RecentCalculation>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            connections: Arc::new(DashMap::new()),
            broadcaster: Arc::new(tx),
            stats: Arc::new(WebSocketStats::default()),
        }
    }

    /// Get reconnection delay based on attempt number
    #[allow(dead_code)]
    pub fn get_reconnect_delay(&self, attempt: usize) -> u64 {
        *RECONNECT_BACKOFF.get(attempt).unwrap_or(&60)
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Add a new connection
    pub fn add_connection(&self, id: String, info: ConnectionInfo) {
        self.connections.insert(id, info);
        self.stats
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Remove a connection
    pub fn remove_connection(&self, id: &str) {
        if self.connections.remove(id).is_some() {
            self.stats
                .active_connections
                .fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(
        &self,
        message: BroadcastMessage,
    ) -> Result<usize, broadcast::error::SendError<BroadcastMessage>> {
        self.broadcaster.send(message)
    }

    /// Add a recent calculation for activity tracking
    pub fn add_recent_calculation(&self, calc: RecentCalculation) {
        let id = Uuid::new_v4().to_string();
        self.stats.recent_calculations.insert(id, calc);
        self.stats
            .calculations_performed
            .fetch_add(1, Ordering::Relaxed);

        // Keep only the most recent 50 calculations
        if self.stats.recent_calculations.len() > 50 {
            // Remove oldest entries (simple cleanup)
            let keys_to_remove: Vec<_> = self
                .stats
                .recent_calculations
                .iter()
                .take(10)
                .map(|entry| entry.key().clone())
                .collect();

            for key in keys_to_remove {
                self.stats.recent_calculations.remove(&key);
            }
        }
    }

    /// Get recent calculations for activity feed
    pub fn get_recent_calculations(&self) -> Vec<RecentCalculation> {
        self.stats
            .recent_calculations
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    let ws_state = app_state
        .websocket_state
        .as_ref()
        .and_then(|any| any.downcast_ref::<WebSocketState>())
        .cloned()
        .unwrap_or_else(|| WebSocketState::new());
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, ws_state))
}

/// Send a WebSocket message in optimal format (Arrow binary or JSON text)
async fn send_websocket_message(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    message: &WebSocketMessage,
    supports_arrow: bool,
) -> Result<(), axum::Error> {
    if supports_arrow && should_use_arrow_format(message) {
        // Send as Arrow binary message
        match serialize_websocket_message_to_arrow(message) {
            Ok(arrow_data) => {
                if let Err(e) = sender.send(Message::Binary(arrow_data.into())).await {
                    tracing::error!("Failed to send Arrow WebSocket message: {}", e);
                    return Err(e);
                }
                tracing::debug!(
                    "📡 Sent Arrow binary message: {:?}",
                    std::mem::discriminant(message)
                );
            }
            Err(e) => {
                tracing::error!("Failed to serialize to Arrow, falling back to JSON: {}", e);
                // Fallback to JSON
                let json = serde_json::to_string(message).unwrap_or_else(|_| "{}".to_string());
                if let Err(e) = sender.send(Message::Text(json.into())).await {
                    return Err(e);
                }
            }
        }
    } else {
        // Send as JSON text message (default)
        let json = serde_json::to_string(message).unwrap_or_else(|_| "{}".to_string());
        if let Err(e) = sender.send(Message::Text(json.into())).await {
            return Err(e);
        }
        tracing::debug!(
            "📡 Sent JSON text message: {:?}",
            std::mem::discriminant(message)
        );
    }
    Ok(())
}

/// Handle individual WebSocket connections
async fn handle_socket(socket: WebSocket, app_state: AppState, ws_state: WebSocketState) {
    let connection_id = Uuid::new_v4().to_string();
    let mut rx = ws_state.broadcaster.subscribe();
    let supports_arrow = false; // Detect this during handshake

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Add connection to the manager
    let connection_info = ConnectionInfo {
        last_activity: current_timestamp(),
        user_agent: None, // Could be extracted from headers
    };

    ws_state.add_connection(connection_id.clone(), connection_info);
    ws_state
        .stats
        .total_connections
        .fetch_add(1, Ordering::Relaxed);

    println!("🔗 WebSocket client connected: {}", connection_id);

    // Send initial connection confirmation
    let welcome_msg = WebSocketMessage::StatsUpdate {
        active_users: ws_state.connection_count(),
        total_connections: ws_state.stats.total_connections.load(Ordering::Relaxed),
        server_load: 0.1, // Could implement actual server load monitoring
    };

    let _ = send_websocket_message(&mut sender, &welcome_msg, supports_arrow).await;

    // Create channel for sending responses from receive task to send task
    let (response_tx, mut response_rx) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();

    // Spawn task to handle incoming messages from this client
    let ws_state_clone = ws_state.clone();
    let app_state_clone = app_state.clone();
    let connection_id_clone = connection_id.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match process_message(msg, &app_state_clone, &ws_state_clone, &connection_id_clone)
                    .await
                {
                    Ok(Some(response_msg)) => {
                        // Send response message through channel to send task
                        if response_tx.send(response_msg).is_err() {
                            println!(
                                "❌ Failed to queue response for client {}",
                                connection_id_clone
                            );
                            break;
                        }
                    }
                    Ok(None) => {
                        // No response needed, continue
                    }
                    Err(_) => {
                        // Connection error, break the loop
                        break;
                    }
                }
            } else {
                break;
            }
        }
        println!("📤 Receive task ended for {}", connection_id_clone);
    });

    // Handle broadcast messages to send to this client
    let ws_state_send = ws_state.clone();
    let connection_id_send = connection_id.clone();

    let send_task = tokio::spawn(async move {
        let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut stats_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                broadcast_result = rx.recv() => {
                    match broadcast_result {
                        Ok(broadcast_msg) => {
                            if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    println!("❌ Failed to send broadcast to client {}", connection_id_send);
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            println!("📡 Broadcast channel closed for {}", connection_id_send);
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            println!("⚠️ Client {} lagged, skipped {} messages", connection_id_send, skipped);
                            continue;
                        }
                    }
                }
                response_msg = response_rx.recv() => {
                    if let Some(msg) = response_msg {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                println!("❌ Failed to send response to client {}", connection_id_send);
                                break;
                            }
                        }
                    } else {
                        // Response channel closed
                        println!("📬 Response channel closed for {}", connection_id_send);
                        break;
                    }
                }
                _ = heartbeat_interval.tick() => {
                    // Send periodic ping to keep connection alive
                    let ping_msg = WebSocketMessage::Ping;
                    if let Ok(json) = serde_json::to_string(&ping_msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            println!("💔 Heartbeat failed for client {}", connection_id_send);
                            break;
                        }
                    }
                }
                _ = stats_interval.tick() => {
                    // Send periodic server stats update
                    let stats_msg = WebSocketMessage::StatsUpdate {
                        active_users: ws_state_send.connection_count(),
                        total_connections: ws_state_send.stats.total_connections.load(std::sync::atomic::Ordering::Relaxed),
                        server_load: calculate_server_load(),
                    };

                    if let Ok(json) = serde_json::to_string(&stats_msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            println!("📊 Stats update failed for client {}", connection_id_send);
                            break;
                        }
                    }
                }
            }
        }
        println!("📡 Send task ended for {}", connection_id_send);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {},
        _ = send_task => {},
    }

    // Clean up connection
    ws_state.remove_connection(&connection_id);
    println!("🔌 WebSocket client disconnected: {}", connection_id);
}

/// Process incoming WebSocket messages from clients
async fn process_message(
    msg: Message,
    app_state: &AppState,
    ws_state: &WebSocketState,
    connection_id: &str,
) -> Result<Option<WebSocketMessage>, Box<dyn std::error::Error + Send + Sync>> {
    // Update connection activity timestamp
    if let Some(mut conn) = ws_state.connections.get_mut(connection_id) {
        conn.last_activity = current_timestamp();
    }

    match msg {
        Message::Text(text) => {
            // Parse incoming JSON message
            let ws_msg: WebSocketMessage = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    println!("❌ Failed to parse WebSocket message: {}", e);
                    return Ok(Some(WebSocketMessage::Error {
                        message: format!("Invalid message format: {}", e),
                    }));
                }
            };

            match ws_msg {
                WebSocketMessage::Connect {
                    session_id,
                    user_agent,
                    supports_arrow,
                } => {
                    // Update connection info with session details
                    if let Some(mut conn) = ws_state.connections.get_mut(connection_id) {
                        conn.user_agent = user_agent.clone();
                        conn.last_activity = current_timestamp();
                    }

                    // Note: We can't modify supports_arrow from here since it's not in scope
                    // of the outer function. We'll handle this differently.

                    let arrow_support_msg = if supports_arrow.unwrap_or(false) {
                        "🚀 Arrow binary support enabled!"
                    } else {
                        "📝 Using JSON text messages"
                    };

                    println!(
                        "📝 Client registered session: {} ({}) - {}",
                        session_id,
                        user_agent.unwrap_or_else(|| "Unknown".to_string()),
                        arrow_support_msg
                    );

                    // Send back connection confirmation with server stats
                    return Ok(Some(WebSocketMessage::StatsUpdate {
                        active_users: ws_state.connection_count(),
                        total_connections: ws_state
                            .stats
                            .total_connections
                            .load(std::sync::atomic::Ordering::Relaxed),
                        server_load: calculate_server_load(),
                    }));
                }

                WebSocketMessage::UserUpdate {
                    bodyweight,
                    squat,
                    bench,
                    deadlift,
                    lift_type,
                    sex,
                } => {
                    // Validate input data
                    if let Some(bw) = bodyweight {
                        if bw <= 0.0 || bw > 500.0 {
                            // Reasonable bodyweight limits
                            return Ok(Some(WebSocketMessage::Error {
                                message: "Invalid bodyweight: must be between 0 and 500kg"
                                    .to_string(),
                            }));
                        }
                    }

                    // Calculate DOTS score and broadcast activity
                    if let (Some(bw), Some(lift)) = (
                        bodyweight,
                        get_lift_value(squat, bench, deadlift, &lift_type),
                    ) {
                        if lift <= 0.0 || lift > 1000.0 {
                            // Reasonable lift limits
                            return Ok(Some(WebSocketMessage::Error {
                                message: "Invalid lift weight: must be between 0 and 1000kg"
                                    .to_string(),
                            }));
                        }

                        let user_sex = sex.as_deref().unwrap_or("M"); // Default to male if not specified
                        let dots_score = calculate_dots_score(lift, bw, user_sex);
                        let strength_level = get_strength_level(dots_score);

                        // Calculate percentile using app data if available
                        let percentile =
                            calculate_percentile_from_data(app_state, lift, &lift_type);

                        // Add to recent calculations for activity feed
                        let recent_calc = RecentCalculation {
                            dots_score,
                            lift_type: lift_type.clone(),
                            strength_level: strength_level.clone(),
                            timestamp: current_timestamp(),
                        };

                        ws_state.add_recent_calculation(recent_calc);

                        // Broadcast activity update to all clients
                        let activity = BroadcastMessage::UserActivity {
                            user_count: ws_state.connection_count(),
                            recent_calculations: ws_state.get_recent_calculations(),
                        };

                        let _ = ws_state.broadcast(activity);

                        // Send detailed calculation result back to the user
                        let dots_result = WebSocketMessage::DotsCalculation {
                            lift_kg: lift,
                            bodyweight_kg: bw,
                            dots_score,
                            strength_level: strength_level.clone(),
                            percentile,
                        };

                        println!(
                            "📊 User update [{}]: {}kg @ {}kg = {:.1} DOTS ({}) - {}%ile",
                            connection_id,
                            lift,
                            bw,
                            dots_score,
                            strength_level,
                            percentile.map(|p| p as i32).unwrap_or(-1)
                        );

                        return Ok(Some(dots_result));
                    } else {
                        return Ok(Some(WebSocketMessage::Error {
                            message:
                                "Missing required data: bodyweight and at least one lift value"
                                    .to_string(),
                        }));
                    }
                }

                WebSocketMessage::Ping => {
                    // Respond with pong to keep connection alive
                    println!("🏓 Ping received from {}", connection_id);
                    return Ok(Some(WebSocketMessage::Pong));
                }

                WebSocketMessage::Pong => {
                    // Client responded to our ping
                    println!("🏓 Pong received from {}", connection_id);
                    // No response needed
                }

                WebSocketMessage::StatsUpdate { .. }
                | WebSocketMessage::ActivityFeed { .. }
                | WebSocketMessage::DotsCalculation { .. } => {
                    // These are server-to-client messages, shouldn't be received from client
                    return Ok(Some(WebSocketMessage::Error {
                        message: "Invalid message type from client".to_string(),
                    }));
                }

                WebSocketMessage::Error { message } => {
                    println!(
                        "❌ Error message from client {}: {}",
                        connection_id, message
                    );
                    // Log the error but don't respond
                }
            }
        }

        Message::Binary(data) => {
            println!(
                "📦 Binary message received from {}: {} bytes",
                connection_id,
                data.len()
            );
            // Handle binary messages if needed (e.g., file uploads, compressed data)
            return Ok(Some(WebSocketMessage::Error {
                message: "Binary messages not supported".to_string(),
            }));
        }

        Message::Ping(_data) => {
            println!("🏓 WebSocket Ping received from {}", connection_id);
            // Axum handles ping/pong automatically, but we can log it
        }

        Message::Pong(_) => {
            println!("🏓 WebSocket Pong received from {}", connection_id);
            // Connection is alive
        }

        Message::Close(close_frame) => {
            if let Some(frame) = close_frame {
                println!(
                    "👋 Client {} closing connection: {} - {}",
                    connection_id, frame.code, frame.reason
                );
            } else {
                println!("👋 Client {} closing connection (no reason)", connection_id);
            }
            return Err("Connection closed".into());
        }
    }

    Ok(None) // No response message needed
}

/// Get lift value based on lift type
fn get_lift_value(
    squat: Option<f32>,
    bench: Option<f32>,
    deadlift: Option<f32>,
    lift_type: &str,
) -> Option<f32> {
    match lift_type {
        "squat" => squat,
        "bench" => bench,
        "deadlift" => deadlift,
        "total" => {
            let s = squat.unwrap_or(0.0);
            let b = bench.unwrap_or(0.0);
            let d = deadlift.unwrap_or(0.0);
            if s > 0.0 || b > 0.0 || d > 0.0 {
                Some(s + b + d)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get strength level from DOTS score
fn get_strength_level(dots_score: f32) -> String {
    if dots_score < 200.0 {
        "Beginner".to_string()
    } else if dots_score < 300.0 {
        "Novice".to_string()
    } else if dots_score < 400.0 {
        "Intermediate".to_string()
    } else if dots_score < 500.0 {
        "Advanced".to_string()
    } else if dots_score < 600.0 {
        "Elite".to_string()
    } else {
        "World Class".to_string()
    }
}

/// Calculate percentile from app data
fn calculate_percentile_from_data(
    app_state: &AppState,
    lift_value: f32,
    lift_type: &str,
) -> Option<f32> {
    use iron_core::models::LiftType;

    let lift_type_enum = LiftType::from_str(lift_type);
    let column_name = lift_type_enum.raw_column();

    // Extract lift values from the dataset
    if let Ok(lift_column) = app_state.data.column(column_name) {
        if let Ok(f32_series) = lift_column.f32() {
            let lift_values: Vec<f32> = f32_series
                .into_no_null_iter()
                .filter(|&x| x > 0.0 && x.is_finite())
                .collect();

            if lift_values.is_empty() {
                return None;
            }

            let below_count = lift_values
                .iter()
                .filter(|&&value| value < lift_value)
                .count();

            let percentile = (below_count as f32 / lift_values.len() as f32) * 100.0;
            return Some(percentile.round());
        }
    }

    None
}

/// Calculate current server load (simplified implementation)
fn calculate_server_load() -> f32 {
    // In a real implementation, this would check:
    // - CPU usage
    // - Memory usage
    // - Active connections vs capacity
    // - Processing queue length

    // For now, return a mock value
    0.15 // 15% load
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
