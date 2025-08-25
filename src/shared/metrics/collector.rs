//shared/metrics/collector.rs
// メトリクス収集器
// 2025/7/8

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crate::domain::entity::log_entry::LogEntry;
use crate::shared::error::application_error::ApplicationResult;

pub trait MetricsCollectorInterface: Send + Sync {
    fn record_log_event(&self, log_entry: &LogEntry) -> ApplicationResult<()>;
    fn increment_log_count(&self);
    fn increment_error_count(&self);
    fn get_log_count(&self) -> u64;
    fn get_error_count(&self) -> u64;
}

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

impl MetricsCollectorInterface for MetricsCollector {
    fn record_log_event(&self, log_entry: &LogEntry) -> ApplicationResult<()> {
        self.increment_log_count();
        
        if log_entry.level().is_error() {
            self.increment_error_count();
        }
        
        Ok(())
    }

    fn increment_log_count(&self) {
        self.log_count.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    fn get_log_count(&self) -> u64 {
        self.log_count.load(Ordering::Relaxed)
    }

    fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
