use crate::domain::entity::account::Account;
use crate::domain::entity_sqlx::account_sqlx::AccountSqlx;
use crate::domain::repository::account_command_repository::AccountCommandRepositoryInterface;
use crate::domain::repository::account_query_repository::{AccountQueryRepositoryInterface, AccountSearchFilters};
use crate::domain::value_object::{AccountId, AccountStatus, Email, Money, pagination::*};
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct MySqlAccountRepository {
    pub pool: MySqlPool,
}

impl MySqlAccountRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

// Command Repository Implementation
#[async_trait]
impl AccountCommandRepositoryInterface for MySqlAccountRepository {
    async fn save(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let account_sqlx = AccountSqlx::from_domain(account);
        
        let query = r#"
            INSERT INTO accounts (id, merchant_name, email, account_status, balance_cents, currency_code, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        sqlx::query(query)
            .bind(&account_sqlx.id)
            .bind(&account_sqlx.merchant_name)
            .bind(&account_sqlx.email)
            .bind(account_sqlx.account_status.to_string())
            .bind(account_sqlx.balance_cents)
            .bind(&account_sqlx.currency_code)
            .bind(&account_sqlx.created_at)
            .bind(&account_sqlx.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        Ok(())
    }

    async fn update(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let account_sqlx = AccountSqlx::from_domain(account);
        
        let query = r#"
            UPDATE accounts 
            SET merchant_name = ?, email = ?, account_status = ?, balance_cents = ?, currency_code = ?, updated_at = ?
            WHERE id = ?
        "#;
        
        let result = sqlx::query(query)
            .bind(&account_sqlx.merchant_name)
            .bind(&account_sqlx.email)
            .bind(account_sqlx.account_status.to_string())
            .bind(account_sqlx.balance_cents)
            .bind(&account_sqlx.currency_code)
            .bind(&account_sqlx.updated_at)
            .bind(&account_sqlx.id)
            .execute(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
            
        if result.rows_affected() == 0 {
            return Err(Box::new(InfrastructureError::DatabaseOperation(
                format!("Account with id {} not found", account_sqlx.id)
            )));
        }
        
        Ok(())
    }

    async fn delete(&self, account_id: &AccountId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = "DELETE FROM accounts WHERE id = ?";
        
        let result = sqlx::query(query)
            .bind(account_id.value())
            .execute(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
            
        if result.rows_affected() == 0 {
            return Err(Box::new(InfrastructureError::DatabaseOperation(
                format!("Account with id {} not found for deletion", account_id.value())
            )));
        }
        
        Ok(())
    }

    async fn update_balance(
        &self,
        account_id: &AccountId,
        new_balance: &Money,
        updated_at: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            UPDATE accounts 
            SET balance_cents = ?, currency_code = ?, updated_at = ?
            WHERE id = ?
        "#;
        
        let result = sqlx::query(query)
            .bind(new_balance.amount_cents())
            .bind(new_balance.currency_code())
            .bind(updated_at.naive_utc())
            .bind(account_id.value())
            .execute(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
            
        if result.rows_affected() == 0 {
            return Err(Box::new(InfrastructureError::DatabaseOperation(
                format!("Account with id {} not found for balance update", account_id.value())
            )));
        }
        
        Ok(())
    }

    async fn save_batch(&self, accounts: &[Account]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = self.pool.begin().await.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(InfrastructureError::DatabaseQuery {
                query: "BEGIN TRANSACTION".to_string(),
                message: e.to_string(),
            })
        })?;
        
        let query = r#"
            INSERT INTO accounts (id, merchant_name, email, account_status, balance_cents, currency_code, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        for account in accounts {
            let account_sqlx = AccountSqlx::from_domain(account);
            
            sqlx::query(query)
                .bind(&account_sqlx.id)
                .bind(&account_sqlx.merchant_name)
                .bind(&account_sqlx.email)
                .bind(account_sqlx.account_status.to_string())
                .bind(account_sqlx.balance_cents)
                .bind(&account_sqlx.currency_code)
                .bind(&account_sqlx.created_at)
                .bind(&account_sqlx.updated_at)
                .execute(&mut *tx)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(InfrastructureError::DatabaseQuery {
                        query: query.to_string(),
                        message: e.to_string(),
                    })
                })?;
        }
        
        tx.commit().await.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(InfrastructureError::DatabaseQuery {
                query: "COMMIT TRANSACTION".to_string(),
                message: e.to_string(),
            })
        })?;
        
        Ok(())
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT COUNT(*) as count FROM accounts WHERE email = ?";
        
        let count: (i64,) = sqlx::query_as(query)
            .bind(email.value())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        Ok(count.0 > 0)
    }

    async fn exists_by_id(&self, account_id: &AccountId) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT COUNT(*) as count FROM accounts WHERE id = ?";
        
        let count: (i64,) = sqlx::query_as(query)
            .bind(account_id.value())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        Ok(count.0 > 0)
    }
}

// Query Repository Implementation
#[async_trait]
impl AccountQueryRepositoryInterface for MySqlAccountRepository {
    async fn find_by_id(&self, id: &AccountId) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT * FROM accounts WHERE id = ?";
        
        let row = sqlx::query_as::<_, AccountSqlx>(query)
            .bind(id.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        if let Some(account_sqlx) = row {
            let account = account_sqlx.to_domain()?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT * FROM accounts WHERE email = ?";
        
        let row = sqlx::query_as::<_, AccountSqlx>(query)
            .bind(email.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        if let Some(account_sqlx) = row {
            let account = account_sqlx.to_domain()?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Delegate to command repository implementation
        AccountCommandRepositoryInterface::exists_by_email(self, email).await
    }

    async fn find_all(&self, pagination: PaginationParams) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT * FROM accounts ORDER BY created_at DESC LIMIT ? OFFSET ?";
        
        let offset = if pagination.page > 0 { (pagination.page - 1) * pagination.limit } else { 0 };
        
        let rows = sqlx::query_as::<_, AccountSqlx>(query)
            .bind(pagination.limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        let mut accounts = Vec::new();
        for account_sqlx in rows {
            accounts.push(account_sqlx.to_domain()?);
        }
        
        let total_count = self.count_total().await?;
        let total_pages = (total_count + pagination.limit as u64 - 1) / pagination.limit as u64;
        
        Ok(PaginatedResult {
            data: accounts,
            pagination: PaginationInfo {
                current_page: pagination.page,
                per_page: pagination.limit,
                total_count,
                total_pages: total_pages as u32,
                has_next: pagination.page < total_pages as u32,
                has_prev: pagination.page > 1,
            },
        })
    }

    async fn count_total(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let query = "SELECT COUNT(*) as count FROM accounts";
        
        let count: (i64,) = sqlx::query_as(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                })
            })?;
        
        Ok(count.0 as u64)
    }

    // TODO: Implement remaining query methods for Week 2
    async fn find_by_status(&self, _status: &AccountStatus, _pagination: PaginationParams) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn count_by_status(&self, _status: &AccountStatus) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn search_accounts(&self, _filters: AccountSearchFilters, _sort: SortParams, _pagination: PaginationParams) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn count_accounts_in_period(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn get_total_balance_by_currency(&self, _currency_code: &str) -> Result<Money, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn get_account_creation_trend(&self, _period: TimePeriod, _granularity: TimeGranularity) -> Result<Vec<TimeSeriesPoint>, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn find_accounts_with_balance_above(&self, _threshold: &Money, _pagination: PaginationParams) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
    
    async fn find_accounts_with_balance_below(&self, _threshold: &Money, _pagination: PaginationParams) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implement in Week 2")
    }
}