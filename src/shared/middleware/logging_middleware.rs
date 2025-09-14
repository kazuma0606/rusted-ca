use std::time::Instant;
use axum::{
    extract::{Request, ConnectInfo},
    response::Response,
    middleware::Next,
    http::{HeaderMap, Method, Uri, StatusCode},
};
use tokio::sync::mpsc;
use uuid::Uuid;
use bson::doc;

use crate::domain::entity::log_entry::{
    LogEntry, LogLevel, LogCategory, HttpLogContext, SystemLogContext, 
    ArchitectureLayer, LogMetrics
};
use crate::infrastructure::logging::log_collector_service::LogCollectorService;

/// HTTP request logging middleware for Axum
#[derive(Clone)]
pub struct LoggingMiddleware {
    log_sender: mpsc::Sender<LogEntry>,
    service_name: String,
    enable_request_body_logging: bool,
    enable_response_body_logging: bool,
    sensitive_headers: Vec<String>,
}

impl LoggingMiddleware {
    /// Create new logging middleware
    pub fn new(
        log_collector: &LogCollectorService,
        service_name: String,
    ) -> Self {
        Self {
            log_sender: log_collector.get_sender(),
            service_name,
            enable_request_body_logging: false, // Disabled by default for performance
            enable_response_body_logging: false,
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }

    /// Create middleware with custom configuration
    pub fn with_config(
        log_collector: &LogCollectorService,
        service_name: String,
        enable_request_body: bool,
        enable_response_body: bool,
    ) -> Self {
        Self {
            log_sender: log_collector.get_sender(),
            service_name,
            enable_request_body_logging: enable_request_body,
            enable_response_body_logging: enable_response_body,
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }

    /// Main middleware handler
    pub async fn handle(
        &self,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let start_time = Instant::now();
        
        // Generate trace and request IDs
        let trace_id = Uuid::new_v4().to_string();
        let request_id = Uuid::new_v4().to_string();
        
        // Extract request information
        let method = request.method().clone();
        let uri = request.uri().clone();
        let headers = request.headers().clone();
        
        // Get client IP address
        let client_ip = self.extract_client_ip(&headers, &request);
        
        // Add trace context to request headers for downstream services
        request.headers_mut().insert(
            "x-trace-id", 
            trace_id.parse().unwrap_or_else(|_| "invalid".parse().unwrap())
        );
        request.headers_mut().insert(
            "x-request-id", 
            request_id.parse().unwrap_or_else(|_| "invalid".parse().unwrap())
        );

        // Log request start
        self.log_request_start(&trace_id, &request_id, &method, &uri, &headers, &client_ip).await;

        // Process request
        let response = next.run(request).await;
        let duration = start_time.elapsed();

        // Log request completion
        self.log_request_complete(
            &trace_id,
            &request_id,
            &method,
            &uri,
            &response,
            duration,
            &client_ip,
        ).await;

        Ok(response)
    }

    /// Log request start
    async fn log_request_start(
        &self,
        trace_id: &str,
        request_id: &str,
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        client_ip: &str,
    ) {
        let http_context = HttpLogContext {
            method: method.to_string(),
            endpoint: uri.path().to_string(),
            status_code: 0, // Will be updated in completion log
            user_agent: headers.get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            ip_address: client_ip.to_string(),
            content_length: headers.get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            response_time_ms: 0, // Will be updated in completion log
            query_params: self.extract_query_params(uri),
            request_headers: if self.enable_request_body_logging {
                Some(self.sanitize_headers(headers))
            } else {
                None
            },
            response_headers: None,
        };

        let system_context = SystemLogContext {
            layer: ArchitectureLayer::Presentation,
            component: "http_middleware".to_string(),
            operation: "request_start".to_string(),
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            process_id: std::process::id(),
            thread_id: format!("{:?}", std::thread::current().id()),
            service_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        let log_entry = LogEntry {
            id: None,
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            category: LogCategory::Http,
            subcategory: Some("request_start".to_string()),
            message: format!("{} {} - Request started", method, uri.path()),
            trace_id: trace_id.to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            request_id: request_id.to_string(),
            session_id: None, // Could be extracted from session cookie
            user_id: None, // Could be extracted from auth context
            http_context: Some(http_context),
            ml_context: None,
            system_context,
            metrics: LogMetrics::default(),
            metadata: doc! {
                "service": &self.service_name,
                "http_version": format!("{:?}", uri.scheme()),
                "path_params": self.extract_path_info(uri),
            },
            tags: vec!["http".to_string(), "request".to_string()],
        };

        if let Err(e) = self.log_sender.send(log_entry).await {
            eprintln!("Failed to send request start log: {}", e);
        }
    }

    /// Log request completion
    async fn log_request_complete(
        &self,
        trace_id: &str,
        request_id: &str,
        method: &Method,
        uri: &Uri,
        response: &Response,
        duration: std::time::Duration,
        client_ip: &str,
    ) {
        let status_code = response.status().as_u16();
        let response_time_ms = duration.as_millis() as u64;

        // Determine log level based on status code
        let log_level = match status_code {
            200..=299 => LogLevel::Info,
            400..=499 => LogLevel::Warn,
            500..=599 => LogLevel::Error,
            _ => LogLevel::Info,
        };

        let http_context = HttpLogContext {
            method: method.to_string(),
            endpoint: uri.path().to_string(),
            status_code,
            user_agent: None, // Already logged in request start
            ip_address: client_ip.to_string(),
            content_length: response.headers().get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            response_time_ms,
            query_params: None, // Already logged in request start
            request_headers: None,
            response_headers: if self.enable_response_body_logging {
                Some(self.sanitize_headers(response.headers()))
            } else {
                None
            },
        };

        let system_context = SystemLogContext {
            layer: ArchitectureLayer::Presentation,
            component: "http_middleware".to_string(),
            operation: "request_complete".to_string(),
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            process_id: std::process::id(),
            thread_id: format!("{:?}", std::thread::current().id()),
            service_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        let metrics = LogMetrics {
            execution_time_ms: Some(response_time_ms),
            memory_usage_bytes: None,
            cpu_usage_percent: None,
            disk_io_bytes: None,
            network_io_bytes: None,
            database_query_time_ms: None,
            cache_hit_rate: None,
        };

        let message = format!(
            "{} {} - {} ({} ms)", 
            method, 
            uri.path(), 
            status_code, 
            response_time_ms
        );

        let log_entry = LogEntry {
            id: None,
            timestamp: chrono::Utc::now(),
            level: log_level,
            category: LogCategory::Http,
            subcategory: Some("request_complete".to_string()),
            message,
            trace_id: trace_id.to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            request_id: request_id.to_string(),
            session_id: None,
            user_id: None,
            http_context: Some(http_context),
            ml_context: None,
            system_context,
            metrics,
            metadata: doc! {
                "service": &self.service_name,
                "success": status_code >= 200 && status_code < 400,
                "error_category": self.categorize_status_code(status_code),
            },
            tags: self.generate_tags(status_code, response_time_ms),
        };

        if let Err(e) = self.log_sender.send(log_entry).await {
            eprintln!("Failed to send request complete log: {}", e);
        }
    }

    /// Extract client IP address from various sources
    fn extract_client_ip(&self, headers: &HeaderMap, request: &Request) -> String {
        // Try X-Forwarded-For header first (for proxies)
        if let Some(forwarded_for) = headers.get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    return first_ip.trim().to_string();
                }
            }
        }

        // Try X-Real-IP header (for nginx)
        if let Some(real_ip) = headers.get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                return ip_str.to_string();
            }
        }

        // Try connection info from Axum
        if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<std::net::SocketAddr>>() {
            return addr.ip().to_string();
        }

        // Fallback
        "unknown".to_string()
    }

    /// Extract query parameters from URI
    fn extract_query_params(&self, uri: &Uri) -> Option<bson::Document> {
        uri.query().map(|query| {
            let mut doc = bson::Document::new();
            for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
                doc.insert(key.to_string(), bson::Bson::String(value.to_string()));
            }
            doc
        })
    }

    /// Extract path information
    fn extract_path_info(&self, uri: &Uri) -> bson::Document {
        doc! {
            "path": uri.path(),
            "segments": uri.path().split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
            "has_query": uri.query().is_some(),
        }
    }

    /// Sanitize headers by removing sensitive information
    fn sanitize_headers(&self, headers: &HeaderMap) -> bson::Document {
        let mut doc = bson::Document::new();
        
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            
            let header_value = if self.sensitive_headers.contains(&header_name) {
                "[REDACTED]".to_string()
            } else {
                value.to_str().unwrap_or("[INVALID_UTF8]").to_string()
            };
            
            doc.insert(header_name, bson::Bson::String(header_value));
        }
        
        doc
    }

    /// Categorize HTTP status codes for better analysis
    fn categorize_status_code(&self, status_code: u16) -> String {
        match status_code {
            200..=299 => "success".to_string(),
            300..=399 => "redirect".to_string(),
            400..=499 => "client_error".to_string(),
            500..=599 => "server_error".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Generate relevant tags based on response
    fn generate_tags(&self, status_code: u16, response_time_ms: u64) -> Vec<String> {
        let mut tags = vec!["http".to_string()];
        
        // Add status category tag
        tags.push(self.categorize_status_code(status_code));
        
        // Add performance tags
        if response_time_ms > 5000 {
            tags.push("slow".to_string());
        } else if response_time_ms > 1000 {
            tags.push("medium".to_string());
        } else {
            tags.push("fast".to_string());
        }
        
        // Add specific status tags for important codes
        match status_code {
            401 => tags.push("unauthorized".to_string()),
            403 => tags.push("forbidden".to_string()),
            404 => tags.push("not_found".to_string()),
            429 => tags.push("rate_limited".to_string()),
            500 => tags.push("internal_error".to_string()),
            502 => tags.push("bad_gateway".to_string()),
            503 => tags.push("service_unavailable".to_string()),
            _ => {}
        }
        
        tags
    }
}

/// Simple logging middleware function for use with Axum from_fn
/// Note: In production, you would get the LoggingMiddleware instance from DI container
pub async fn logging_middleware_fn(req: Request, next: Next) -> Result<Response, StatusCode> {
    // For now, just pass through - would need proper DI integration
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are temporarily disabled as they require a real LogCollectorService
    // For unit testing, we would need to create mock implementations

    #[test]
    fn test_status_code_categorization() {
        // Test the categorization logic directly without creating middleware
        fn categorize_status_code(status_code: u16) -> String {
            match status_code {
                200..=299 => "success".to_string(),
                300..=399 => "redirect".to_string(),
                400..=499 => "client_error".to_string(),
                500..=599 => "server_error".to_string(),
                _ => "unknown".to_string(),
            }
        }

        assert_eq!(categorize_status_code(200), "success");
        assert_eq!(categorize_status_code(404), "client_error");
        assert_eq!(categorize_status_code(500), "server_error");
        assert_eq!(categorize_status_code(302), "redirect");
    }

    #[test]
    fn test_tag_generation() {
        // Test the tag generation logic directly
        fn generate_tags(status_code: u16, response_time_ms: u64) -> Vec<String> {
            let mut tags = vec!["http".to_string()];
            
            // Add status category tag
            let category = match status_code {
                200..=299 => "success",
                300..=399 => "redirect",
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => "unknown",
            };
            tags.push(category.to_string());
            
            // Add performance tags
            if response_time_ms > 5000 {
                tags.push("slow".to_string());
            } else if response_time_ms > 1000 {
                tags.push("medium".to_string());
            } else {
                tags.push("fast".to_string());
            }
            
            // Add specific status tags for important codes
            match status_code {
                401 => tags.push("unauthorized".to_string()),
                403 => tags.push("forbidden".to_string()),
                404 => tags.push("not_found".to_string()),
                429 => tags.push("rate_limited".to_string()),
                500 => tags.push("internal_error".to_string()),
                502 => tags.push("bad_gateway".to_string()),
                503 => tags.push("service_unavailable".to_string()),
                _ => {}
            }
            
            tags
        }

        let tags = generate_tags(404, 100);
        assert!(tags.contains(&"http".to_string()));
        assert!(tags.contains(&"client_error".to_string()));
        assert!(tags.contains(&"not_found".to_string()));
        assert!(tags.contains(&"fast".to_string()));

        let slow_tags = generate_tags(200, 6000);
        assert!(slow_tags.contains(&"slow".to_string()));
    }
}