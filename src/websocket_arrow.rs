// websocket_arrow.rs - Arrow serialization for WebSocket messages
use arrow::array::{Float32Array, StringArray, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use std::io::Cursor;
use std::sync::Arc;

use crate::websocket::WebSocketMessage;

/// Serialize WebSocket message to Arrow IPC format
/// This creates a unified schema that can handle all WebSocket message types
pub fn serialize_websocket_message_to_arrow(
    message: &WebSocketMessage,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Create a comprehensive schema for all WebSocket message types
    let schema = Schema::new(vec![
        Field::new("message_type", DataType::Utf8, false),
        // Connect fields
        Field::new("session_id", DataType::Utf8, true),
        Field::new("user_agent", DataType::Utf8, true),
        // UserUpdate fields
        Field::new("bodyweight", DataType::Float32, true),
        Field::new("squat", DataType::Float32, true),
        Field::new("bench", DataType::Float32, true),
        Field::new("deadlift", DataType::Float32, true),
        Field::new("lift_type", DataType::Utf8, true),
        Field::new("sex", DataType::Utf8, true),
        // StatsUpdate fields
        Field::new("active_users", DataType::UInt32, true),
        Field::new("total_connections", DataType::UInt32, true),
        Field::new("server_load", DataType::Float32, true),
        // DotsCalculation fields
        Field::new("lift_kg", DataType::Float32, true),
        Field::new("bodyweight_kg", DataType::Float32, true),
        Field::new("dots_score", DataType::Float32, true),
        Field::new("strength_level", DataType::Utf8, true),
        Field::new("percentile", DataType::Float32, true),
        // Common fields
        Field::new("error_message", DataType::Utf8, true),
        Field::new("timestamp", DataType::UInt64, true),
    ]);

    // Initialize all arrays with null values
    let message_type_val;
    let mut session_id_val: Option<String> = None;
    let mut user_agent_val: Option<String> = None;
    let mut bodyweight_val: Option<f32> = None;
    let mut squat_val: Option<f32> = None;
    let mut bench_val: Option<f32> = None;
    let mut deadlift_val: Option<f32> = None;
    let mut lift_type_val: Option<String> = None;
    let mut sex_val: Option<String> = None;
    let mut active_users_val: Option<u32> = None;
    let mut total_connections_val: Option<u32> = None;
    let mut server_load_val: Option<f32> = None;
    let mut lift_kg_val: Option<f32> = None;
    let mut bodyweight_kg_val: Option<f32> = None;
    let mut dots_score_val: Option<f32> = None;
    let mut strength_level_val: Option<String> = None;
    let mut percentile_val: Option<f32> = None;
    let mut error_message_val: Option<String> = None;
    let mut timestamp_val: Option<u64> = None;

    // Fill values based on message type
    match message {
        WebSocketMessage::Connect {
            session_id,
            user_agent,
            supports_arrow: _,
        } => {
            message_type_val = "connect";
            session_id_val = Some(session_id.clone());
            user_agent_val = user_agent.clone();
        }
        WebSocketMessage::UserUpdate {
            bodyweight,
            squat,
            bench,
            deadlift,
            lift_type,
            sex,
        } => {
            message_type_val = "user_update";
            bodyweight_val = *bodyweight;
            squat_val = *squat;
            bench_val = *bench;
            deadlift_val = *deadlift;
            lift_type_val = Some(lift_type.clone());
            sex_val = sex.clone();
        }
        WebSocketMessage::StatsUpdate {
            active_users,
            total_connections,
            server_load,
        } => {
            message_type_val = "stats_update";
            active_users_val = Some(*active_users as u32);
            total_connections_val = Some(*total_connections as u32);
            server_load_val = Some(*server_load);
        }
        WebSocketMessage::DotsCalculation {
            lift_kg,
            bodyweight_kg,
            dots_score,
            strength_level,
            percentile,
        } => {
            message_type_val = "dots_calculation";
            lift_kg_val = Some(*lift_kg);
            bodyweight_kg_val = Some(*bodyweight_kg);
            dots_score_val = Some(*dots_score);
            strength_level_val = Some(strength_level.clone());
            percentile_val = *percentile;
        }
        WebSocketMessage::Error { message } => {
            message_type_val = "error";
            error_message_val = Some(message.clone());
        }
        WebSocketMessage::Ping => {
            message_type_val = "ping";
        }
        WebSocketMessage::Pong => {
            message_type_val = "pong";
        }
        WebSocketMessage::ActivityFeed {
            activity_type: _,
            data: _,
            timestamp,
        } => {
            message_type_val = "activity_feed";
            timestamp_val = Some(*timestamp);
        }
    }

    // Create arrays (all single-row)
    let message_type_array = StringArray::from(vec![message_type_val]);
    let session_id_array = StringArray::from(vec![session_id_val]);
    let user_agent_array = StringArray::from(vec![user_agent_val]);
    let bodyweight_array = Float32Array::from(vec![bodyweight_val]);
    let squat_array = Float32Array::from(vec![squat_val]);
    let bench_array = Float32Array::from(vec![bench_val]);
    let deadlift_array = Float32Array::from(vec![deadlift_val]);
    let lift_type_array = StringArray::from(vec![lift_type_val]);
    let sex_array = StringArray::from(vec![sex_val]);
    let active_users_array = UInt32Array::from(vec![active_users_val]);
    let total_connections_array = UInt32Array::from(vec![total_connections_val]);
    let server_load_array = Float32Array::from(vec![server_load_val]);
    let lift_kg_array = Float32Array::from(vec![lift_kg_val]);
    let bodyweight_kg_array = Float32Array::from(vec![bodyweight_kg_val]);
    let dots_score_array = Float32Array::from(vec![dots_score_val]);
    let strength_level_array = StringArray::from(vec![strength_level_val]);
    let percentile_array = Float32Array::from(vec![percentile_val]);
    let error_message_array = StringArray::from(vec![error_message_val]);
    let timestamp_array = UInt64Array::from(vec![timestamp_val]);

    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(message_type_array),
            Arc::new(session_id_array),
            Arc::new(user_agent_array),
            Arc::new(bodyweight_array),
            Arc::new(squat_array),
            Arc::new(bench_array),
            Arc::new(deadlift_array),
            Arc::new(lift_type_array),
            Arc::new(sex_array),
            Arc::new(active_users_array),
            Arc::new(total_connections_array),
            Arc::new(server_load_array),
            Arc::new(lift_kg_array),
            Arc::new(bodyweight_kg_array),
            Arc::new(dots_score_array),
            Arc::new(strength_level_array),
            Arc::new(percentile_array),
            Arc::new(error_message_array),
            Arc::new(timestamp_array),
        ],
    )?;

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut stream_writer = StreamWriter::try_new(&mut buffer, &record_batch.schema())?;
        stream_writer.write(&record_batch)?;
        stream_writer.finish()?;
    }

    Ok(buffer.into_inner())
}

/// Check if a WebSocket message should use Arrow format
/// Some message types benefit more from Arrow than others
pub fn should_use_arrow_format(message: &WebSocketMessage) -> bool {
    match message {
        // High-frequency, data-heavy messages benefit most
        WebSocketMessage::DotsCalculation { .. } => true,
        WebSocketMessage::StatsUpdate { .. } => true,
        WebSocketMessage::UserUpdate { .. } => true,
        // Simple messages can stick with JSON
        WebSocketMessage::Ping | WebSocketMessage::Pong => false,
        WebSocketMessage::Error { .. } => false,
        // Connect can benefit from Arrow for consistency
        WebSocketMessage::Connect { .. } => true,
        WebSocketMessage::ActivityFeed { .. } => true,
    }
}
