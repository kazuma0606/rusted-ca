//shared/metrics/collector.rs
// メトリクス収集器
// 2025/7/8

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct MetricsCollector {
    pub log_count: Arc<AtomicU64>,
    pub error_count: Arc<AtomicU64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            log_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_log_count(&self) {
        self.log_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_log_count(&self) -> u64 {
        self.log_count.load(Ordering::Relaxed)
    }

    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
