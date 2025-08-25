use std::sync::Arc;

use crate::domain::{
    entity::log_entry::LogEntry,
    repository::log_repository::{
        LogRepositoryInterface, LogSearchCriteria, LogSearchResult, PerformanceMetrics,
    },
    value_object::{log_id::LogId, request_id::RequestId},
};
use crate::shared::error::application_error::ApplicationError;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug, Clone)]
pub struct AnalysisCriteria {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub include_layers:
        Option<Vec<crate::domain::value_object::architecture_layer::ArchitectureLayer>>,
    pub min_response_time_ms: Option<u64>,
    pub max_response_time_ms: Option<u64>,
}

impl AnalysisCriteria {
    pub fn new(
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            start_time,
            end_time,
            include_layers: None,
            min_response_time_ms: None,
            max_response_time_ms: None,
        }
    }

    pub fn to_search_criteria(&self) -> LogSearchCriteria {
        LogSearchCriteria::new().with_time_range(self.start_time, self.end_time)
    }

    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.start_time >= self.end_time {
            return Err(ApplicationError::ValidationError(
                "Start time must be before end time".to_string(),
            ));
        }

        let duration = self.end_time - self.start_time;
        if duration > chrono::Duration::days(30) {
            return Err(ApplicationError::ValidationError(
                "Analysis period cannot exceed 30 days".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub metrics: PerformanceMetrics,
    pub analysis_period: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    pub top_slow_requests: Vec<LogEntry>,
    pub top_errors: Vec<LogEntry>,
    pub recommendations: Vec<String>,
}

impl PerformanceReport {
    pub fn analyze(logs: Vec<LogEntry>) -> Result<Self, ApplicationError> {
        if logs.is_empty() {
            return Err(ApplicationError::ValidationError(
                "No logs available for analysis".to_string(),
            ));
        }

        let start_time = logs.iter().map(|l| l.timestamp()).min().unwrap();
        let end_time = logs.iter().map(|l| l.timestamp()).max().unwrap();

        // Basic performance metrics calculation
        let mut metrics = PerformanceMetrics::new();
        let mut response_times = Vec::new();
        let mut error_logs = Vec::new();
        let mut slow_requests = Vec::new();

        for log in &logs {
            if log.architecture_layer()
                == crate::domain::value_object::architecture_layer::ArchitectureLayer::Presentation
            {
                metrics.total_requests += 1;

                let response_time = log.http_context().response_time_ms();
                response_times.push(response_time);

                let status_code = log.http_context().status_code();
                *metrics
                    .requests_by_status_code
                    .entry(status_code)
                    .or_insert(0) += 1;

                if status_code >= 400 {
                    error_logs.push(log.clone());
                }

                if response_time > 1000 {
                    slow_requests.push(log.clone());
                }
            }

            let layer = log.architecture_layer();
            *metrics.requests_by_layer.entry(layer).or_insert(0) += 1;
        }

        if !response_times.is_empty() {
            response_times.sort();
            metrics.min_response_time_ms = response_times[0];
            metrics.max_response_time_ms = response_times[response_times.len() - 1];
            metrics.avg_response_time_ms =
                response_times.iter().sum::<u64>() as f64 / response_times.len() as f64;

            let p95_index = (response_times.len() as f64 * 0.95) as usize;
            metrics.p95_response_time_ms = response_times[p95_index.min(response_times.len() - 1)];
        }

        if metrics.total_requests > 0 {
            metrics.error_rate = error_logs.len() as f64 / metrics.total_requests as f64;
        }

        // Sort and limit results
        error_logs.sort_by(|a, b| b.timestamp().cmp(&a.timestamp()));
        slow_requests.sort_by(|a, b| {
            b.http_context()
                .response_time_ms()
                .cmp(&a.http_context().response_time_ms())
        });

        // Generate recommendations
        let recommendations = Self::generate_recommendations(&metrics, &error_logs, &slow_requests);

        Ok(Self {
            metrics,
            analysis_period: (start_time, end_time),
            top_slow_requests: slow_requests.into_iter().take(10).collect(),
            top_errors: error_logs.into_iter().take(10).collect(),
            recommendations,
        })
    }

    fn generate_recommendations(
        metrics: &PerformanceMetrics,
        error_logs: &[LogEntry],
        slow_requests: &[LogEntry],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Error rate recommendations
        if metrics.error_rate > 0.05 {
            recommendations.push(format!(
                "High error rate detected: {:.2}%. Consider investigating the most frequent error types.",
                metrics.error_rate * 100.0
            ));
        }

        // Response time recommendations
        if metrics.avg_response_time_ms > 1000.0 {
            recommendations.push(format!(
                "Average response time is high: {:.0}ms. Consider optimizing slow endpoints.",
                metrics.avg_response_time_ms
            ));
        }

        if metrics.p95_response_time_ms > 2000 {
            recommendations.push(format!(
                "95th percentile response time is concerning: {}ms. Focus on worst-performing requests.",
                metrics.p95_response_time_ms
            ));
        }

        // Specific endpoint recommendations
        if !slow_requests.is_empty() {
            let slow_paths: std::collections::HashSet<&str> = slow_requests
                .iter()
                .map(|log| log.http_context().path())
                .collect();

            if slow_paths.len() <= 3 {
                recommendations.push(format!(
                    "Focus optimization on these slow endpoints: {}",
                    slow_paths.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        // Error pattern recommendations
        if !error_logs.is_empty() {
            let error_patterns: std::collections::HashMap<u16, usize> =
                error_logs
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut acc, log| {
                        *acc.entry(log.http_context().status_code()).or_insert(0) += 1;
                        acc
                    });

            for (status_code, count) in error_patterns {
                if count > metrics.total_requests / 20 {
                    recommendations.push(format!(
                        "Frequent {} errors detected ({} occurrences). Investigate root cause.",
                        status_code, count
                    ));
                }
            }
        }

        if recommendations.is_empty() {
            recommendations
                .push("System performance looks healthy. Continue monitoring.".to_string());
        }

        recommendations
    }
}

pub struct SearchLogsUsecase {
    log_repository: Arc<dyn LogRepositoryInterface>,
}

impl SearchLogsUsecase {
    pub fn new(log_repository: Arc<dyn LogRepositoryInterface>) -> Self {
        Self { log_repository }
    }

    pub async fn search(&self, criteria: LogSearchCriteria) -> ApplicationResult<LogSearchResult> {
        criteria
            .validate()
            .map_err(|e| ApplicationError::ValidationError(e.to_string()))?;

        self.log_repository
            .find_by_criteria(&criteria)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn analyze_performance(
        &self,
        criteria: AnalysisCriteria,
    ) -> ApplicationResult<PerformanceReport> {
        criteria.validate()?;

        let search_criteria = criteria.to_search_criteria();
        let search_result = self.search(search_criteria).await?;

        PerformanceReport::analyze(search_result.logs)
    }

    pub async fn find_by_request_id(
        &self,
        request_id: &RequestId,
    ) -> ApplicationResult<Vec<LogEntry>> {
        self.log_repository
            .find_by_request_id(request_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn find_errors_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> ApplicationResult<Vec<LogEntry>> {
        if start >= end {
            return Err(ApplicationError::ValidationError(
                "Start time must be before end time".to_string(),
            ));
        }

        self.log_repository
            .find_errors_in_range(start, end)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn get_performance_metrics(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> ApplicationResult<PerformanceMetrics> {
        if start >= end {
            return Err(ApplicationError::ValidationError(
                "Start time must be before end time".to_string(),
            ));
        }

        self.log_repository
            .get_performance_metrics(start, end)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn count_logs(&self, criteria: LogSearchCriteria) -> ApplicationResult<usize> {
        criteria
            .validate()
            .map_err(|e| ApplicationError::ValidationError(e.to_string()))?;

        self.log_repository
            .count_by_criteria(&criteria)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn find_by_id(&self, log_id: &LogId) -> ApplicationResult<Option<LogEntry>> {
        self.log_repository
            .find_by_id(log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn get_aggregated_data(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> ApplicationResult<crate::presentation::dto::log_search_response::LogAggregationResponse>
    {
        use crate::presentation::dto::log_search_response::*;
        use std::collections::HashMap;

        // エラーログを取得
        let error_logs = self.find_errors_in_range(start_time, end_time).await?;

        // パフォーマンスメトリクスを取得
        let metrics = self.get_performance_metrics(start_time, end_time).await?;

        // エラー集計
        let total_errors = error_logs.len();
        let critical_errors = error_logs
            .iter()
            .filter(|log| log.level() == crate::domain::value_object::log_level::LogLevel::Critical)
            .count();

        let error_rate_24h = if metrics.total_requests > 0 {
            error_logs.len() as f64 / metrics.total_requests as f64
        } else {
            0.0
        };

        let most_common_error = if !error_logs.is_empty() {
            let mut error_counts: HashMap<String, usize> = HashMap::new();
            for log in &error_logs {
                *error_counts.entry(log.message().to_string()).or_insert(0) += 1;
            }
            error_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(message, _)| message)
        } else {
            None
        };

        let error_summary = ErrorSummary {
            total_errors,
            critical_errors,
            error_rate_24h,
            most_common_error,
        };

        // パフォーマンス概要
        let duration_hours = (end_time - start_time).num_minutes() as f64 / 60.0;
        let throughput_requests_per_minute = if duration_hours > 0.0 {
            metrics.total_requests as f64 / (duration_hours * 60.0)
        } else {
            0.0
        };

        let performance_overview = PerformanceOverview {
            avg_response_time_24h: metrics.avg_response_time_ms,
            p95_response_time_24h: metrics.p95_response_time_ms,
            throughput_requests_per_minute,
            active_requests: 0, // リアルタイムメトリクスが必要
        };

        // アクティビティタイムライン（時間別集計）
        let mut activity_timeline = Vec::new();
        let time_intervals = 24; // 24時間分
        let interval_duration = (end_time - start_time) / time_intervals;

        for i in 0..time_intervals {
            let interval_start = start_time + interval_duration * i;
            let interval_end = interval_start + interval_duration;

            // この時間間隔のログを検索
            let criteria = LogSearchCriteria::new().with_time_range(interval_start, interval_end);

            let interval_result = self.search(criteria).await?;
            let request_count = interval_result.logs.len();
            let error_count = interval_result
                .logs
                .iter()
                .filter(|log| log.level().is_error())
                .count();

            let avg_response_time_ms = if !interval_result.logs.is_empty() {
                interval_result
                    .logs
                    .iter()
                    .map(|log| log.http_context().response_time_ms())
                    .sum::<u64>() as f64
                    / interval_result.logs.len() as f64
            } else {
                0.0
            };

            activity_timeline.push(ActivityTimelineEntry {
                timestamp: interval_start,
                request_count,
                error_count,
                avg_response_time_ms,
            });
        }

        // トップエラー
        let mut error_message_counts: HashMap<
            String,
            (
                usize,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
                Vec<String>,
            ),
        > = HashMap::new();

        for log in &error_logs {
            let message = log.message().to_string();
            let endpoint = log.http_context().endpoint().to_string();

            let entry = error_message_counts.entry(message.clone()).or_insert((
                0,
                log.timestamp(),
                log.timestamp(),
                Vec::new(),
            ));
            entry.0 += 1;
            entry.1 = entry.1.min(log.timestamp());
            entry.2 = entry.2.max(log.timestamp());
            if !entry.3.contains(&endpoint) {
                entry.3.push(endpoint);
            }
        }

        let mut top_errors: Vec<TopErrorEntry> = error_message_counts
            .into_iter()
            .map(
                |(message, (count, first_seen, last_seen, affected_endpoints))| TopErrorEntry {
                    message,
                    count,
                    first_seen,
                    last_seen,
                    affected_endpoints,
                },
            )
            .collect();
        top_errors.sort_by(|a, b| b.count.cmp(&a.count));
        top_errors.truncate(10);

        // 最も遅いエンドポイント
        let mut endpoint_metrics: HashMap<String, (Vec<u64>, usize)> = HashMap::new();

        // 全ログを取得して遅いエンドポイントを計算
        let all_criteria = LogSearchCriteria::new()
            .with_time_range(start_time, end_time)
            .with_architecture_layer(
                crate::domain::value_object::architecture_layer::ArchitectureLayer::Presentation,
            );

        let all_logs = self.search(all_criteria).await?;

        for log in &all_logs.logs {
            let endpoint = log.http_context().endpoint().to_string();
            let response_time = log.http_context().response_time_ms();

            let entry = endpoint_metrics.entry(endpoint).or_insert((Vec::new(), 0));
            entry.0.push(response_time);
            entry.1 += 1;
        }

        let mut slowest_endpoints: Vec<SlowEndpointEntry> = endpoint_metrics
            .into_iter()
            .map(|(endpoint, (response_times, request_count))| {
                let avg_response_time_ms =
                    response_times.iter().sum::<u64>() as f64 / response_times.len() as f64;
                let slowest_request_time_ms = *response_times.iter().max().unwrap_or(&0);

                SlowEndpointEntry {
                    endpoint,
                    avg_response_time_ms,
                    request_count,
                    slowest_request_time_ms,
                }
            })
            .collect();
        slowest_endpoints.sort_by(|a, b| {
            b.avg_response_time_ms
                .partial_cmp(&a.avg_response_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        slowest_endpoints.truncate(10);

        Ok(LogAggregationResponse {
            error_summary,
            performance_overview,
            activity_timeline,
            top_errors,
            slowest_endpoints,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_analysis_criteria_validation() {
        let now = Utc::now();
        let valid_criteria = AnalysisCriteria::new(now - chrono::Duration::hours(1), now);
        assert!(valid_criteria.validate().is_ok());

        let invalid_criteria = AnalysisCriteria::new(now, now - chrono::Duration::hours(1));
        assert!(invalid_criteria.validate().is_err());

        let too_long_criteria = AnalysisCriteria::new(now - chrono::Duration::days(31), now);
        assert!(too_long_criteria.validate().is_err());
    }

    #[test]
    fn test_performance_report_empty_logs() {
        let result = PerformanceReport::analyze(Vec::new());
        assert!(result.is_err());
    }
}
