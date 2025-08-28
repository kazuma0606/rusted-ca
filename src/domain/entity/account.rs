//domain/entity/account.rs
// Account エンティティ + ビジネスロジック
// 2025/8/28

use crate::domain::value_object::{
    AccountId, AccountStatus, Email, MerchantName, Money,
};
use crate::shared::error::domain_error::{DomainError, DomainResult};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub id: AccountId,
    pub merchant_name: MerchantName,
    pub email: Email,
    pub status: AccountStatus,
    pub balance: Money,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(
        id: AccountId,
        merchant_name: MerchantName,
        email: Email,
        status: AccountStatus,
        balance: Money,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        // Business rule: Account must have valid currency
        let valid_currencies = ["JPY", "USD", "EUR"];
        if !valid_currencies.contains(&balance.currency_code()) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "ValidCurrency".to_string(),
                message: format!(
                    "Account currency must be one of: {}. Got: {}",
                    valid_currencies.join(", "),
                    balance.currency_code()
                ),
            });
        }

        Ok(Account {
            id,
            merchant_name,
            email,
            status,
            balance,
            created_at,
            updated_at,
        })
    }

    /// Create a new account with default values
    pub fn create_new(
        merchant_name: MerchantName,
        email: Email,
        currency_code: String,
    ) -> DomainResult<Self> {
        let id = AccountId::generate();
        let status = AccountStatus::default(); // Active
        let balance = Money::zero(currency_code)?;
        let now = Utc::now();

        Self::new(id, merchant_name, email, status, balance, now, now)
    }

    // Getters
    pub fn id(&self) -> &AccountId {
        &self.id
    }

    pub fn merchant_name(&self) -> &MerchantName {
        &self.merchant_name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn status(&self) -> &AccountStatus {
        &self.status
    }

    pub fn balance(&self) -> &Money {
        &self.balance
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    // Business operations
    /// Update account status (with business rules)
    pub fn update_status(&mut self, new_status: AccountStatus) -> DomainResult<()> {
        // Business rule: Cannot reactivate a closed account
        if self.status.is_closed() && new_status.is_active() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoReactivateClosedAccount".to_string(),
                message: "Cannot reactivate a closed account".to_string(),
            });
        }

        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Credit money to account
    pub fn credit(&mut self, amount: &Money) -> DomainResult<()> {
        // Business rule: Cannot modify closed account
        if self.status.is_closed() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoModifyClosedAccount".to_string(),
                message: "Cannot modify balance of a closed account".to_string(),
            });
        }

        // Business rule: Credit amount must be positive
        if !amount.is_positive() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "PositiveCreditAmount".to_string(),
                message: "Credit amount must be positive".to_string(),
            });
        }

        self.balance = self.balance.add(amount)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Debit money from account
    pub fn debit(&mut self, amount: &Money) -> DomainResult<()> {
        // Business rule: Cannot modify closed account
        if self.status.is_closed() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoModifyClosedAccount".to_string(),
                message: "Cannot modify balance of a closed account".to_string(),
            });
        }

        // Business rule: Debit amount must be positive
        if !amount.is_positive() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "PositiveDebitAmount".to_string(),
                message: "Debit amount must be positive".to_string(),
            });
        }

        // Business rule: Cannot debit from suspended account
        if self.status.is_suspended() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoDebitSuspendedAccount".to_string(),
                message: "Cannot debit from a suspended account".to_string(),
            });
        }

        self.balance = self.balance.subtract(amount)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if account can perform transactions
    pub fn can_transact(&self) -> bool {
        self.status.is_active()
    }

    /// Update merchant details
    pub fn update_merchant_info(&mut self, merchant_name: MerchantName) -> DomainResult<()> {
        // Business rule: Cannot modify closed account
        if self.status.is_closed() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoModifyClosedAccount".to_string(),
                message: "Cannot modify a closed account".to_string(),
            });
        }

        self.merchant_name = merchant_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Close account
    pub fn close(&mut self) -> DomainResult<()> {
        // Business rule: Cannot close account with positive balance
        if self.balance.is_positive() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoCloseWithBalance".to_string(),
                message: "Cannot close account with positive balance".to_string(),
            });
        }

        self.status = AccountStatus::Closed;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_account() -> Account {
        let merchant_name = MerchantName::new("Test Merchant".to_string()).unwrap();
        let email = Email::new("test@merchant.com".to_string()).unwrap();
        Account::create_new(merchant_name, email, "USD".to_string()).unwrap()
    }

    #[test]
    fn test_account_creation() {
        let account = create_test_account();
        assert_eq!(account.merchant_name().value(), "Test Merchant");
        assert_eq!(account.email().value(), "test@merchant.com");
        assert!(account.status().is_active());
        assert!(account.balance().is_zero());
        assert_eq!(account.balance().currency_code(), "USD");
    }

    #[test]
    fn test_account_creation_invalid_currency() {
        let merchant_name = MerchantName::new("Test Merchant".to_string()).unwrap();
        let email = Email::new("test@merchant.com".to_string()).unwrap();
        let result = Account::create_new(merchant_name, email, "XXX".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_account_credit() {
        let mut account = create_test_account();
        let credit_amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        
        assert!(account.credit(&credit_amount).is_ok());
        assert_eq!(account.balance().amount_cents(), 10000); // $100 = 10000 cents
    }

    #[test]
    fn test_account_debit() {
        let mut account = create_test_account();
        let credit_amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let debit_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
        
        account.credit(&credit_amount).unwrap();
        assert!(account.debit(&debit_amount).is_ok());
        assert_eq!(account.balance().amount_cents(), 5000); // $50 = 5000 cents
    }

    #[test]
    fn test_account_cannot_debit_suspended() {
        let mut account = create_test_account();
        let credit_amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let debit_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
        
        account.credit(&credit_amount).unwrap();
        account.update_status(AccountStatus::Suspended).unwrap();
        
        assert!(account.debit(&debit_amount).is_err());
    }

    #[test]
    fn test_account_cannot_modify_closed() {
        let mut account = create_test_account();
        account.close().unwrap();
        
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        assert!(account.credit(&amount).is_err());
        assert!(account.debit(&amount).is_err());
    }

    #[test]
    fn test_account_cannot_close_with_balance() {
        let mut account = create_test_account();
        let credit_amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        
        account.credit(&credit_amount).unwrap();
        assert!(account.close().is_err());
    }

    #[test]
    fn test_account_cannot_reactivate_closed() {
        let mut account = create_test_account();
        account.close().unwrap();
        
        assert!(account.update_status(AccountStatus::Active).is_err());
    }

    #[test]
    fn test_account_currency_mismatch() {
        let mut account = create_test_account();
        let wrong_currency_amount = Money::from_major_units(100.0, "JPY".to_string()).unwrap();
        
        assert!(account.credit(&wrong_currency_amount).is_err());
    }
}