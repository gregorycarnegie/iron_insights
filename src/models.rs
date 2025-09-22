// models.rs - Updated with new filter parameters and DuckDB integration
use moka::future::Cache;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use crate::duckdb_analytics::DuckDBAnalytics;

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub data: Arc<DataFrame>,
    pub cache: Cache<String, CachedResult>,
    pub websocket_state: Option<crate::websocket::WebSocketState>,
    pub duckdb: Option<Arc<DuckDBAnalytics>>,
}

impl AppState {
    pub fn new(data: Arc<DataFrame>, cache_config: (u64, std::time::Duration)) -> Self {
        let cache = Cache::builder()
            .max_capacity(cache_config.0)
            .time_to_live(cache_config.1)
            .build();

        Self {
            data,
            cache,
            websocket_state: None,
            duckdb: None,
        }
    }

    pub fn with_duckdb(mut self, duckdb: DuckDBAnalytics) -> Self {
        self.duckdb = Some(Arc::new(duckdb));
        self
    }
}

#[derive(Clone, Debug)]
pub struct CachedResult {
    pub data: Vec<u8>, // Pre-serialized JSON or Arrow
    pub computed_at: Instant,
}

// Enhanced filter parameters with additional fields
#[derive(Serialize, Deserialize, Debug)]
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
    pub min_bodyweight: Option<f32>, // New: minimum bodyweight filter
    pub max_bodyweight: Option<f32>, // New: maximum bodyweight filter
    pub years_filter: Option<String>, // New: "all", "last_5_years", "past_10_years", "ytd", "current_year", "previous_year", "last_12_months"
    pub federation: Option<String>, // New: "all", "ipf", "usapl", "uspa", "wrpf"
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            sex: None,
            equipment: None,
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
            federation: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RankingsParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sex: Option<String>,
    pub equipment: Option<String>,
    pub weight_class: Option<String>,
    pub federation: Option<String>,
    pub year: Option<u32>,
    pub sort_by: Option<String>, // "dots", "total", "squat", "bench", "deadlift"
}

impl Default for RankingsParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(100),
            sex: None,
            equipment: None,
            weight_class: None,
            federation: None,
            year: None,
            sort_by: Some("dots".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RankingEntry {
    pub rank: u32,
    pub name: String,
    pub federation: String,
    pub date: String,
    pub sex: String,
    pub equipment: String,
    pub weight_class: String,
    pub bodyweight: f32,
    pub squat: f32,
    pub bench: f32,
    pub deadlift: f32,
    pub total: f32,
    pub dots: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RankingsResponse {
    pub entries: Vec<RankingEntry>,
    pub total_count: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct StatsData {
    pub total_records: u32,
    pub cache_entries: u32,
    pub cache_size: u64,
    pub scoring_system: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistogramData {
    pub values: Vec<f32>,
    pub counts: Vec<u32>,
    pub bins: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScatterData {
    pub x: Vec<f32>, // bodyweight
    pub y: Vec<f32>, // lift values or DOTS scores
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
        match s.to_lowercase().as_str() {
            "squat" => Self::Squat,
            "bench" => Self::Bench,
            "deadlift" => Self::Deadlift,
            "total" => Self::Total,
            _ => Self::Squat, // Default to squat
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
