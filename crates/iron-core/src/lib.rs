// iron-core: Core data processing library
pub mod arrow_utils;
pub mod cache;
pub mod config;
pub mod data;
pub mod duckdb_analytics;
pub mod filters;
pub mod models;
pub mod percentiles;
pub mod scoring;
pub mod viz;

// Re-export commonly used types
pub use config::AppConfig;
pub use data::DataProcessor;
pub use models::AppState;
