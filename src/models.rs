// models.rs
use moka::future::Cache;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub data: Arc<DataFrame>,
    pub cache: Cache<String, CachedResult>,
    pub websocket_state: Option<crate::websocket::WebSocketState>,
}

impl AppState {
    pub fn new(data: Arc<DataFrame>, cache_config: (u64, std::time::Duration)) -> Self {
        let cache = Cache::builder()
            .max_capacity(cache_config.0)
            .time_to_live(cache_config.1)
            .build();
        
        Self { data, cache, websocket_state: None }
    }
}

#[derive(Clone, Debug)]
pub struct CachedResult {
    pub data: Vec<u8>, // Pre-serialized JSON
    pub computed_at: Instant,
}

// API request/response types
#[derive(Deserialize, Debug)]
pub struct FilterParams {
    pub sex: Option<String>,
    pub equipment: Option<Vec<String>>,
    pub weight_class: Option<String>,
    pub squat: Option<f32>,
    pub bench: Option<f32>,
    pub deadlift: Option<f32>,
    pub bodyweight: Option<f32>,
    pub units: Option<String>,
    pub lift_type: Option<String>, // "squat", "bench", "deadlift", "total"
}

#[derive(Serialize, Deserialize)]
pub struct VisualizationResponse {
    pub histogram_data: HistogramData,
    pub scatter_data: ScatterData,
    pub dots_histogram_data: HistogramData,
    pub dots_scatter_data: ScatterData,
    pub user_percentile: Option<f32>,
    pub user_dots_percentile: Option<f32>,
    pub processing_time_ms: u64,
    pub total_records: usize,
}

#[derive(Serialize, Deserialize)]
pub struct HistogramData {
    pub values: Vec<f32>,
    pub counts: Vec<u32>,
    pub bins: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ScatterData {
    pub x: Vec<f32>, // bodyweight
    pub y: Vec<f32>, // lift values
    pub sex: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LiftType {
    Squat,
    Bench,
    Deadlift,
    Total,
}

impl LiftType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "squat" => Self::Squat,
            "bench" => Self::Bench,
            "deadlift" => Self::Deadlift,
            "total" => Self::Total,
            _ => Self::Squat,
        }
    }
    
    pub fn raw_column(&self) -> &'static str {
        match self {
            Self::Squat => "Best3SquatKg",
            Self::Bench => "Best3BenchKg",
            Self::Deadlift => "Best3DeadliftKg",
            Self::Total => "TotalKg",
        }
    }
    
    pub fn dots_column(&self) -> &'static str {
        match self {
            Self::Squat => "SquatDOTS",
            Self::Bench => "BenchDOTS",
            Self::Deadlift => "DeadliftDOTS",
            Self::Total => "TotalDOTS",
        }
    }
}