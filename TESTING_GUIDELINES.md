# 🧪 テストガイドライン - Rusted CA クリーンアーキテクチャテンプレート

## 📋 概要

このドキュメントでは、Rusted CAクリーンアーキテクチャテンプレートにおけるテストの設計思想、ベストプラクティス、および新規テスト追加時のガイドラインを説明します。

## 🏗️ テストアーキテクチャ設計思想

### Clean Architecture準拠のテスト構造

```
テスト構造
├── tests/                    # 統合テスト
│   ├── simple_tests.rs      # 基本機能・パフォーマンス
│   └── api_basic_tests.rs   # API統合テスト
└── src/                     # ユニットテスト
    ├── domain/              # ドメイン層テスト
    ├── application/         # アプリケーション層テスト
    ├── infrastructure/      # インフラ層テスト
    └── presentation/        # プレゼンテーション層テスト
```

### テスト種別と責務

#### 1. ユニットテスト (src/内)
- **責務**: 各層の独立したロジック検証
- **対象**: 単一関数・メソッド・構造体
- **依存**: 外部依存なし、または最小限のモック

#### 2. 統合テスト (tests/内)
- **責務**: 複数コンポーネント間の連携検証
- **対象**: API エンドポイント、ワークフロー
- **依存**: 実際のHTTPリクエスト・レスポンス

#### 3. パフォーマンステスト
- **責務**: 処理速度・並行処理の安全性検証
- **対象**: 高負荷シナリオ、リソース使用量
- **依存**: 時間計測、並行実行

## 📚 テストベストプラクティス

### 1. テスト命名規則

#### 良い例
```rust
#[test]
fn test_user_creation_with_valid_email() { ... }

#[test] 
fn test_password_hashing_fails_with_empty_input() { ... }

#[tokio::test]
async fn test_concurrent_requests_maintain_data_consistency() { ... }
```

#### 悪い例
```rust
#[test]
fn test1() { ... }

#[test]
fn user_test() { ... }

#[test]
fn test_stuff() { ... }
```

### 2. アサーション設計

#### 明確なエラーメッセージ
```rust
// Good
assert_eq!(
    result.status(), 
    StatusCode::OK,
    "Health check should return OK status, got: {:?}", 
    result.status()
);

// Bad
assert_eq!(result.status(), StatusCode::OK);
```

#### 複数の検証項目
```rust
#[test]
fn test_user_creation_complete() {
    let user = create_test_user()?;
    
    // 必須フィールドの検証
    assert!(!user.id().value().is_empty());
    assert!(user.email().value().contains('@'));
    assert!(!user.name().value().is_empty());
    
    // ビジネスルールの検証
    assert!(user.email().value().len() <= 255);
    assert!(user.name().value().len() >= 3);
}
```

### 3. エラーハンドリングテスト

```rust
#[test]
fn test_invalid_email_format() {
    let invalid_emails = vec![
        "",
        "invalid-email",
        "missing@domain", 
        "@missing-local.com"
    ];
    
    for email in invalid_emails {
        let result = Email::new(email.to_string());
        assert!(
            result.is_err(),
            "Email '{}' should be invalid but was accepted", 
            email
        );
    }
}
```

### 4. 非同期テスト

```rust
#[tokio::test]
async fn test_async_operation_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        async_operation()
    ).await;
    
    assert!(result.is_ok(), "Operation should complete within 100ms");
    assert_eq!(result.unwrap(), expected_value);
}
```

### 5. 並行処理テスト

```rust
#[tokio::test]
async fn test_concurrent_safety() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();
    
    // 10個のタスクを並行実行
    for _ in 0..10 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    
    // 全タスク完了を待機
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 結果検証
    assert_eq!(counter.load(Ordering::Relaxed), 1000);
}
```

## 🎯 層別テストガイドライン

### ドメイン層テスト

#### バリューオブジェクト
```rust
#[test]
fn test_email_validation() {
    // 正常ケース
    let email = Email::new("test@example.com".to_string());
    assert!(email.is_ok());
    
    // 異常ケース
    let invalid = Email::new("invalid".to_string());
    assert!(invalid.is_err());
    
    // エラー内容の検証
    match invalid {
        Err(DomainError::InvalidEmail { email, reason }) => {
            assert_eq!(email, "invalid");
            assert!(reason.contains("Invalid email format"));
        },
        _ => panic!("Expected InvalidEmail error")
    }
}
```

#### エンティティ
```rust
#[test]
fn test_user_entity_business_rules() {
    let user = User::new(
        UserId::new("user-123"),
        Email::new("test@example.com".to_string())?,
        UserName::new("testuser".to_string())?,
        Password::new("SecurePass123!".to_string())?,
        None, None
    )?;
    
    // ビジネスルール検証
    assert_eq!(user.id().value(), "user-123");
    assert!(user.email().value().contains("@"));
}
```

### アプリケーション層テスト

#### ユースケース
```rust
#[tokio::test]
async fn test_create_user_usecase() {
    let mock_repo = MockUserRepository::new();
    let usecase = CreateUserUsecase::new(mock_repo);
    
    let request = CreateUserRequest {
        email: "test@example.com".to_string(),
        name: "testuser".to_string(),
        password: "SecurePass123!".to_string(),
    };
    
    let result = usecase.execute(request).await;
    assert!(result.is_ok());
    
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

### インフラ層テスト

#### リポジトリ実装
```rust
#[tokio::test]
async fn test_repository_error_handling() {
    let mock_repo = MockUserRepository::new();
    mock_repo.set_should_fail(true);
    
    let user = create_test_user();
    let result = mock_repo.save(&user).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), InfrastructureError::DatabaseOperation(_)));
}
```

### プレゼンテーション層テスト

#### APIエンドポイント
```rust
#[tokio::test]
async fn test_health_check_endpoint() {
    let app = create_test_app();
    
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

## 🚀 新規テスト追加ガイド

### 1. テストファイル作成

```rust
// tests/new_feature_tests.rs
//! Tests for new feature functionality

mod new_feature_tests {
    use super::*;
    
    #[test]
    fn test_new_feature_basic_functionality() {
        // テスト実装
    }
    
    #[tokio::test]
    async fn test_new_feature_async_operation() {
        // 非同期テスト実装
    }
}
```

### 2. モックの活用

```rust
// テストヘルパー
fn create_mock_dependencies() -> (MockRepo, MockService) {
    let repo = MockRepo::new();
    let service = MockService::new();
    (repo, service)
}

#[tokio::test]
async fn test_with_mocks() {
    let (mock_repo, mock_service) = create_mock_dependencies();
    mock_repo.set_should_fail(false);
    
    // テスト実装
}
```

### 3. テストデータの管理

```rust
// テストデータビルダー
struct TestUserBuilder {
    email: String,
    name: String,
    password: String,
}

impl TestUserBuilder {
    fn new() -> Self {
        Self {
            email: "test@example.com".to_string(),
            name: "testuser".to_string(),
            password: "SecurePass123!".to_string(),
        }
    }
    
    fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }
    
    fn build(self) -> CreateUserRequest {
        CreateUserRequest {
            email: self.email,
            name: self.name,
            password: self.password,
        }
    }
}

#[test]
fn test_with_builder() {
    let request = TestUserBuilder::new()
        .with_email("custom@example.com")
        .build();
        
    assert_eq!(request.email, "custom@example.com");
}
```

## 📊 パフォーマンステスト

### 実行時間の測定
```rust
#[test]
fn test_operation_performance() {
    let start = std::time::Instant::now();
    
    // 測定対象の処理
    for _ in 0..1000 {
        expensive_operation();
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Operation too slow: {:?}", duration);
}
```

### メモリ使用量の監視
```rust
#[test]
fn test_memory_efficiency() {
    let initial_allocation = get_memory_usage();
    
    // メモリを使用する処理
    let large_data = create_large_dataset();
    process_data(large_data);
    
    let final_allocation = get_memory_usage();
    let memory_growth = final_allocation - initial_allocation;
    
    assert!(memory_growth < MAX_ACCEPTABLE_GROWTH);
}
```

## 🔧 トラブルシューティング

### よくある問題と解決策

#### 1. 非同期テストのタイムアウト
```rust
// 問題: テストが無限に待機
#[tokio::test]
async fn problematic_test() {
    let result = never_completing_future().await; // これは完了しない
    assert!(result.is_ok());
}

// 解決: タイムアウトを設定
#[tokio::test]
async fn fixed_test() {
    let result = tokio::time::timeout(
        Duration::from_millis(1000),
        never_completing_future()
    ).await;
    
    assert!(result.is_err()); // タイムアウトエラーを期待
}
```

#### 2. 並行テストでの競合状態
```rust
// 問題: 共有状態による競合
static mut COUNTER: i32 = 0;

#[test]
fn racy_test() {
    unsafe {
        COUNTER += 1;
        assert_eq!(COUNTER, 1); // 他のテストと競合する可能性
    }
}

// 解決: テスト毎に独立した状態
#[test]
fn isolated_test() {
    let counter = AtomicI32::new(0);
    counter.fetch_add(1, Ordering::Relaxed);
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}
```

#### 3. モックの設定ミス
```rust
// 問題: モックが期待通りに動作しない
#[tokio::test]
async fn problematic_mock_test() {
    let mock_repo = MockRepo::new();
    // mock_repo.set_expectation() を忘れている
    
    let result = service.call_repo(&mock_repo).await;
    // 予期しない結果になる
}

// 解決: 明示的なモック設定
#[tokio::test] 
async fn properly_mocked_test() {
    let mock_repo = MockRepo::new();
    mock_repo.expect_save()
        .times(1)
        .returning(|_| Ok(()));
    
    let result = service.call_repo(&mock_repo).await;
    assert!(result.is_ok());
}
```

## 📈 継続的な改善

### テストメトリクス監視
- **実行時間**: 全テストが2秒以内で完了
- **通過率**: 100%維持
- **カバレッジ**: 新機能追加時に80%以上維持

### 定期的なレビュー項目
1. **廃止予定テスト**: 古い機能のテスト削除
2. **冗長テスト**: 重複するテストの統合
3. **フレイキーテスト**: 不安定なテストの修正
4. **パフォーマンス回帰**: 実行時間の監視

---

## 🎯 まとめ

このガイドラインに従うことで：

✅ **保守性の高いテスト**: 理解しやすく変更に強い  
✅ **信頼性の高い検証**: エラーケースを含む網羅的テスト  
✅ **効率的な実行**: 高速で並行実行可能  
✅ **継続的品質**: 新機能追加時の回帰防止  

新しいテスト追加時は、このガイドラインを参考にして一貫性のある高品質なテストを作成してください。