use iron_insights::cache::*;
use iron_insights::models::AppState;
use polars::prelude::*;
use std::sync::Arc;
use std::time::Duration as STD_Duration;

#[tokio::test]
async fn test_cache_operations() {
    // Create test app state
    let data = DataFrame::empty();
    let state = AppState::new(Arc::new(data), (100, STD_Duration::from_secs(60)));

    // Test that cache operations complete without error
    let stats = cache_stats(&state);
    assert_eq!(stats.entry_count, 0);

    cleanup_expired(&state).await;
}

#[test]
fn test_cache_key_generation() {
    #[derive(Debug)]
    struct TestParams {
        #[allow(dead_code)]
        id: u32,
        #[allow(dead_code)]
        name: String,
    }

    let params = TestParams {
        id: 123,
        name: "test".to_string(),
    };

    let key1 = make_cache_key(&params, "json");
    let key2 = make_cache_key(&params, "arrow");

    assert!(key1.contains("json"));
    assert!(key2.contains("arrow"));
    assert_ne!(key1, key2);
}
