// config.rs
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cache_max_capacity: u64,
    pub cache_ttl_seconds: u64,
    pub sample_size: usize,
    pub histogram_bins: usize,
    pub cache_refresh_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache_max_capacity: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            sample_size: 100000,
            histogram_bins: 50,
            cache_refresh_seconds: 300, // 5 minutes
        }
    }
}

impl AppConfig {
    pub fn cache_config(&self) -> (u64, Duration) {
        (self.cache_max_capacity, Duration::from_secs(self.cache_ttl_seconds))
    }
    
    pub fn cache_refresh_duration(&self) -> Duration {
        Duration::from_secs(self.cache_refresh_seconds)
    }
}