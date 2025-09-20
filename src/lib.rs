// lib.rs - Library interface for Iron Insights
pub mod arrow_utils;
pub mod cache;
pub mod config;
pub mod data;
pub mod duckdb_analytics;
pub mod filters;
pub mod handlers;
pub mod http3_server;
pub mod models;
pub mod percentiles;
pub mod scoring;
pub mod share_card;
pub mod ui;
pub mod viz;
pub mod websocket;
pub mod websocket_arrow;

// Re-export commonly used types for testing
pub use config::AppConfig;
pub use data::DataProcessor;
pub use http3_server::Http3Server;
pub use models::AppState;
