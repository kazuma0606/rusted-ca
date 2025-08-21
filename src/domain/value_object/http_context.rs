use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpContext {
    method: String,
    path: String,
    status_code: u16,
    response_time_ms: u64,
    user_agent: Option<String>,
    ip_address: Option<String>,
}

impl HttpContext {
    pub fn new(
        method: String,
        path: String,
        status_code: u16,
        response_time_ms: u64,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Self {
        Self {
            method,
            path,
            status_code,
            response_time_ms,
            user_agent,
            ip_address,
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn response_time_ms(&self) -> u64 {
        self.response_time_ms
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status_code)
    }

    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500
    }
}

impl fmt::Display for HttpContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}ms",
            self.method, self.path, self.status_code, self.response_time_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_context_new() {
        let context = HttpContext::new(
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            150,
            Some("Mozilla/5.0".to_string()),
            Some("192.168.1.1".to_string()),
        );

        assert_eq!(context.method(), "GET");
        assert_eq!(context.path(), "/api/users");
        assert_eq!(context.status_code(), 200);
        assert_eq!(context.response_time_ms(), 150);
        assert_eq!(context.user_agent(), Some("Mozilla/5.0"));
        assert_eq!(context.ip_address(), Some("192.168.1.1"));
    }

    #[test]
    fn test_status_code_checks() {
        let success_context = HttpContext::new(
            "GET".to_string(),
            "/".to_string(),
            200,
            100,
            None,
            None,
        );
        assert!(success_context.is_success());
        assert!(!success_context.is_client_error());
        assert!(!success_context.is_server_error());

        let client_error_context = HttpContext::new(
            "GET".to_string(),
            "/".to_string(),
            404,
            100,
            None,
            None,
        );
        assert!(!client_error_context.is_success());
        assert!(client_error_context.is_client_error());
        assert!(!client_error_context.is_server_error());

        let server_error_context = HttpContext::new(
            "GET".to_string(),
            "/".to_string(),
            500,
            100,
            None,
            None,
        );
        assert!(!server_error_context.is_success());
        assert!(!server_error_context.is_client_error());
        assert!(server_error_context.is_server_error());
    }
}