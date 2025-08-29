//tests/integration_test.rs
// 金融系API統合テスト
// 2025/8/28

use rusted_ca::domain::entity::account::Account;
use rusted_ca::domain::entity::payment::Payment;
use rusted_ca::domain::value_object::{
    AccountId, AccountStatus, Email, MerchantName, Money, PaymentMethod, PaymentStatus,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_account_creation_and_retrieval() {
    // Test Account entity creation
    let merchant_name = MerchantName::new("Test Merchant Co.".to_string()).unwrap();
    let email = Email::new("test@merchant.com".to_string()).unwrap();

    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();

    // Verify account properties
    assert_eq!(account.merchant_name().value(), "Test Merchant Co.");
    assert_eq!(account.email().value(), "test@merchant.com");
    assert_eq!(account.status(), &AccountStatus::Active);
    assert!(account.balance().is_zero());
    assert_eq!(account.balance().currency_code(), "USD");

    println!("✅ Account creation test passed");
}

#[tokio::test]
async fn test_payment_lifecycle() {
    // Create test account
    let account_id = AccountId::generate();
    let amount = Money::from_major_units(150.75, "USD".to_string()).unwrap();

    // Create payment
    let payment = Payment::create_new(
        account_id,
        amount,
        PaymentMethod::Card,
        Some("Integration test payment".to_string()),
        Some("customer@example.com".to_string()),
        Some(json!({
            "test": true,
            "source": "integration_test"
        })),
    )
    .unwrap();

    // Verify initial state
    assert_eq!(payment.status(), &PaymentStatus::Pending);
    assert_eq!(payment.amount().amount_cents(), 15075); // $150.75 = 15075 cents
    assert_eq!(payment.payment_method(), &PaymentMethod::Card);
    assert_eq!(
        payment.description(),
        Some(&"Integration test payment".to_string())
    );
    assert_eq!(
        payment.customer_email(),
        Some(&"customer@example.com".to_string())
    );

    // Test payment processing
    let mut payment_processing = payment.clone();
    payment_processing.start_processing().unwrap();
    assert_eq!(payment_processing.status(), &PaymentStatus::Processing);

    // Test successful completion
    let mut payment_success = payment_processing.clone();
    payment_success.mark_successful().unwrap();
    assert_eq!(payment_success.status(), &PaymentStatus::Success);
    assert!(payment_success.is_successful());
    assert!(payment_success.can_be_refunded());

    // Test refund
    let mut payment_refunded = payment_success.clone();
    payment_refunded
        .refund(None, "Customer requested refund".to_string())
        .unwrap();
    assert_eq!(payment_refunded.status(), &PaymentStatus::Refunded);
    assert!(!payment_refunded.can_be_refunded());

    println!("✅ Payment lifecycle test passed");
}

#[tokio::test]
async fn test_account_balance_operations() {
    // Create account
    let merchant_name = MerchantName::new("Balance Test Merchant".to_string()).unwrap();
    let email = Email::new("balance@test.com".to_string()).unwrap();
    let mut account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();

    // Test credit operation
    let credit_amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
    account.credit(&credit_amount).unwrap();
    assert_eq!(account.balance().amount_cents(), 10000); // $100 = 10000 cents

    // Test debit operation
    let debit_amount = Money::from_major_units(30.50, "USD".to_string()).unwrap();
    account.debit(&debit_amount).unwrap();
    assert_eq!(account.balance().amount_cents(), 6950); // $69.50 = 6950 cents

    // Test overdraft (current implementation allows negative balance)
    let large_debit = Money::from_major_units(100.0, "USD".to_string()).unwrap();
    let result = account.debit(&large_debit);
    assert!(result.is_ok()); // Current implementation allows negative balance
    assert_eq!(account.balance().amount_cents(), -3050); // -$30.50 = -3050 cents

    println!("✅ Account balance operations test passed");
}

#[tokio::test]
async fn test_currency_validation() {
    // Test valid currencies
    let valid_currencies = ["USD", "EUR", "JPY"];
    for currency in valid_currencies {
        let merchant_name = MerchantName::new("Currency Test".to_string()).unwrap();
        let email = Email::new("currency@test.com".to_string()).unwrap();
        let result = Account::create_new(merchant_name, email, currency.to_string());
        assert!(result.is_ok(), "Currency {} should be valid", currency);
    }

    // Test invalid currency
    let merchant_name = MerchantName::new("Invalid Currency Test".to_string()).unwrap();
    let email = Email::new("invalid@test.com".to_string()).unwrap();
    let result = Account::create_new(merchant_name, email, "INVALID".to_string());
    assert!(result.is_err(), "Invalid currency should be rejected");

    println!("✅ Currency validation test passed");
}

#[tokio::test]
async fn test_payment_business_rules() {
    let account_id = AccountId::generate();

    // Test payment with zero amount (should fail)
    let zero_amount = Money::new(0, "USD".to_string()).unwrap();
    let result = Payment::create_new(
        account_id.clone(),
        zero_amount,
        PaymentMethod::Card,
        None,
        None,
        None,
    );
    assert!(result.is_err(), "Zero amount payment should be rejected");

    // Test payment with negative amount (should fail)
    let negative_amount = Money::new(-1000, "USD".to_string()).unwrap();
    let result = Payment::create_new(
        account_id.clone(),
        negative_amount,
        PaymentMethod::Card,
        None,
        None,
        None,
    );
    assert!(
        result.is_err(),
        "Negative amount payment should be rejected"
    );

    // Test valid payment
    let valid_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
    let result = Payment::create_new(
        account_id,
        valid_amount,
        PaymentMethod::Card,
        Some("Valid payment".to_string()),
        Some("valid@example.com".to_string()),
        None,
    );
    assert!(result.is_ok(), "Valid payment should be accepted");

    println!("✅ Payment business rules test passed");
}

#[tokio::test]
async fn test_account_status_transitions() {
    let merchant_name = MerchantName::new("Status Test Merchant".to_string()).unwrap();
    let email = Email::new("status@test.com".to_string()).unwrap();
    let mut account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();

    // Test suspension
    account.update_status(AccountStatus::Suspended).unwrap();
    assert_eq!(account.status(), &AccountStatus::Suspended);

    // Test reactivation
    account.update_status(AccountStatus::Active).unwrap();
    assert_eq!(account.status(), &AccountStatus::Active);

    // Test closing (should fail with balance)
    let credit = Money::from_major_units(10.0, "USD".to_string()).unwrap();
    account.credit(&credit).unwrap();
    let close_result = account.close();
    assert!(close_result.is_err(), "Cannot close account with balance");

    // Clear balance and close
    let debit = Money::from_major_units(10.0, "USD".to_string()).unwrap();
    account.debit(&debit).unwrap();
    account.close().unwrap();
    assert_eq!(account.status(), &AccountStatus::Closed);

    // Test reactivation of closed account (should fail)
    let reactivate_result = account.update_status(AccountStatus::Active);
    assert!(
        reactivate_result.is_err(),
        "Cannot reactivate closed account"
    );

    println!("✅ Account status transitions test passed");
}

// Placeholder for database integration tests
// These would require actual database setup and would be more complex

#[tokio::test]
#[ignore] // Use 'cargo test -- --ignored' to run database tests
async fn test_mysql_integration() {
    // This would test actual MySQL database operations
    // Requires database setup and connection
    println!("⚠️  MySQL integration test - requires database setup");

    // Example structure:
    // 1. Setup test database
    // 2. Create account via repository
    // 3. Retrieve account and verify data
    // 4. Create payment and link to account
    // 5. Process payment through various states
    // 6. Verify all data persisted correctly
    // 7. Cleanup test data

    // TODO: Implement when database integration is ready
}

#[tokio::test]
#[ignore] // Database integration test
async fn test_end_to_end_payment_flow() {
    // This would test the complete payment flow with database persistence
    println!("⚠️  End-to-end payment flow test - requires full system setup");

    // Example flow:
    // 1. Create merchant account
    // 2. Create payment request
    // 3. Process payment (simulate external gateway)
    // 4. Verify account balance updated
    // 5. Test refund flow
    // 6. Verify final balances and payment states

    // TODO: Implement when full system is integrated
}

// Helper function for test data cleanup
async fn cleanup_test_data() {
    // In a real implementation, this would clean up test data from database
    println!("🧹 Test cleanup completed");
}
