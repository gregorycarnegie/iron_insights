// config.rs
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cache_max_capacity: u64,
    pub cache_ttl_seconds: u64,
    pub sample_size: usize,
    pub histogram_bins: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache_max_capacity: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            sample_size: 500_000,
            histogram_bins: 50,
        }
    }
}

impl AppConfig {
    pub fn cache_config(&self) -> (u64, Duration) {
        (
            self.cache_max_capacity,
            Duration::from_secs(self.cache_ttl_seconds),
        )
    }
}
