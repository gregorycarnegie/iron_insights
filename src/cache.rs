// src/cache.rs - Cache management helpers with Arrow IPC binary protocol
use crate::arrow_utils::{
    deserialize_visualization_response_from_arrow, serialize_visualization_response_to_arrow,
};
use crate::models::{AppState, CachedResult, VisualizationResponse};
use serde::Serialize;
use std::time::Duration as STD_Duration;
use std::time::Instant;

pub const CACHE_TTL_SECS: u64 = 300; // 5 minutes default

/// Try to get a cached result using Arrow IPC binary protocol for VisualizationResponse
pub async fn cache_get_arrow(state: &AppState, key: &str) -> Option<VisualizationResponse> {
    let hit = state.cache.get(key).await?;

    // Check TTL
    if hit.computed_at.elapsed() > STD_Duration::from_secs(CACHE_TTL_SECS) {
        // Expired, remove from cache
        state.cache.invalidate(key).await;
        return None;
    }

    // Deserialize using Arrow IPC
    deserialize_visualization_response_from_arrow(&hit.data).ok()
}

/// Store a VisualizationResponse in cache using Arrow IPC binary protocol
pub async fn cache_put_arrow(state: &AppState, key: &str, value: &VisualizationResponse) {
    if let Ok(bytes) = serialize_visualization_response_to_arrow(value) {
        let cached = CachedResult {
            data: bytes,
            computed_at: Instant::now(),
        };
        state.cache.insert(key.to_string(), cached).await;
    }
}

/// Generate a deterministic cache key from parameters
pub fn make_cache_key<T: std::fmt::Debug>(params: &T, suffix: &str) -> String {
    format!("{:?}:{}", params, suffix)
}

/// Clear expired entries from cache (housekeeping)
pub async fn cleanup_expired(state: &AppState) {
    let _now = Instant::now();
    let _ttl = STD_Duration::from_secs(CACHE_TTL_SECS);

    // Moka cache handles TTL automatically, but we can force cleanup
    state.cache.run_pending_tasks().await;
}

/// Get cache statistics
pub fn cache_stats(state: &AppState) -> CacheStats {
    CacheStats {
        entry_count: state.cache.entry_count(),
        weighted_size: state.cache.weighted_size(),
        // Moka doesn't expose hit/miss directly, would need custom tracking
        hit_rate: 0.0,
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
    pub hit_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use std::sync::Arc;

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
}
