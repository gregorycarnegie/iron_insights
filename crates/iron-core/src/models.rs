// models.rs - Updated with new filter parameters and DuckDB integration
use crate::duckdb_analytics::DuckDBAnalytics;
use moka::future::Cache;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{any::Any, sync::Arc, time::Instant};

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub data: Arc<DataFrame>,
    pub cache: Cache<String, CachedResult>,
    pub websocket_state: Option<Arc<dyn Any + Send + Sync>>,
    pub duckdb: Option<Arc<DuckDBAnalytics>>,
    pub manifest: AssetManifest,
}

impl AppState {
    pub fn new(
        data: Arc<DataFrame>,
        cache_config: (u64, std::time::Duration),
        manifest: AssetManifest,
    ) -> Self {
        let cache = Cache::builder()
            .max_capacity(cache_config.0)
            .time_to_live(cache_config.1)
            .build();

        Self {
            data,
            cache,
            websocket_state: None,
            duckdb: None,
            manifest,
        }
    }

    pub fn with_duckdb(mut self, duckdb: DuckDBAnalytics) -> Self {
        self.duckdb = Some(Arc::new(duckdb));
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct AssetManifest {
    pub assets: std::collections::HashMap<String, String>,
}

impl AssetManifest {
    pub fn new() -> Self {
        Self {
            assets: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> String {
        if let Some(hashed_name) = self.assets.get(key) {
            format!("/static/js/dist/{}", hashed_name)
        } else {
            format!("/static/js/{}", key)
        }
    }
}

#[derive(Clone, Debug)]
pub struct CachedResult {
    pub data: Vec<u8>, // Pre-serialized JSON or Arrow
    pub computed_at: Instant,
}

// Enhanced filter parameters with additional fields
#[derive(Serialize, Deserialize, Debug, Default)]
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
    pub federation: Option<String>,   // New: "all", "ipf", "usapl", "uspa", "wrpf"
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

impl From<crate::viz::VizData> for VisualizationResponse {
    fn from(data: crate::viz::VizData) -> Self {
        VisualizationResponse {
            histogram_data: data.hist,
            scatter_data: data.scatter,
            dots_histogram_data: data.dots_hist,
            dots_scatter_data: data.dots_scatter,
            user_percentile: data.user_percentile,
            user_dots_percentile: data.user_dots_percentile,
            processing_time_ms: data.processing_time_ms,
            total_records: data.total_records,
        }
    }
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
    pub fn parse(s: &str) -> Self {
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
