//tests/database_test.rs
// Database Integration Tests for Financial API
// 2025/8/28

use rusted_ca::domain::entity::account::Account;
use rusted_ca::domain::entity::payment::Payment;
use rusted_ca::domain::repository::account_command_repository::AccountCommandRepositoryInterface;
use rusted_ca::domain::repository::account_query_repository::AccountQueryRepositoryInterface;
use rusted_ca::domain::repository::payment_command_repository::PaymentCommandRepository;
use rusted_ca::domain::repository::payment_query_repository::PaymentQueryRepository;
use rusted_ca::domain::value_object::{
    AccountId, Email, MerchantName, Money, PaymentMethod, PaymentStatus,
};
use rusted_ca::infrastructure::repository::mysql_account_repository::MySqlAccountRepository;
use rusted_ca::infrastructure::repository::mysql_payment_repository::MySqlPaymentRepository;
use sqlx::{MySql, Pool};
use std::sync::Arc;

// Helper function to setup test database connection
async fn setup_test_db() -> Result<Pool<MySql>, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:root@localhost:3306/rusted_ca".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url).await?;

    // Run migrations to ensure tables exist
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

// Helper function to clean up test data
async fn cleanup_test_account(pool: &Pool<MySql>, account_id: &str) {
    let _ = sqlx::query("DELETE FROM payments WHERE account_id = ?")
        .bind(account_id)
        .execute(pool)
        .await;

    let _ = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await;
}

#[tokio::test]
#[ignore] // Run with: cargo test database_test -- --ignored
async fn test_account_crud_operations() {
    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let repository = MySqlAccountRepository::new(pool.clone());

    // Create test account
    let merchant_name = MerchantName::new("Test Database Merchant".to_string()).unwrap();
    let email = Email::new("dbtest@merchant.com".to_string()).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Test CREATE
    let create_result = repository.save(&account).await;
    assert!(
        create_result.is_ok(),
        "Failed to create account: {:?}",
        create_result
    );
    println!("✅ Account created successfully");

    // Test READ
    let retrieved_account = repository.find_by_id(&account_id).await.unwrap();
    assert!(
        retrieved_account.is_some(),
        "Account not found after creation"
    );
    let retrieved = retrieved_account.unwrap();

    assert_eq!(retrieved.id(), account.id());
    assert_eq!(
        retrieved.merchant_name().value(),
        account.merchant_name().value()
    );
    assert_eq!(retrieved.email().value(), account.email().value());
    assert_eq!(retrieved.balance().amount_cents(), 0);
    assert_eq!(retrieved.balance().currency_code(), "USD");
    println!("✅ Account retrieved successfully with correct data");

    // Test UPDATE (credit operation)
    let mut updated_account = retrieved;
    let credit_amount = Money::from_major_units(250.75, "USD".to_string()).unwrap();
    updated_account.credit(&credit_amount).unwrap();

    let update_result = repository.update(&updated_account).await;
    assert!(
        update_result.is_ok(),
        "Failed to update account: {:?}",
        update_result
    );

    // Verify update
    let updated_retrieved = repository.find_by_id(&account_id).await.unwrap().unwrap();
    assert_eq!(updated_retrieved.balance().amount_cents(), 25075); // $250.75
    println!("✅ Account balance updated successfully");

    // Test business rule violation (attempt to create duplicate email)
    let duplicate_account = Account::create_new(
        MerchantName::new("Duplicate Test".to_string()).unwrap(),
        Email::new("dbtest@merchant.com".to_string()).unwrap(), // Same email
        "USD".to_string(),
    )
    .unwrap();

    let duplicate_result = repository.save(&duplicate_account).await;
    assert!(
        duplicate_result.is_err(),
        "Should fail to create duplicate email account"
    );
    println!("✅ Duplicate email constraint enforced");

    // Cleanup
    cleanup_test_account(&pool, account_id.value()).await;
    println!("✅ Database test cleanup completed");
}

#[tokio::test]
#[ignore] // Run with: cargo test database_test -- --ignored  
async fn test_payment_database_operations() {
    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Create test account first
    let merchant_name = MerchantName::new("Payment Test Merchant".to_string()).unwrap();
    let email = Email::new("payment@test.com".to_string()).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    account_repo.save(&account).await.unwrap();
    println!("✅ Test account created for payment testing");

    // Create test payment
    let amount = Money::from_major_units(99.99, "USD".to_string()).unwrap();
    let payment = Payment::create_new(
        account_id.clone(),
        amount,
        PaymentMethod::Card,
        Some("Database integration test payment".to_string()),
        Some("customer@example.com".to_string()),
        Some(serde_json::json!({
            "test": true,
            "source": "database_integration_test"
        })),
    )
    .unwrap();
    let payment_id = payment.id().clone();

    // Test payment CREATE
    let create_result = payment_repo.create(&payment).await;
    assert!(
        create_result.is_ok(),
        "Failed to create payment: {:?}",
        create_result
    );
    println!("✅ Payment created successfully");

    // Test payment READ
    let retrieved_payment = payment_repo.find_by_id(&payment_id).await.unwrap();
    assert!(
        retrieved_payment.is_some(),
        "Payment not found after creation"
    );
    let retrieved = retrieved_payment.unwrap();

    assert_eq!(retrieved.id(), payment.id());
    assert_eq!(retrieved.account_id(), payment.account_id());
    assert_eq!(retrieved.amount().amount_cents(), 9999); // $99.99
    assert_eq!(retrieved.status(), &PaymentStatus::Pending);
    assert_eq!(retrieved.payment_method(), &PaymentMethod::Card);
    assert_eq!(
        retrieved.description(),
        Some(&"Database integration test payment".to_string())
    );
    assert_eq!(
        retrieved.customer_email(),
        Some(&"customer@example.com".to_string())
    );
    println!("✅ Payment retrieved with correct data");

    // Test payment UPDATE (status transition)
    let mut processing_payment = retrieved;
    processing_payment.start_processing().unwrap();
    processing_payment.mark_successful().unwrap();

    let update_result = payment_repo.update(&processing_payment).await;
    assert!(
        update_result.is_ok(),
        "Failed to update payment: {:?}",
        update_result
    );

    // Verify status update
    let updated_payment = payment_repo.find_by_id(&payment_id).await.unwrap().unwrap();
    assert_eq!(updated_payment.status(), &PaymentStatus::Success);
    println!("✅ Payment status updated successfully");

    // Test query operations
    let account_payments = payment_repo.find_by_account_id(&account_id).await.unwrap();
    assert_eq!(account_payments.len(), 1);
    assert_eq!(account_payments[0].id(), payment.id());
    println!("✅ Payment query by account ID works");

    let successful_payments = payment_repo
        .find_by_status(&PaymentStatus::Success)
        .await
        .unwrap();
    assert!(!successful_payments.is_empty());
    println!("✅ Payment query by status works");

    // Test payment statistics
    let stats = payment_repo.get_payment_stats(&account_id).await.unwrap();
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.successful_payments, 1);
    assert_eq!(stats.successful_amount_cents, 9999);
    println!("✅ Payment statistics calculated correctly");

    // Cleanup
    cleanup_test_account(&pool, account_id.value()).await;
    println!("✅ Payment database test cleanup completed");
}

#[tokio::test]
#[ignore] // Run with: cargo test database_test -- --ignored
async fn test_transaction_consistency() {
    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Create test account
    let merchant_name = MerchantName::new("Transaction Test Merchant".to_string()).unwrap();
    let email = Email::new("transaction@test.com".to_string()).unwrap();
    let mut account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Add initial balance
    let initial_balance = Money::from_major_units(1000.0, "USD".to_string()).unwrap();
    account.credit(&initial_balance).unwrap();

    account_repo.save(&account).await.unwrap();
    println!("✅ Test account created with initial balance");

    // Create and process multiple payments
    let payment_amounts = vec![100.0, 250.50, 75.25];
    let mut payment_ids = Vec::new();

    for (i, amount_value) in payment_amounts.iter().enumerate() {
        let amount = Money::from_major_units(*amount_value, "USD".to_string()).unwrap();
        let payment = Payment::create_new(
            account_id.clone(),
            amount.clone(),
            PaymentMethod::Card,
            Some(format!("Transaction test payment #{}", i + 1)),
            Some(format!("customer{}@test.com", i + 1)),
            None,
        )
        .unwrap();

        payment_ids.push(payment.id().clone());
        payment_repo.create(&payment).await.unwrap();

        // Process payment
        let mut retrieved_payment = payment_repo
            .find_by_id(payment.id())
            .await
            .unwrap()
            .unwrap();
        retrieved_payment.start_processing().unwrap();
        retrieved_payment.mark_successful().unwrap();
        payment_repo.update(&retrieved_payment).await.unwrap();

        // Update account balance (simulate payment processing)
        account.debit(&amount).unwrap();
        account_repo.update(&account).await.unwrap();
    }

    println!("✅ Multiple payments created and processed");

    // Verify final state
    let final_account = account_repo.find_by_id(&account_id).await.unwrap().unwrap();
    let expected_balance = 100000 - 10000 - 25050 - 7525; // Initial - payment amounts (in cents)
    assert_eq!(final_account.balance().amount_cents(), expected_balance);
    println!("✅ Account balance correctly updated after all transactions");

    // Verify all payments are successful
    let account_payments = payment_repo.find_by_account_id(&account_id).await.unwrap();
    assert_eq!(account_payments.len(), 3);
    for payment in account_payments {
        assert_eq!(payment.status(), &PaymentStatus::Success);
    }
    println!("✅ All payments processed to successful state");

    // Test payment statistics consistency
    let stats = payment_repo.get_payment_stats(&account_id).await.unwrap();
    assert_eq!(stats.total_payments, 3);
    assert_eq!(stats.successful_payments, 3);
    assert_eq!(stats.failed_payments, 0);

    let total_payment_amount = 10000 + 25050 + 7525; // Sum of payment amounts in cents
    assert_eq!(stats.successful_amount_cents, total_payment_amount);
    println!("✅ Payment statistics are consistent");

    // Cleanup
    cleanup_test_account(&pool, account_id.value()).await;
    println!("✅ Transaction consistency test cleanup completed");
}

#[tokio::test]
#[ignore] // Run with: cargo test database_test -- --ignored
async fn test_error_handling_and_constraints() {
    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Test 1: Invalid account creation (duplicate email)
    let merchant1 = MerchantName::new("Merchant One".to_string()).unwrap();
    let merchant2 = MerchantName::new("Merchant Two".to_string()).unwrap();
    let same_email = Email::new("shared@email.com".to_string()).unwrap();

    let account1 = Account::create_new(merchant1, same_email.clone(), "USD".to_string()).unwrap();
    let account2 = Account::create_new(merchant2, same_email, "USD".to_string()).unwrap();

    // First account should succeed
    let result1 = account_repo.save(&account1).await;
    assert!(result1.is_ok(), "First account creation should succeed");

    // Second account with same email should fail
    let result2 = account_repo.save(&account2).await;
    assert!(result2.is_err(), "Duplicate email should be rejected");
    println!("✅ Duplicate email constraint properly enforced");

    // Test 2: Payment with invalid account ID
    let invalid_account_id = AccountId::generate(); // Random, non-existent account
    let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
    let invalid_payment = Payment::create_new(
        invalid_account_id,
        amount,
        PaymentMethod::Card,
        Some("Invalid account payment".to_string()),
        None,
        None,
    )
    .unwrap();

    let invalid_payment_result = payment_repo.create(&invalid_payment).await;
    assert!(
        invalid_payment_result.is_err(),
        "Payment with invalid account should fail"
    );
    println!("✅ Foreign key constraint properly enforced");

    // Test 3: Attempt to retrieve non-existent records
    let non_existent_account_id = AccountId::generate();
    let non_existent_account = account_repo
        .find_by_id(&non_existent_account_id)
        .await
        .unwrap();
    assert!(
        non_existent_account.is_none(),
        "Non-existent account should return None"
    );

    let non_existent_payment_id = rusted_ca::domain::value_object::PaymentId::generate();
    let non_existent_payment = payment_repo
        .find_by_id(&non_existent_payment_id)
        .await
        .unwrap();
    assert!(
        non_existent_payment.is_none(),
        "Non-existent payment should return None"
    );
    println!("✅ Non-existent record queries handled correctly");

    // Cleanup
    cleanup_test_account(&pool, account1.id().value()).await;
    println!("✅ Error handling test cleanup completed");
}

// Performance test with multiple concurrent operations
#[tokio::test]
#[ignore] // Run with: cargo test database_test -- --ignored
async fn test_concurrent_operations() {
    use tokio::time::{Duration, timeout};

    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = Arc::new(MySqlAccountRepository::new(pool.clone()));

    // Create test accounts concurrently
    let mut handles = Vec::new();
    let num_accounts = 5;

    for i in 0..num_accounts {
        let repo = account_repo.clone();
        let handle = tokio::spawn(async move {
            let merchant_name = MerchantName::new(format!("Concurrent Merchant {}", i)).unwrap();
            let email = Email::new(format!("concurrent{}@test.com", i)).unwrap();
            let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();

            let result = repo.save(&account).await;
            (account.id().clone(), result)
        });
        handles.push(handle);
    }

    // Wait for all operations to complete with timeout
    let results = timeout(Duration::from_secs(30), async {
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        results
    })
    .await;

    assert!(results.is_ok(), "Concurrent operations timed out");
    let results = results.unwrap();

    // Verify all operations succeeded
    let mut account_ids = Vec::new();
    for (account_id, result) in results {
        assert!(
            result.is_ok(),
            "Concurrent account creation failed: {:?}",
            result
        );
        account_ids.push(account_id);
    }

    println!("✅ {} accounts created concurrently", account_ids.len());

    // Cleanup all created accounts
    for account_id in account_ids {
        cleanup_test_account(&pool, account_id.value()).await;
    }

    println!("✅ Concurrent operations test completed successfully");
}
