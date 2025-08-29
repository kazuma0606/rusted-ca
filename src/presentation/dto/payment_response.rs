//presentation/dto/payment_response.rs
// Payment Response DTO
// 2025/8/29

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentResponse {
    pub id: String,
    pub account_id: String,
    pub amount: f64,
    pub currency_code: String,
    pub payment_method: String,
    pub status: String,
    pub reference_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

impl PaymentResponse {
    pub fn new(
        id: String,
        account_id: String,
        amount: f64,
        currency_code: String,
        payment_method: String,
        status: String,
        reference_number: String,
        description: Option<String>,
        customer_email: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            account_id,
            amount,
            currency_code,
            payment_method,
            status,
            reference_number,
            description,
            customer_email,
            metadata,
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentListResponse {
    pub payments: Vec<PaymentResponse>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub has_next: bool,
}

impl PaymentListResponse {
    pub fn new(
        payments: Vec<PaymentResponse>,
        total_count: u64,
        page: u64,
        page_size: u64,
    ) -> Self {
        let has_next = (page * page_size) < total_count;
        Self {
            payments,
            total_count,
            page,
            page_size,
            has_next,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentStatsResponse {
    pub account_id: String,
    pub total_payments: u64,
    pub successful_payments: u64,
    pub failed_payments: u64,
    pub pending_payments: u64,
    pub refunded_payments: u64,
    pub total_amount: f64,
    pub successful_amount: f64,
    pub currency_code: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
}

impl PaymentStatsResponse {
    pub fn new(
        account_id: String,
        total_payments: u64,
        successful_payments: u64,
        failed_payments: u64,
        pending_payments: u64,
        refunded_payments: u64,
        total_amount: f64,
        successful_amount: f64,
        currency_code: String,
    ) -> Self {
        Self {
            account_id,
            total_payments,
            successful_payments,
            failed_payments,
            pending_payments,
            refunded_payments,
            total_amount,
            successful_amount,
            currency_code,
            period_start: None,
            period_end: None,
        }
    }

    pub fn with_period(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.period_start = Some(start.to_rfc3339());
        self.period_end = Some(end.to_rfc3339());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_payment_response_creation() {
        let now = Utc::now();
        let response = PaymentResponse::new(
            "payment-123".to_string(),
            "account-456".to_string(),
            100.50,
            "USD".to_string(),
            "CARD".to_string(),
            "SUCCESS".to_string(),
            "REF-789".to_string(),
            Some("Test payment".to_string()),
            Some("customer@example.com".to_string()),
            None,
            now,
            now,
        );

        assert_eq!(response.id, "payment-123");
        assert_eq!(response.account_id, "account-456");
        assert_eq!(response.amount, 100.50);
        assert_eq!(response.status, "SUCCESS");
    }

    #[test]
    fn test_payment_list_response_pagination() {
        let payments = vec![];
        let response = PaymentListResponse::new(payments, 150, 3, 50);

        assert_eq!(response.total_count, 150);
        assert_eq!(response.page, 3);
        assert_eq!(response.page_size, 50);
        assert!(!response.has_next); // 3 * 50 = 150, so no next page
    }

    #[test]
    fn test_payment_stats_response() {
        let stats = PaymentStatsResponse::new(
            "account-123".to_string(),
            100,
            80,
            10,
            5,
            5,
            10000.0,
            8000.0,
            "USD".to_string(),
        );

        assert_eq!(stats.total_payments, 100);
        assert_eq!(stats.successful_payments, 80);
        assert_eq!(stats.successful_amount, 8000.0);
    }
}