// src/cache.rs - Cache management helpers with Arrow IPC binary protocol
use serde::{Serialize, de::DeserializeOwned};
use std::time::Instant;
use std::time::Duration as STD_Duration;
use crate::models::{AppState, CachedResult, VisualizationResponse};
use crate::arrow_utils::{serialize_visualization_response_to_arrow, deserialize_visualization_response_from_arrow};

pub const CACHE_TTL_SECS: u64 = 300; // 5 minutes default

/// Try to get a cached result using Arrow IPC binary protocol for VisualizationResponse
pub async fn cache_get_arrow(
    state: &AppState,
    key: &str,
) -> Option<VisualizationResponse> {
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

/// Try to get a cached result (legacy JSON fallback)
pub async fn cache_get<T: DeserializeOwned>(
    state: &AppState,
    key: &str,
) -> Option<T> {
    let hit = state.cache.get(key).await?;
    
    // Check TTL
    if hit.computed_at.elapsed() > STD_Duration::from_secs(CACHE_TTL_SECS) {
        // Expired, remove from cache
        state.cache.invalidate(key).await;
        return None;
    }
    
    // Deserialize the cached data
    serde_json::from_slice(&hit.data).ok()
}

/// Store a VisualizationResponse in cache using Arrow IPC binary protocol
pub async fn cache_put_arrow(
    state: &AppState,
    key: &str,
    value: &VisualizationResponse,
) {
    if let Ok(bytes) = serialize_visualization_response_to_arrow(value) {
        let cached = CachedResult {
            data: bytes,
            computed_at: Instant::now(),
        };
        state.cache.insert(key.to_string(), cached).await;
    }
}

/// Store a result in cache (legacy JSON fallback)
pub async fn cache_put<T: Serialize>(
    state: &AppState,
    key: &str,
    value: &T,
) {
    if let Ok(bytes) = serde_json::to_vec(value) {
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

/// Invalidate cache entries matching a pattern
pub async fn invalidate_pattern(state: &AppState, pattern: &str) {
    // Since Moka doesn't support pattern matching directly,
    // we'd need to track keys separately if pattern invalidation is needed
    // For now, this is a placeholder that could be enhanced
    
    // Option 1: Clear entire cache (nuclear option)
    if pattern == "*" {
        state.cache.invalidate_all();
    }
    
    // Option 2: Iterate through known keys (would need key tracking)
    // This would require maintaining a separate list of cache keys
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
    use serde::{Deserialize, Serialize};
    use moka::future::Cache;
    use std::sync::Arc;
    use polars::prelude::*;
    
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
        count: u32,
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        // Create a test cache
        let cache: moka::future::Cache<String, CachedResult> = Cache::builder()
            .max_capacity(100)
            .time_to_live(STD_Duration::from_secs(60))
            .build();
        
        // Create test app state
        let data = DataFrame::empty();
        let state = AppState::new(Arc::new(data), (100, STD_Duration::from_secs(60)));
        
        let test_data = TestData {
            value: "test".to_string(),
            count: 42,
        };
        
        // Test cache put
        cache_put(&state, "test_key", &test_data).await;
        
        // Test cache get
        let retrieved: Option<TestData> = cache_get(&state, "test_key").await;
        assert_eq!(retrieved, Some(test_data));
        
        // Test cache miss
        let miss: Option<TestData> = cache_get(&state, "nonexistent").await;
        assert_eq!(miss, None);
    }
    
    #[test]
    fn test_cache_key_generation() {
        #[derive(Debug)]
        struct TestParams {
            id: u32,
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