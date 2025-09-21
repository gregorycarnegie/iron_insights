use iron_insights::duckdb_analytics::*;
use std::env;

#[test]
fn test_duckdb_initialization() {
    // This test requires a real parquet file
    if let Ok(test_file) = env::var("TEST_PARQUET_PATH") {
        let analytics = DuckDBAnalytics::from_parquet(&test_file);
        assert!(analytics.is_ok());
    }
}