use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, MySql, Type,
    decode::Decode,
    encode::Encode,
    mysql::{MySqlTypeInfo, MySqlValueRef},
};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatusSqlx {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "SUSPENDED")]
    Suspended,
    #[serde(rename = "CLOSED")]
    Closed,
}

impl fmt::Display for AccountStatusSqlx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AccountStatusSqlx::Active => "ACTIVE",
            AccountStatusSqlx::Suspended => "SUSPENDED",
            AccountStatusSqlx::Closed => "CLOSED",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccountStatusSqlx {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Ok(AccountStatusSqlx::Active),
            "SUSPENDED" => Ok(AccountStatusSqlx::Suspended),
            "CLOSED" => Ok(AccountStatusSqlx::Closed),
            _ => Err(()),
        }
    }
}

// Custom sqlx implementations for MySQL ENUM
impl Type<MySql> for AccountStatusSqlx {
    fn type_info() -> MySqlTypeInfo {
        // Treat ENUM as VARCHAR for compatibility
        <str as Type<MySql>>::type_info()
    }
}

impl Encode<'_, MySql> for AccountStatusSqlx {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        // For MySQL ENUM, we need to pass the actual string values
        // The enum values are defined in the database schema: ENUM('ACTIVE', 'SUSPENDED', 'CLOSED')
        let enum_value = match self {
            AccountStatusSqlx::Active => "ACTIVE",
            AccountStatusSqlx::Suspended => "SUSPENDED",
            AccountStatusSqlx::Closed => "CLOSED",
        };
        <&str as Encode<MySql>>::encode_by_ref(&enum_value, buf)
    }
}

impl Decode<'_, MySql> for AccountStatusSqlx {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        // Try to decode as string first (for ENUM values)
        if let Ok(s) = <&str as Decode<MySql>>::decode(value.clone()) {
            return Self::from_str(s).map_err(|_| format!("Invalid account status: {}", s).into());
        }

        // Fallback: try to decode as bytes and convert to string
        let bytes = <&[u8] as Decode<MySql>>::decode(value)?;
        let s = std::str::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        Self::from_str(s).map_err(|_| format!("Invalid account status: {}", s).into())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AccountSqlx {
    pub id: String,                                // VARCHAR(36) PRIMARY KEY
    pub merchant_name: String,                     // VARCHAR(255) NOT NULL
    pub email: String,                             // VARCHAR(255) NOT NULL UNIQUE
    pub account_status: AccountStatusSqlx,         // ENUM('ACTIVE', 'SUSPENDED', 'CLOSED')
    pub balance_cents: i64,                        // BIGINT NOT NULL DEFAULT 0
    pub currency_code: String,                     // CHAR(3) NOT NULL DEFAULT 'JPY'
    pub created_at: chrono::DateTime<chrono::Utc>, // TIMESTAMP
    pub updated_at: chrono::DateTime<chrono::Utc>, // TIMESTAMP
}

// Domain Entity との変換処理
impl AccountSqlx {
    /// Domain Account から AccountSqlx へ変換
    pub fn from_domain(account: &crate::domain::entity::account::Account) -> Self {
        Self {
            id: account.id().value().to_string(),
            merchant_name: account.merchant_name().value().to_string(),
            email: account.email().value().to_string(),
            account_status: match account.status() {
                crate::domain::value_object::AccountStatus::Active => AccountStatusSqlx::Active,
                crate::domain::value_object::AccountStatus::Suspended => {
                    AccountStatusSqlx::Suspended
                }
                crate::domain::value_object::AccountStatus::Closed => AccountStatusSqlx::Closed,
            },
            balance_cents: account.balance().amount_cents(),
            currency_code: account.balance().currency_code().to_string(),
            created_at: *account.created_at(),
            updated_at: *account.updated_at(),
        }
    }

    /// AccountSqlx から Domain Account へ変換
    pub fn to_domain(
        &self,
    ) -> Result<crate::domain::entity::account::Account, Box<dyn std::error::Error + Send + Sync>>
    {
        use crate::domain::entity::account::Account;
        use crate::domain::value_object::*;
        use chrono::{DateTime, Utc};

        let account_id = AccountId::new(self.id.clone())?;
        let merchant_name = MerchantName::new(self.merchant_name.clone())?;
        let email = Email::new(self.email.clone())?;
        let status = match self.account_status {
            AccountStatusSqlx::Active => AccountStatus::Active,
            AccountStatusSqlx::Suspended => AccountStatus::Suspended,
            AccountStatusSqlx::Closed => AccountStatus::Closed,
        };
        let balance = Money::new(self.balance_cents, self.currency_code.clone())?;
        let created_at = self.created_at;
        let updated_at = self.updated_at;

        Account::new(
            account_id,
            merchant_name,
            email,
            status,
            balance,
            created_at,
            updated_at,
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::account::Account;
    use crate::domain::value_object::{AccountId, AccountStatus, Email, MerchantName, Money};
    use chrono::Utc;

    #[test]
    fn test_account_sqlx_status_display() {
        assert_eq!(AccountStatusSqlx::Active.to_string(), "ACTIVE");
        assert_eq!(AccountStatusSqlx::Suspended.to_string(), "SUSPENDED");
        assert_eq!(AccountStatusSqlx::Closed.to_string(), "CLOSED");
    }

    #[test]
    fn test_account_sqlx_status_from_str() {
        assert_eq!(
            "ACTIVE".parse::<AccountStatusSqlx>().unwrap(),
            AccountStatusSqlx::Active
        );
        assert_eq!(
            "active".parse::<AccountStatusSqlx>().unwrap(),
            AccountStatusSqlx::Active
        );
        assert_eq!(
            "SUSPENDED".parse::<AccountStatusSqlx>().unwrap(),
            AccountStatusSqlx::Suspended
        );
        assert_eq!(
            "CLOSED".parse::<AccountStatusSqlx>().unwrap(),
            AccountStatusSqlx::Closed
        );
    }

    #[test]
    fn test_domain_conversion() {
        // Create a domain account
        let merchant_name = MerchantName::new("Test Merchant".to_string()).unwrap();
        let email = Email::new("test@merchant.com".to_string()).unwrap();
        let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();

        // Convert to SQLx
        let account_sqlx = AccountSqlx::from_domain(&account);
        assert_eq!(account_sqlx.merchant_name, "Test Merchant");
        assert_eq!(account_sqlx.email, "test@merchant.com");
        assert_eq!(account_sqlx.currency_code, "USD");
        assert_eq!(account_sqlx.balance_cents, 0);

        // Convert back to domain
        let domain_account = account_sqlx.to_domain().unwrap();
        assert_eq!(domain_account.merchant_name().value(), "Test Merchant");
        assert_eq!(domain_account.email().value(), "test@merchant.com");
        assert_eq!(domain_account.balance().currency_code(), "USD");
        assert!(domain_account.balance().is_zero());
    }
}
