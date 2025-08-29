//tests/real_db_test.rs
// Real Database Write/Read Integration Tests
// 2025/8/28

use chrono;
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
use std::env;
use std::sync::Arc;

// Helper function to setup test database connection
async fn setup_test_db() -> Result<Pool<MySql>, Box<dyn std::error::Error>> {
    // Generate unique database name for each test
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_name = format!("rusted_ca_test_{}", test_id);

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| format!("mysql://root:root@localhost:3306/{}", db_name));

    println!("🔗 Connecting to database: {}", database_url);

    // First, connect to MySQL server (without specifying database)
    let server_url = "mysql://root:root@localhost:3306";
    let server_pool = sqlx::MySqlPool::connect(server_url).await?;

    // Drop and recreate database to ensure clean state
    println!("🗑️  Dropping existing database...");
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
        .execute(&server_pool)
        .await;

    println!("🏗️  Creating fresh database: {}", db_name);
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&server_pool)
        .await?;

    // Close server connection
    server_pool.close().await;

    // Now connect to the specific database
    let pool = sqlx::MySqlPool::connect(&database_url).await?;

    // Test connection
    sqlx::query("SELECT 1").fetch_one(&pool).await?;
    println!("✅ Database connection successful");

    // Run migrations to ensure tables exist
    println!("🔧 Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("✅ Migrations completed");

    Ok(pool)
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &Pool<MySql>, account_id: &str) -> Result<(), sqlx::Error> {
    println!("🧹 Cleaning up test data for account: {}", account_id);

    // Delete payments first (foreign key constraint)
    let payments_deleted = sqlx::query("DELETE FROM payments WHERE account_id = ?")
        .bind(account_id)
        .execute(pool)
        .await?
        .rows_affected();

    // Delete account
    let accounts_deleted = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await?
        .rows_affected();

    println!(
        "🗑️  Deleted {} payments, {} accounts",
        payments_deleted, accounts_deleted
    );
    Ok(())
}

#[tokio::test]
async fn test_real_account_database_operations() {
    println!("\n🚀 Starting Real Account Database Test");

    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let repository = MySqlAccountRepository::new(pool.clone());

    // Create test account with unique email
    let merchant_name = MerchantName::new("Real DB Test Merchant".to_string()).unwrap();
    let unique_email = format!(
        "realdb+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let email = Email::new(unique_email.clone()).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    println!("📝 Creating account with ID: {}", account_id.value());

    // Test CREATE - Write to database
    let create_result = repository.save(&account).await;
    assert!(
        create_result.is_ok(),
        "Failed to create account: {:?}",
        create_result
    );
    println!("✅ Account written to database successfully");

    // Test READ - Read from database
    println!("📖 Reading account from database...");
    let retrieved_account = repository
        .find_by_id(&account_id)
        .await
        .expect("Database query failed");

    assert!(retrieved_account.is_some(), "Account not found in database");
    let retrieved = retrieved_account.unwrap();

    // Verify data integrity
    assert_eq!(retrieved.id().value(), account.id().value());
    assert_eq!(retrieved.merchant_name().value(), "Real DB Test Merchant");
    assert_eq!(retrieved.email().value(), unique_email);
    assert_eq!(retrieved.balance().amount_cents(), 0);
    assert_eq!(retrieved.balance().currency_code(), "USD");
    assert_eq!(retrieved.status().as_str(), "ACTIVE");

    println!("✅ Account read from database with correct data");
    println!("   - ID: {}", retrieved.id().value());
    println!("   - Merchant: {}", retrieved.merchant_name().value());
    println!("   - Email: {}", retrieved.email().value());
    println!(
        "   - Balance: {} {}",
        retrieved.balance().to_major_units(),
        retrieved.balance().currency_code()
    );

    // Test UPDATE - Modify balance
    println!("💰 Testing balance update...");
    let mut updated_account = retrieved;
    let credit_amount = Money::from_major_units(500.75, "USD".to_string()).unwrap();
    updated_account.credit(&credit_amount).unwrap();

    let update_result = repository.update(&updated_account).await;
    assert!(
        update_result.is_ok(),
        "Failed to update account: {:?}",
        update_result
    );
    println!("✅ Account balance updated in database");

    // Verify update by reading again
    println!("📖 Verifying balance update...");
    let updated_retrieved = repository
        .find_by_id(&account_id)
        .await
        .expect("Database query failed")
        .expect("Account should exist");

    assert_eq!(updated_retrieved.balance().amount_cents(), 50075); // $500.75
    assert_eq!(updated_retrieved.balance().to_major_units(), 500.75);
    println!(
        "✅ Balance update verified: ${}",
        updated_retrieved.balance().to_major_units()
    );

    // Test business rule: Try to create duplicate email
    println!("🚫 Testing duplicate email constraint...");
    let duplicate_account = Account::create_new(
        MerchantName::new("Duplicate Merchant".to_string()).unwrap(),
        Email::new(unique_email).unwrap(), // Same email
        "USD".to_string(),
    )
    .unwrap();

    let duplicate_result = repository.save(&duplicate_account).await;
    assert!(
        duplicate_result.is_err(),
        "Should fail to create duplicate email account"
    );
    println!("✅ Duplicate email constraint properly enforced");

    // Cleanup
    cleanup_test_data(&pool, account_id.value())
        .await
        .expect("Cleanup failed");
    println!("✅ Real Account Database Test completed successfully\n");
}

#[tokio::test]
async fn test_real_payment_database_operations() {
    println!("\n🚀 Starting Real Payment Database Test");

    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Create test account first with unique email
    let merchant_name = MerchantName::new("Payment DB Test Merchant".to_string()).unwrap();
    let unique_email = format!(
        "payment_db+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let email = Email::new(unique_email).unwrap();
    let account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    account_repo.save(&account).await.unwrap();
    println!("✅ Test account created for payment testing");

    // Create test payment
    let amount = Money::from_major_units(199.99, "USD".to_string()).unwrap();
    let payment = Payment::create_new(
        account_id.clone(),
        amount,
        PaymentMethod::Card,
        Some("Real DB payment test".to_string()),
        Some("customer@realdb.test".to_string()),
        Some(serde_json::json!({
            "test_type": "real_database_test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "test_data": {
                "environment": "integration_test",
                "version": "1.0"
            }
        })),
    )
    .unwrap();
    let payment_id = payment.id().clone();

    println!("📝 Creating payment with ID: {}", payment_id.value());

    // Test Payment CREATE - Write to database
    let create_result = payment_repo.create(&payment).await;
    assert!(
        create_result.is_ok(),
        "Failed to create payment: {:?}",
        create_result
    );
    println!("✅ Payment written to database successfully");

    // Test Payment READ - Read from database
    println!("📖 Reading payment from database...");
    let retrieved_payment = payment_repo
        .find_by_id(&payment_id)
        .await
        .expect("Database query failed");

    assert!(retrieved_payment.is_some(), "Payment not found in database");
    let retrieved = retrieved_payment.unwrap();

    // Verify payment data integrity
    assert_eq!(retrieved.id().value(), payment.id().value());
    assert_eq!(retrieved.account_id().value(), account_id.value());
    assert_eq!(retrieved.amount().amount_cents(), 19999); // $199.99
    assert_eq!(retrieved.status(), &PaymentStatus::Pending);
    assert_eq!(retrieved.payment_method(), &PaymentMethod::Card);
    assert_eq!(
        retrieved.description(),
        Some(&"Real DB payment test".to_string())
    );
    assert_eq!(
        retrieved.customer_email(),
        Some(&"customer@realdb.test".to_string())
    );

    // Verify metadata
    let metadata = retrieved.metadata().expect("Metadata should exist");
    assert_eq!(metadata["test_type"], "real_database_test");
    assert!(metadata["test_data"]["environment"] == "integration_test");

    println!("✅ Payment read from database with correct data");
    println!("   - ID: {}", retrieved.id().value());
    println!("   - Amount: ${}", retrieved.amount().to_major_units());
    println!("   - Status: {}", retrieved.status().as_str());
    println!("   - Method: {}", retrieved.payment_method().as_str());

    // Test Payment UPDATE - Process payment through lifecycle
    println!("⚡ Testing payment lifecycle updates...");

    // Step 1: Start processing
    let mut processing_payment = retrieved;
    processing_payment.start_processing().unwrap();
    let update_result = payment_repo.update(&processing_payment).await;
    assert!(
        update_result.is_ok(),
        "Failed to update payment to processing"
    );
    println!("✅ Payment status updated to PROCESSING");

    // Step 2: Mark successful
    processing_payment.mark_successful().unwrap();
    let success_result = payment_repo.update(&processing_payment).await;
    assert!(
        success_result.is_ok(),
        "Failed to update payment to success"
    );
    println!("✅ Payment status updated to SUCCESS");

    // Verify lifecycle updates by reading from DB
    let final_payment = payment_repo
        .find_by_id(&payment_id)
        .await
        .expect("Database query failed")
        .expect("Payment should exist");

    assert_eq!(final_payment.status(), &PaymentStatus::Success);
    println!("✅ Payment lifecycle verified in database");

    // Test Payment QUERIES
    println!("🔍 Testing payment query operations...");

    // Query by account ID
    let account_payments = payment_repo
        .find_by_account_id(&account_id)
        .await
        .expect("Query by account ID failed");
    assert_eq!(account_payments.len(), 1);
    assert_eq!(account_payments[0].id().value(), payment_id.value());
    println!(
        "✅ Query by account ID successful: {} payments found",
        account_payments.len()
    );

    // Query by status
    let successful_payments = payment_repo
        .find_by_status(&PaymentStatus::Success)
        .await
        .expect("Query by status failed");
    assert!(!successful_payments.is_empty());
    println!(
        "✅ Query by status successful: {} successful payments found",
        successful_payments.len()
    );

    // Test payment statistics
    let stats = payment_repo
        .get_payment_stats(&account_id)
        .await
        .expect("Failed to get payment stats");
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.successful_payments, 1);
    assert_eq!(stats.successful_amount_cents, 19999);
    println!("✅ Payment statistics calculated correctly");
    println!("   - Total payments: {}", stats.total_payments);
    println!("   - Successful: {}", stats.successful_payments);
    println!(
        "   - Total amount: ${}",
        stats.successful_amount_cents as f64 / 100.0
    );

    // Cleanup
    cleanup_test_data(&pool, account_id.value())
        .await
        .expect("Cleanup failed");
    println!("✅ Real Payment Database Test completed successfully\n");
}

#[tokio::test]
async fn test_complete_financial_transaction_flow() {
    println!("\n🚀 Starting Complete Financial Transaction Flow Test");

    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Step 1: Create merchant account with initial balance
    let merchant_name = MerchantName::new("Complete Flow Merchant".to_string()).unwrap();
    let unique_email = format!(
        "flow+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let email = Email::new(unique_email).unwrap();
    let mut account = Account::create_new(merchant_name, email, "USD".to_string()).unwrap();
    let account_id = account.id().clone();

    // Add initial balance
    let initial_balance = Money::from_major_units(1000.0, "USD".to_string()).unwrap();
    account.credit(&initial_balance).unwrap();

    account_repo.save(&account).await.unwrap();
    println!("✅ Created merchant account with $1000 initial balance");

    // Step 2: Process multiple payments
    let payment_scenarios = vec![
        (150.25, PaymentMethod::Card, "Credit card payment"),
        (75.50, PaymentMethod::BankTransfer, "Bank transfer payment"),
        (
            299.99,
            PaymentMethod::DigitalWallet,
            "Digital wallet payment",
        ),
    ];

    let mut processed_payments = Vec::new();

    for (amount_value, method, description) in payment_scenarios {
        println!(
            "💳 Processing {} payment: ${}",
            method.as_str(),
            amount_value
        );

        // Create payment
        let amount = Money::from_major_units(amount_value, "USD".to_string()).unwrap();
        let payment = Payment::create_new(
            account_id.clone(),
            amount.clone(),
            method.clone(),
            Some(description.to_string()),
            Some(format!("customer@{}.com", method.as_str().to_lowercase())),
            None,
        )
        .unwrap();

        let payment_id = payment.id().clone();

        // Save to database
        payment_repo.create(&payment).await.unwrap();

        // Process payment lifecycle
        let mut retrieved_payment = payment_repo.find_by_id(&payment_id).await.unwrap().unwrap();
        retrieved_payment.start_processing().unwrap();
        payment_repo.update(&retrieved_payment).await.unwrap();

        retrieved_payment.mark_successful().unwrap();
        payment_repo.update(&retrieved_payment).await.unwrap();

        // Update account balance (simulate settlement)
        account.debit(&amount).unwrap();
        account_repo.update(&account).await.unwrap();

        processed_payments.push(payment_id);
        println!("✅ Payment processed successfully: ${}", amount_value);
    }

    // Step 3: Verify final state in database
    println!("📊 Verifying final transaction state...");

    // Check account balance
    let final_account = account_repo.find_by_id(&account_id).await.unwrap().unwrap();
    let expected_balance = 100000 - 15025 - 7550 - 29999; // Initial - payments (in cents)
    assert_eq!(final_account.balance().amount_cents(), expected_balance);
    println!(
        "✅ Final account balance: ${}",
        final_account.balance().to_major_units()
    );

    // Check all payments are successful
    let account_payments = payment_repo.find_by_account_id(&account_id).await.unwrap();
    assert_eq!(account_payments.len(), 3);
    for payment in &account_payments {
        assert_eq!(payment.status(), &PaymentStatus::Success);
    }
    println!(
        "✅ All {} payments processed to successful state",
        account_payments.len()
    );

    // Check payment statistics
    let stats = payment_repo.get_payment_stats(&account_id).await.unwrap();
    assert_eq!(stats.total_payments, 3);
    assert_eq!(stats.successful_payments, 3);
    assert_eq!(stats.failed_payments, 0);

    let total_payment_amount = 15025 + 7550 + 29999; // Sum of payments in cents
    assert_eq!(stats.successful_amount_cents, total_payment_amount);

    println!("📈 Transaction Statistics:");
    println!("   - Total payments: {}", stats.total_payments);
    println!("   - Successful: {}", stats.successful_payments);
    println!("   - Failed: {}", stats.failed_payments);
    println!(
        "   - Total processed: ${}",
        stats.successful_amount_cents as f64 / 100.0
    );

    // Step 4: Test refund scenario
    println!("↩️  Testing refund scenario...");
    let refund_payment_id = processed_payments[0].clone(); // Refund the first payment

    let mut refund_payment = payment_repo
        .find_by_id(&refund_payment_id)
        .await
        .unwrap()
        .unwrap();
    let refund_amount = refund_payment.amount().clone();

    refund_payment
        .refund(None, "Customer requested refund".to_string())
        .unwrap();
    payment_repo.update(&refund_payment).await.unwrap();

    // Update account balance for refund
    account.credit(&refund_amount).unwrap();
    account_repo.update(&account).await.unwrap();

    // Verify refund
    let refunded_payment = payment_repo
        .find_by_id(&refund_payment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(refunded_payment.status(), &PaymentStatus::Refunded);

    let refunded_account = account_repo.find_by_id(&account_id).await.unwrap().unwrap();
    let expected_after_refund = expected_balance + 15025; // Add refund back
    assert_eq!(
        refunded_account.balance().amount_cents(),
        expected_after_refund
    );

    println!(
        "✅ Refund processed successfully: ${}",
        refund_amount.to_major_units()
    );
    println!(
        "✅ Account balance after refund: ${}",
        refunded_account.balance().to_major_units()
    );

    // Cleanup
    cleanup_test_data(&pool, account_id.value())
        .await
        .expect("Cleanup failed");
    println!("✅ Complete Financial Transaction Flow Test completed successfully\n");
}

#[tokio::test]
async fn test_database_constraints_and_errors() {
    println!("\n🚀 Starting Database Constraints and Error Handling Test");

    // Setup database connection
    let pool = setup_test_db()
        .await
        .expect("Failed to connect to database");
    let account_repo = MySqlAccountRepository::new(pool.clone());
    let payment_repo = MySqlPaymentRepository::new(Arc::new(pool.clone()));

    // Test 1: Unique email constraint
    println!("🔒 Testing unique email constraint...");
    let unique_email = format!(
        "constraint+{}@test.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let email_value = unique_email.clone();

    let account1 = Account::create_new(
        MerchantName::new("First Merchant".to_string()).unwrap(),
        Email::new(email_value.clone()).unwrap(),
        "USD".to_string(),
    )
    .unwrap();

    let account2 = Account::create_new(
        MerchantName::new("Second Merchant".to_string()).unwrap(),
        Email::new(email_value).unwrap(), // Same email
        "USD".to_string(),
    )
    .unwrap();

    // First account should succeed
    let result1 = account_repo.save(&account1).await;
    assert!(result1.is_ok(), "First account creation should succeed");
    println!("✅ First account created successfully");

    // Second account should fail
    let result2 = account_repo.save(&account2).await;
    assert!(result2.is_err(), "Duplicate email should be rejected");
    println!("✅ Duplicate email constraint enforced");

    // Test 2: Foreign key constraint (payment with non-existent account)
    println!("🔗 Testing foreign key constraint...");
    let invalid_account_id = AccountId::generate(); // Random non-existent ID
    let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();

    let invalid_payment = Payment::create_new(
        invalid_account_id,
        amount,
        PaymentMethod::Card,
        Some("Invalid payment".to_string()),
        None,
        None,
    )
    .unwrap();

    let invalid_result = payment_repo.create(&invalid_payment).await;
    assert!(
        invalid_result.is_err(),
        "Payment with invalid account should fail"
    );
    println!("✅ Foreign key constraint enforced");

    // Test 3: Non-existent record queries
    println!("🔍 Testing non-existent record queries...");
    let fake_account_id = AccountId::generate();
    let fake_payment_id = rusted_ca::domain::value_object::PaymentId::generate();

    let missing_account = account_repo.find_by_id(&fake_account_id).await.unwrap();
    assert!(
        missing_account.is_none(),
        "Should return None for non-existent account"
    );

    let missing_payment = payment_repo.find_by_id(&fake_payment_id).await.unwrap();
    assert!(
        missing_payment.is_none(),
        "Should return None for non-existent payment"
    );

    println!("✅ Non-existent record queries handled correctly");

    // Cleanup
    cleanup_test_data(&pool, account1.id().value())
        .await
        .expect("Cleanup failed");
    println!("✅ Database Constraints and Error Handling Test completed successfully\n");
}
