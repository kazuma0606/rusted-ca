use chrono::{DateTime, Utc};

use crate::domain::value_object::{
    architecture_layer::ArchitectureLayer, http_context::HttpContext, log_level::LogLevel,
    log_metadata::LogMetadata, operation::Operation, request_id::RequestId,
    user_context::UserContext,
};

#[derive(Debug, Clone)]
pub struct CollectLogRequest {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub request_id: RequestId,
    pub user_context: Option<UserContext>,
    pub http_context: HttpContext,
    pub architecture_layer: ArchitectureLayer,
    pub operation: Operation,
    pub metadata: LogMetadata,
}

impl CollectLogRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: DateTime<Utc>,
        level: LogLevel,
        message: String,
        request_id: RequestId,
        user_context: Option<UserContext>,
        http_context: HttpContext,
        architecture_layer: ArchitectureLayer,
        operation: Operation,
        metadata: LogMetadata,
    ) -> Self {
        Self {
            timestamp,
            level,
            message,
            request_id,
            user_context,
            http_context,
            architecture_layer,
            operation,
            metadata,
        }
    }

    // Convenience constructors
    pub fn http_request(
        request_id: RequestId,
        method: String,
        path: String,
        status_code: u16,
        response_time_ms: u64,
        user_context: Option<UserContext>,
    ) -> Self {
        let level = match status_code {
            200..=299 => LogLevel::Info,
            400..=499 => LogLevel::Warn,
            500..=599 => LogLevel::Error,
            _ => LogLevel::Debug,
        };

        let http_context = HttpContext::new(
            method.clone(),
            path.clone(),
            status_code,
            response_time_ms,
            None,
            None,
        );

        let operation = match method.to_uppercase().as_str() {
            "GET" => Operation::http_get(),
            "POST" => Operation::http_post(),
            "PUT" => Operation::http_put(),
            "DELETE" => Operation::http_delete(),
            _ => Operation::new(&format!("HTTP_{}", method.to_uppercase())).unwrap_or_else(|_| Operation::http_get()),
        };

        let mut metadata = LogMetadata::new();
        metadata.add_duration_ms(response_time_ms);

        Self::new(
            Utc::now(),
            level,
            format!("{} {} - {}", method.to_uppercase(), path, status_code),
            request_id,
            user_context,
            http_context,
            ArchitectureLayer::Presentation,
            operation,
            metadata,
        )
    }

    pub fn usecase_execution(
        request_id: RequestId,
        usecase_name: &str,
        duration_ms: u64,
        success: bool,
        user_context: Option<UserContext>,
    ) -> Self {
        let level = if success { LogLevel::Info } else { LogLevel::Error };
        let operation = Operation::usecase(usecase_name).unwrap_or_else(|_| Operation::new("USECASE_UNKNOWN").unwrap());
        let message = format!("UseCase: {} - Duration: {}ms - Success: {}", usecase_name, duration_ms, success);
        
        let mut metadata = LogMetadata::new();
        metadata.add_duration_ms(duration_ms);
        metadata.add_string("usecase_name", usecase_name);
        metadata.add_bool("success", success);

        Self::new(
            Utc::now(),
            level,
            message,
            request_id,
            user_context,
            HttpContext::new("INTERNAL".to_string(), "".to_string(), 0, 0, None, None), // Placeholder HTTP context
            ArchitectureLayer::Application,
            operation,
            metadata,
        )
    }

    pub fn domain_operation(
        request_id: RequestId,
        operation_name: &str,
        entity_type: &str,
        success: bool,
        user_context: Option<UserContext>,
    ) -> Self {
        let level = if success { LogLevel::Info } else { LogLevel::Error };
        let operation = Operation::domain_service(operation_name).unwrap_or_else(|_| Operation::new("DOMAIN_UNKNOWN").unwrap());
        let message = format!("Domain: {} - Entity: {} - Success: {}", operation_name, entity_type, success);
        
        let mut metadata = LogMetadata::new();
        metadata.add_string("entity_type", entity_type);
        metadata.add_bool("success", success);

        Self::new(
            Utc::now(),
            level,
            message,
            request_id,
            user_context,
            HttpContext::new("INTERNAL".to_string(), "".to_string(), 0, 0, None, None), // Placeholder HTTP context
            ArchitectureLayer::Domain,
            operation,
            metadata,
        )
    }

    pub fn infrastructure_operation(
        request_id: RequestId,
        operation_name: &str,
        resource_type: &str,
        duration_ms: u64,
        success: bool,
        error_message: Option<&str>,
        user_context: Option<UserContext>,
    ) -> Self {
        let level = if success { LogLevel::Info } else { LogLevel::Error };
        let operation = Operation::repository(operation_name).unwrap_or_else(|_| Operation::new("INFRA_UNKNOWN").unwrap());
        let message = if success {
            format!("Infrastructure: {} - Resource: {} - Duration: {}ms", operation_name, resource_type, duration_ms)
        } else {
            format!(
                "Infrastructure: {} - Resource: {} - Duration: {}ms - Error: {}",
                operation_name,
                resource_type,
                duration_ms,
                error_message.unwrap_or("Unknown error")
            )
        };
        
        let mut metadata = LogMetadata::new();
        metadata.add_string("resource_type", resource_type);
        metadata.add_duration_ms(duration_ms);
        metadata.add_bool("success", success);
        if let Some(error) = error_message {
            metadata.add_error_info("InfrastructureError", error);
        }

        Self::new(
            Utc::now(),
            level,
            message,
            request_id,
            user_context,
            HttpContext::new("INTERNAL".to_string(), "".to_string(), 0, 0, None, None), // Placeholder HTTP context
            ArchitectureLayer::Infrastructure,
            operation,
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_http_request_constructor() {
        let request_id = RequestId::from(Uuid::new_v4());
        let request = CollectLogRequest::http_request(
            request_id.clone(),
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            150,
            None,
        );

        assert_eq!(request.level, LogLevel::Info);
        assert_eq!(request.request_id, request_id);
        assert_eq!(request.http_context.method(), "GET");
        assert_eq!(request.http_context.status_code(), 200);
        assert_eq!(request.http_context.response_time_ms(), 150);
        assert_eq!(request.architecture_layer, ArchitectureLayer::Presentation);
    }

    #[test]
    fn test_usecase_execution_constructor() {
        let request_id = RequestId::from(Uuid::new_v4());
        let request = CollectLogRequest::usecase_execution(
            request_id.clone(),
            "CreateUser",
            250,
            true,
            None,
        );

        assert_eq!(request.level, LogLevel::Info);
        assert_eq!(request.architecture_layer, ArchitectureLayer::Application);
        assert!(request.message.contains("CreateUser"));
        assert!(request.message.contains("250ms"));
    }

    #[test]
    fn test_error_status_code_mapping() {
        let request_id = RequestId::from(Uuid::new_v4());
        
        let error_request = CollectLogRequest::http_request(
            request_id,
            "POST".to_string(),
            "/api/users".to_string(),
            500,
            1000,
            None,
        );

        assert_eq!(error_request.level, LogLevel::Error);
    }
}