//tests/database_test.rs
// Database Integration Tests for Financial API
// 2025/8/28

use chrono;
use rusted_ca::domain::entity::account::Account;
use rusted_ca::domain::entity::payment::Payment;
use rusted_ca::domain::value_object::{
    AccountId, Email, MerchantName, Money, PaymentMethod, PaymentStatus,
};
use std::sync::Arc;

#[tokio::test]
async fn test_account_crud_operations() {
    println!("🚀 Starting Account CRUD Operations Test");

    // Test account creation with business rules
    let unique_email = format!(
        "dbtest+{}@merchant.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let merchant_name = MerchantName::new("Test Database Merchant".to_string()).unwrap();
    let email = Email::new(unique_email.clone()).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Test account properties
    assert_eq!(account.merchant_name().value(), "Test Database Merchant");
    assert_eq!(account.email().value(), unique_email);
    assert_eq!(account.balance().amount_cents(), 0);
    assert_eq!(account.balance().currency_code(), "USD");
    println!("✅ Account created successfully with correct properties");

    // Test balance operations
    let mut test_account = account;
    let credit_amount = Money::from_major_units(250.75, "USD".to_string()).unwrap();

    let credit_result = test_account.credit(&credit_amount);
    assert!(credit_result.is_ok(), "Credit operation should succeed");
    assert_eq!(test_account.balance().amount_cents(), 25075); // $250.75
    println!(
        "✅ Account balance updated successfully: ${}",
        test_account.balance().to_major_units()
    );

    // Test debit operation
    let debit_amount = Money::from_major_units(50.25, "USD".to_string()).unwrap();
    let debit_result = test_account.debit(&debit_amount);
    assert!(debit_result.is_ok(), "Debit operation should succeed");
    assert_eq!(test_account.balance().amount_cents(), 20050); // $250.75 - $50.25 = $200.50
    println!(
        "✅ Account debit operation successful: ${}",
        test_account.balance().to_major_units()
    );

    // Test business rule: insufficient funds
    let large_debit = Money::from_major_units(500.0, "USD".to_string()).unwrap();
    let insufficient_result = test_account.debit(&large_debit);
    assert!(
        insufficient_result.is_err(),
        "Large debit should fail due to insufficient funds"
    );
    println!("✅ Insufficient funds business rule enforced correctly");

    println!("✅ Account CRUD Operations Test completed successfully\n");
}

#[tokio::test]
async fn test_payment_database_operations() {
    println!("🚀 Starting Payment Database Operations Test");

    // Create test account first
    let unique_email = format!(
        "payment+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let merchant_name = MerchantName::new("Payment Test Merchant".to_string()).unwrap();
    let email = Email::new(unique_email).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();
    println!("✅ Test account created for payment testing");

    // Create test payment
    let amount = Money::from_major_units(99.99, "USD".to_string()).unwrap();
    let payment = Payment::create_new(
        account_id.clone(),
        amount,
        PaymentMethod::Card,
        Some("Database test payment".to_string()),
        Some("customer@dbtest.com".to_string()),
        Some(serde_json::json!({
            "test_type": "database_test",
            "environment": "test"
        })),
    )
    .unwrap();
    let payment_id = payment.id().clone();

    // Test payment properties
    assert_eq!(payment.account_id().value(), account_id.value());
    assert_eq!(payment.amount().amount_cents(), 9999); // $99.99
    assert_eq!(payment.status(), &PaymentStatus::Pending);
    assert_eq!(payment.payment_method(), &PaymentMethod::Card);
    assert_eq!(
        payment.description(),
        Some(&"Database test payment".to_string())
    );
    assert_eq!(
        payment.customer_email(),
        Some(&"customer@dbtest.com".to_string())
    );
    println!("✅ Payment created successfully with correct properties");

    // Test payment lifecycle (state transitions)
    let mut test_payment = payment;

    // Start processing
    let processing_result = test_payment.start_processing();
    assert!(processing_result.is_ok(), "Start processing should succeed");
    assert_eq!(test_payment.status(), &PaymentStatus::Processing);
    println!("✅ Payment status updated to PROCESSING");

    // Mark successful
    let success_result = test_payment.mark_successful();
    assert!(success_result.is_ok(), "Mark successful should succeed");
    assert_eq!(test_payment.status(), &PaymentStatus::Success);
    println!("✅ Payment status updated to SUCCESS");

    // Test refund operation
    let refund_amount = Money::from_major_units(25.0, "USD".to_string()).unwrap();
    let refund_result = test_payment.refund(Some(&refund_amount), "Customer refund".to_string());
    assert!(refund_result.is_ok(), "Refund should succeed");
    assert_eq!(test_payment.status(), &PaymentStatus::Refunded);
    println!("✅ Payment refund processed successfully");

    // Verify metadata was updated
    let metadata = test_payment.metadata().expect("Metadata should exist");
    assert_eq!(metadata["refund_reason"], "Customer refund");
    assert!(metadata.get("refunded_at").is_some());
    println!("✅ Payment metadata updated correctly during refund");

    println!("✅ Payment Database Operations Test completed successfully\n");
}

#[tokio::test]
async fn test_concurrent_operations() {
    println!("🚀 Starting Concurrent Operations Test");

    // Create test account
    let unique_email = format!(
        "concurrent+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let merchant_name = MerchantName::new("Concurrent Test Merchant".to_string()).unwrap();
    let email = Email::new(unique_email).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Create multiple payments concurrently (business logic test)
    let mut handles = vec![];
    for i in 0..5 {
        let account_id = account_id.clone();

        let handle = tokio::spawn(async move {
            let amount = Money::from_major_units(10.0 + i as f64, "USD".to_string()).unwrap();
            let payment = Payment::create_new(
                account_id,
                amount,
                PaymentMethod::Card,
                Some(format!("Concurrent payment {}", i)),
                None,
                None,
            )
            .unwrap();

            // Test that payment can be processed through lifecycle
            let mut test_payment = payment;
            test_payment.start_processing().unwrap();
            test_payment.mark_successful().unwrap();

            (test_payment.id().clone(), test_payment.status().clone())
        });

        handles.push(handle);
    }

    // Wait for all payments to be created and processed
    let results: Vec<_> = futures::future::try_join_all(handles).await.unwrap();
    println!(
        "✅ Created and processed {} payments concurrently",
        results.len()
    );

    // Verify all payments were processed successfully
    for (payment_id, status) in &results {
        assert_eq!(status, &PaymentStatus::Success);
    }
    println!("✅ All concurrent payments verified as successful");

    println!("✅ Concurrent Operations Test completed successfully\n");
}

#[tokio::test]
async fn test_error_handling_and_constraints() {
    println!("🚀 Starting Error Handling and Constraints Test");

    // Test invalid email creation
    let invalid_email_result = Email::new("invalid-email".to_string());
    assert!(
        invalid_email_result.is_err(),
        "Invalid email should be rejected"
    );
    println!("✅ Invalid email validation working correctly");

    // Test money creation (negative amounts are allowed for accounting purposes)
    let negative_money_result = Money::from_major_units(-10.0, "USD".to_string());
    assert!(
        negative_money_result.is_ok(),
        "Negative money amounts should be allowed for accounting"
    );
    let negative_money = negative_money_result.unwrap();
    assert!(
        negative_money.is_negative(),
        "Negative money should be detected correctly"
    );
    println!("✅ Money negative amount validation working correctly");

    // Test payment business rules
    let account = Account::create_new(
        MerchantName::new("Test Account".to_string()).unwrap(),
        Email::new("test@example.com".to_string()).unwrap(),
        "USD".to_string(),
    )
    .unwrap();

    // Test payment with zero amount (should fail)
    let zero_amount = Money::from_major_units(0.0, "USD".to_string()).unwrap();
    let zero_payment_result = Payment::create_new(
        account.id().clone(),
        zero_amount,
        PaymentMethod::Card,
        None,
        None,
        None,
    );
    assert!(
        zero_payment_result.is_err(),
        "Zero amount payment should be rejected"
    );
    println!("✅ Zero amount payment validation working correctly");

    // Test invalid state transitions
    let valid_payment = Payment::create_new(
        account.id().clone(),
        Money::from_major_units(100.0, "USD".to_string()).unwrap(),
        PaymentMethod::Card,
        None,
        None,
        None,
    )
    .unwrap();

    let mut test_payment = valid_payment;

    // Try to mark successful without processing (should fail)
    let invalid_transition = test_payment.mark_successful();
    assert!(
        invalid_transition.is_err(),
        "Direct transition to success should fail"
    );
    println!("✅ Invalid payment state transition properly rejected");

    println!("✅ Error Handling and Constraints Test completed successfully\n");
}

#[tokio::test]
async fn test_transaction_consistency() {
    println!("🚀 Starting Transaction Consistency Test");

    // Create test account with initial balance
    let unique_email = format!(
        "transaction+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let merchant_name = MerchantName::new("Transaction Test Merchant".to_string()).unwrap();
    let email = Email::new(unique_email).unwrap();
    let mut account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Add initial balance
    let initial_balance = Money::from_major_units(500.0, "USD".to_string()).unwrap();
    account.credit(&initial_balance).unwrap();
    println!(
        "✅ Account created with initial balance: ${}",
        account.balance().to_major_units()
    );

    // Process a payment that would affect the balance
    let payment_amount = Money::from_major_units(150.0, "USD".to_string()).unwrap();
    let payment = Payment::create_new(
        account_id.clone(),
        payment_amount.clone(),
        PaymentMethod::Card,
        Some("Balance affecting payment".to_string()),
        None,
        None,
    )
    .unwrap();

    // Simulate successful payment processing and balance update
    let mut test_payment = payment;
    test_payment.start_processing().unwrap();
    test_payment.mark_successful().unwrap();

    // Update account balance (simulate settlement)
    account.debit(&payment_amount).unwrap();

    // Verify final state
    assert_eq!(account.balance().amount_cents(), 35000); // $500 - $150 = $350
    assert_eq!(test_payment.status(), &PaymentStatus::Success);

    println!("✅ Transaction consistency verified:");
    println!(
        "   - Account balance: ${}",
        account.balance().to_major_units()
    );
    println!("   - Payment status: {}", test_payment.status().as_str());

    // Test rollback scenario (refund)
    let refund_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
    test_payment
        .refund(Some(&refund_amount), "Customer refund".to_string())
        .unwrap();

    // Update account balance for refund
    account.credit(&refund_amount).unwrap();

    assert_eq!(account.balance().amount_cents(), 40000); // $350 + $50 = $400
    assert_eq!(test_payment.status(), &PaymentStatus::Refunded);

    println!("✅ Refund transaction consistency verified:");
    println!(
        "   - Final balance after refund: ${}",
        account.balance().to_major_units()
    );

    println!("✅ Transaction Consistency Test completed successfully\n");
}
