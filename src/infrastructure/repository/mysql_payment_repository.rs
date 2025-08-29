//infrastructure/repository/mysql_payment_repository.rs
// MySQL Payment Repository Implementation
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::repository::payment_command_repository::PaymentCommandRepository;
use crate::domain::repository::payment_query_repository::{
    PaymentQueryRepository, PaymentSearchCriteria, PaymentStats,
};
use crate::domain::value_object::{
    AccountId, Money, PaymentId, PaymentMethod, PaymentStatus, ReferenceNumber,
};
use crate::shared::error::domain_error::{DomainError, DomainResult};
use crate::shared::error::infrastructure_error::{InfrastructureError, InfrastructureResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool, Row, types::BigDecimal};
use std::sync::Arc;

pub struct MySqlPaymentRepository {
    pool: Arc<Pool<MySql>>,
}

impl MySqlPaymentRepository {
    pub fn new(pool: Arc<Pool<MySql>>) -> Self {
        Self { pool }
    }

    /// Convert database row to Payment entity
    fn row_to_payment(&self, row: &sqlx::mysql::MySqlRow) -> DomainResult<Payment> {
        let id = PaymentId::new(row.get::<String, _>("id"))?;
        let account_id = AccountId::new(row.get::<String, _>("account_id"))?;
        let amount_cents: i64 = row.get("amount_cents");
        let currency_code: String = row.get("currency_code");
        let amount = Money::new(amount_cents, currency_code)?;

        let payment_method_str: String = row.get("payment_method");
        let payment_method = PaymentMethod::from_str(&payment_method_str)?;

        let status_str: String = row.get("payment_status");
        let status = PaymentStatus::from_str(&status_str)?;

        let reference_number = ReferenceNumber::new(row.get::<String, _>("reference_number"))?;

        let description: Option<String> = row.get("description");
        let customer_email: Option<String> = row.get("customer_email");

        let metadata_json: Option<String> = row.get("metadata");
        let metadata = metadata_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| DomainError::InvalidValue(format!("Invalid metadata JSON: {}", e)))?;

        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        Payment::new(
            id,
            account_id,
            amount,
            payment_method,
            status,
            reference_number,
            description,
            customer_email,
            metadata,
            created_at,
            updated_at,
        )
    }

    /// Convert Payment entity to database parameters
    fn payment_to_params(
        &self,
        payment: &Payment,
    ) -> (
        String,         // id
        String,         // account_id
        i64,            // amount_cents
        String,         // currency_code
        String,         // payment_method
        String,         // payment_status
        String,         // reference_number
        Option<String>, // description
        Option<String>, // customer_email
        Option<String>, // metadata
        DateTime<Utc>,  // created_at
        DateTime<Utc>,  // updated_at
    ) {
        let metadata_json = payment
            .metadata()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        (
            payment.id().value().to_string(),
            payment.account_id().value().to_string(),
            payment.amount().amount_cents(),
            payment.amount().currency_code().to_string(),
            payment.payment_method().as_str().to_string(),
            payment.status().as_str().to_string(),
            payment.reference_number().value().to_string(),
            payment.description().cloned(),
            payment.customer_email().cloned(),
            metadata_json,
            *payment.created_at(),
            *payment.updated_at(),
        )
    }
}

#[async_trait]
impl PaymentCommandRepository for MySqlPaymentRepository {
    async fn create(&self, payment: &Payment) -> DomainResult<()> {
        let params = self.payment_to_params(payment);

        let result = sqlx::query!(
            r#"
            INSERT INTO payments (
                id, account_id, amount_cents, currency_code, payment_method, 
                payment_status, reference_number, description, customer_email, 
                metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params.0,
            params.1,
            params.2,
            params.3,
            params.4,
            params.5,
            params.6,
            params.7,
            params.8,
            params.9,
            params.10,
            params.11
        )
        .execute(&*self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DomainError::BusinessRuleViolation {
                    rule: "UniquePaymentId".to_string(),
                    message: format!("Payment with ID {} already exists", payment.id()),
                })
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn update(&self, payment: &Payment) -> DomainResult<()> {
        let params = self.payment_to_params(payment);

        let result = sqlx::query!(
            r#"
            UPDATE payments 
            SET account_id = ?, amount_cents = ?, currency_code = ?, 
                payment_method = ?, payment_status = ?, reference_number = ?, 
                description = ?, customer_email = ?, metadata = ?, updated_at = ?
            WHERE id = ?
            "#,
            params.1,
            params.2,
            params.3,
            params.4,
            params.5,
            params.6,
            params.7,
            params.8,
            params.9,
            params.11,
            params.0
        )
        .execute(&*self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DomainError::InvalidOperation(format!(
                        "Payment with ID {} not found",
                        payment.id()
                    )))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn delete(&self, payment_id: &PaymentId) -> DomainResult<()> {
        let result = sqlx::query!("DELETE FROM payments WHERE id = ?", payment_id.value())
            .execute(&*self.pool)
            .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DomainError::InvalidOperation(format!(
                        "Payment with ID {} not found",
                        payment_id
                    )))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn exists_by_id(&self, payment_id: &PaymentId) -> DomainResult<bool> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM payments WHERE id = ?",
            payment_id.value()
        )
        .fetch_one(&*self.pool)
        .await;

        match result {
            Ok(row) => Ok(row.count > 0),
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn is_reference_unique(&self, reference_number: &ReferenceNumber) -> DomainResult<bool> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM payments WHERE reference_number = ?",
            reference_number.value()
        )
        .fetch_one(&*self.pool)
        .await;

        match result {
            Ok(row) => Ok(row.count == 0),
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn begin_transaction(&self) -> DomainResult<()> {
        // Transaction management would be handled at a higher level
        // This is a placeholder for the interface
        Ok(())
    }

    async fn commit_transaction(&self) -> DomainResult<()> {
        // Transaction management would be handled at a higher level
        // This is a placeholder for the interface
        Ok(())
    }

    async fn rollback_transaction(&self) -> DomainResult<()> {
        // Transaction management would be handled at a higher level
        // This is a placeholder for the interface
        Ok(())
    }
}

#[async_trait]
impl PaymentQueryRepository for MySqlPaymentRepository {
    async fn find_by_id(&self, payment_id: &PaymentId) -> DomainResult<Option<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE id = ?
            "#,
        )
        .bind(payment_id.value())
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(self.row_to_payment(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn find_by_reference(
        &self,
        reference_number: &ReferenceNumber,
    ) -> DomainResult<Option<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE reference_number = ?
            "#,
        )
        .bind(reference_number.value())
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(self.row_to_payment(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn find_by_account_id(&self, account_id: &AccountId) -> DomainResult<Vec<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE account_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(account_id.value())
        .fetch_all(&*self.pool)
        .await;

        match result {
            Ok(rows) => {
                let mut payments = Vec::new();
                for row in rows {
                    payments.push(self.row_to_payment(&row)?);
                }
                Ok(payments)
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn find_by_status(&self, status: &PaymentStatus) -> DomainResult<Vec<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE payment_status = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(status.as_str())
        .fetch_all(&*self.pool)
        .await;

        match result {
            Ok(rows) => {
                let mut payments = Vec::new();
                for row in rows {
                    payments.push(self.row_to_payment(&row)?);
                }
                Ok(payments)
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn search(&self, criteria: &PaymentSearchCriteria) -> DomainResult<Vec<Payment>> {
        // This is a simplified implementation - a real implementation would use dynamic query building
        let mut query = String::from(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE 1=1
            "#,
        );

        let mut params: Vec<Box<dyn sqlx::Encode<'_, MySql> + Send + Sync>> = Vec::new();

        if let Some(account_id) = &criteria.account_id {
            query.push_str(" AND account_id = ?");
            params.push(Box::new(account_id.value().to_string()));
        }

        if let Some(status) = &criteria.status {
            query.push_str(" AND payment_status = ?");
            params.push(Box::new(status.as_str().to_string()));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = criteria.limit {
            query.push_str(" LIMIT ?");
            params.push(Box::new(limit));
        }

        if let Some(offset) = criteria.offset {
            query.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        // Due to the complexity of dynamic query building with sqlx,
        // this is a placeholder. A real implementation would use a query builder
        // or construct the query more carefully.

        Err(DomainError::InvalidOperation(
            "Search not fully implemented yet".to_string(),
        ))
    }

    async fn count(&self, _criteria: &PaymentSearchCriteria) -> DomainResult<u64> {
        // Placeholder implementation
        Err(DomainError::InvalidOperation(
            "Count not implemented yet".to_string(),
        ))
    }

    async fn find_requiring_processing(
        &self,
        older_than_minutes: u32,
    ) -> DomainResult<Vec<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE payment_status = 'PENDING' 
            AND created_at < DATE_SUB(NOW(), INTERVAL ? MINUTE)
            ORDER BY created_at ASC
            "#,
        )
        .bind(older_than_minutes)
        .fetch_all(&*self.pool)
        .await;

        match result {
            Ok(rows) => {
                let mut payments = Vec::new();
                for row in rows {
                    payments.push(self.row_to_payment(&row)?);
                }
                Ok(payments)
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn find_successful_payments(
        &self,
        account_id: &AccountId,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> DomainResult<Vec<Payment>> {
        let result = sqlx::query(
            r#"
            SELECT id, account_id, amount_cents, currency_code, payment_method,
                   payment_status, reference_number, description, customer_email,
                   metadata, created_at, updated_at
            FROM payments 
            WHERE account_id = ? 
            AND payment_status = 'SUCCESS'
            AND created_at >= ? AND created_at <= ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(account_id.value())
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&*self.pool)
        .await;

        match result {
            Ok(rows) => {
                let mut payments = Vec::new();
                for row in rows {
                    payments.push(self.row_to_payment(&row)?);
                }
                Ok(payments)
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }

    async fn get_payment_stats(&self, account_id: &AccountId) -> DomainResult<PaymentStats> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_payments,
                COUNT(CASE WHEN payment_status = 'SUCCESS' THEN 1 END) as successful_payments,
                COUNT(CASE WHEN payment_status = 'FAILED' THEN 1 END) as failed_payments,
                COUNT(CASE WHEN payment_status = 'PENDING' THEN 1 END) as pending_payments,
                COALESCE(SUM(amount_cents), 0) as total_amount_cents,
                COALESCE(SUM(CASE WHEN payment_status = 'SUCCESS' THEN amount_cents ELSE 0 END), 0) as successful_amount_cents,
                COALESCE(AVG(amount_cents), 0) as average_amount_cents,
                COALESCE(MIN(currency_code), 'USD') as currency_code
            FROM payments 
            WHERE account_id = ?
            "#,
        )
        .bind(account_id.value())
        .fetch_one(&*self.pool)
        .await;

        match result {
            Ok(row) => {
                let total_payments: i64 = row.try_get("total_payments")?;
                let successful_payments: i64 = row.try_get("successful_payments")?;
                let failed_payments: i64 = row.try_get("failed_payments")?;
                let pending_payments: i64 = row.try_get("pending_payments")?;

                // Handle DECIMAL types that come as BigDecimal
                let total_amount_cents_bd: BigDecimal = row.try_get("total_amount_cents")?;
                let successful_amount_cents_bd: BigDecimal =
                    row.try_get("successful_amount_cents")?;
                let average_amount_cents_bd: BigDecimal = row.try_get("average_amount_cents")?;

                let total_amount_cents = total_amount_cents_bd
                    .to_string()
                    .parse::<i64>()
                    .unwrap_or(0);
                let successful_amount_cents = successful_amount_cents_bd
                    .to_string()
                    .parse::<i64>()
                    .unwrap_or(0);
                let average_amount_cents = average_amount_cents_bd
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let currency_code: String = row.try_get("currency_code")?;

                Ok(PaymentStats {
                    total_payments: total_payments as u64,
                    successful_payments: successful_payments as u64,
                    failed_payments: failed_payments as u64,
                    pending_payments: pending_payments as u64,
                    total_amount_cents,
                    successful_amount_cents,
                    average_amount_cents: average_amount_cents as i64,
                    currency_code,
                })
            }
            Err(e) => Err(DomainError::InvalidOperation(format!(
                "Database error: {}",
                e
            ))),
        }
    }
}
