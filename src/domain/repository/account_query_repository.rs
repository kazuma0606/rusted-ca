//domain/repository/account_query_repository.rs
// Account Query Repository トレイト
// 2025/8/28

use crate::domain::entity::account::Account;
use crate::domain::value_object::{AccountId, AccountStatus, Email, Money, pagination::*};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait AccountQueryRepositoryInterface: Send + Sync {
    // 基本検索
    async fn find_by_id(
        &self,
        id: &AccountId,
    ) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>>;
    async fn exists_by_email(
        &self,
        email: &Email,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    // 一覧・ページング
    async fn find_all(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>>;
    async fn count_total(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;

    // ステータス別検索
    async fn find_by_status(
        &self,
        status: &AccountStatus,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>>;
    async fn count_by_status(
        &self,
        status: &AccountStatus,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;

    // 高度な検索
    async fn search_accounts(
        &self,
        filters: AccountSearchFilters,
        sort: SortParams,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>>;

    // Analytics用
    async fn count_accounts_in_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_total_balance_by_currency(
        &self,
        currency_code: &str,
    ) -> Result<Money, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_account_creation_trend(
        &self,
        period: TimePeriod,
        granularity: TimeGranularity,
    ) -> Result<Vec<TimeSeriesPoint>, Box<dyn std::error::Error + Send + Sync>>;

    // 残高関連クエリ
    async fn find_accounts_with_balance_above(
        &self,
        threshold: &Money,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_accounts_with_balance_below(
        &self,
        threshold: &Money,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Account>, Box<dyn std::error::Error + Send + Sync>>;
}

// Account用検索フィルター
#[derive(Debug, Clone)]
pub struct AccountSearchFilters {
    pub merchant_name: Option<String>,
    pub email: Option<String>,
    pub status: Option<AccountStatus>,
    pub currency_code: Option<String>,
    pub balance_min: Option<Money>,
    pub balance_max: Option<Money>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
}

impl Default for AccountSearchFilters {
    fn default() -> Self {
        Self {
            merchant_name: None,
            email: None,
            status: None,
            currency_code: None,
            balance_min: None,
            balance_max: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        }
    }
}