# DIContainer金融系API統合ガイド

## 📋 概要

Rusted-CAプロジェクトの金融系API（Account/Payment）をDIContainer経由で完全統合するための詳細な実装手順書です。

現在の状況：
- ✅ APIルート定義完了（13エンドポイント）
- ✅ Use Cases実装完了
- ✅ Repository実装完了 
- ❌ **DIContainer統合未完了**（プレースホルダー実装のみ）

## 🎯 目標

プレースホルダー実装から実際のビジネスロジック実行への移行を完了し、完全に動作する金融系APIを実現する。

---

## 📊 現在の実装状況分析

### ✅ 既存実装済み

#### 1. Use Cases
```
src/application/usecases/
├── create_account_usecase.rs     ✓ 実装済み
├── get_account_usecase.rs        ✓ 実装済み  
├── create_payment_usecase.rs     ✓ 実装済み
├── process_payment_usecase.rs    ✓ 実装済み
└── refund_payment_usecase.rs     ✓ 実装済み
```

#### 2. Repositories
```
src/infrastructure/repository/
├── mysql_account_repository.rs   ✓ 実装済み
└── mysql_payment_repository.rs   ✓ 実装済み
```

#### 3. Controllers & DTOs
```
src/presentation/
├── controller/
│   ├── account_controller.rs     ✓ 実装済み
│   └── payment_controller.rs     ✓ 実装済み
└── dto/
    ├── payment_create_request.rs ✓ 実装済み
    ├── payment_operation_request.rs ✓ 実装済み
    └── payment_response.rs       ✓ 実装済み
```

#### 4. API Routes
```
src/infrastructure/web/api_router.rs
- 13個の金融系エンドポイント定義完了 ✓
- プレースホルダーハンドラー実装済み ✓
```

### ❌ 未実装・問題点

#### 1. 型名不一致
- Controller期待: `CreateAccountUseCase` (PascalCase)
- 実装: `CreateAccountUsecase` (小文字) **→ 修正済み**

#### 2. DTO不一致  
- Controller: Presentation DTOs使用
- UseCase: Application DTOs期待
- **→ 変換ロジック必要**

#### 3. DIContainer未統合
- 金融系Use Cases未登録
- Repository依存関係未定義
- **→ 完全実装必要**

---

## 🚀 実装手順

### Phase 1: 依存関係の型不一致解決

#### Step 1-1: Repository Interface確認
既存のRepositoryが適切なInterfaceを実装しているか確認：

```bash
# 確認すべきファイル
src/domain/repository/account_command_repository.rs
src/domain/repository/account_query_repository.rs  
src/domain/repository/payment_command_repository.rs
src/domain/repository/payment_query_repository.rs
```

#### Step 1-2: DTO変換ロジック実装
ControllerでPresentation DTO → Application DTO変換を追加：

```rust
// src/presentation/controller/account_controller.rs
impl AccountController {
    pub async fn create_account(&self, request: Json<AccountCreateRequest>) -> Result<...> {
        // Presentation DTO → Application DTO 変換
        let app_dto = CreateAccountRequestDto::new(
            request.merchant_name.clone(),
            request.email.clone(),
            request.currency_code.clone()
        );
        
        // UseCase実行
        let result = self.create_account_usecase.execute(app_dto).await?;
        
        // Application DTO → Presentation DTO 変換
        let response = AccountResponse::from_app_dto(result);
        Ok(Json(response))
    }
}
```

### Phase 2: DIContainer拡張

#### Step 2-1: DIContainer構造体拡張
`src/infrastructure/di/container.rs`に金融系フィールド追加：

```rust
pub struct DIContainer {
    // 既存フィールド...
    
    // === 金融系追加 ===
    // Account関連
    pub mysql_account_repository: Arc<MySqlAccountRepository>,
    pub create_account_usecase: Arc<CreateAccountUseCase>,
    pub get_account_usecase: Arc<GetAccountUseCase>,
    pub account_controller: Arc<AccountController>,
    
    // Payment関連  
    pub mysql_payment_repository: Arc<MySqlPaymentRepository>,
    pub create_payment_usecase: Arc<CreatePaymentUseCase>,
    pub process_payment_usecase: Arc<ProcessPaymentUseCase>,
    pub refund_payment_usecase: Arc<RefundPaymentUseCase>,
    pub payment_controller: Arc<PaymentController>,
}
```

#### Step 2-2: DIContainer::new()メソッド拡張
初期化ロジックに金融系依存関係追加：

```rust
impl DIContainer {
    pub async fn new(
        tidb_pool: MySqlPool,
        redis_pool: RedisPool,
    ) -> Result<Self, InfrastructureError> {
        // 既存の初期化...
        
        // === 金融系Repository初期化 ===
        let mysql_account_repo = Arc::new(MySqlAccountRepository::new(tidb_pool.clone()));
        let mysql_payment_repo = Arc::new(MySqlPaymentRepository::new(Arc::new(tidb_pool.clone())));
        
        // === Account UseCase初期化 ===
        let create_account_usecase = Arc::new(CreateAccountUseCase::new(
            mysql_account_repo.clone(),
            mysql_account_repo.clone(), // Query repository
        ));
        
        let get_account_usecase = Arc::new(GetAccountUseCase::new(
            mysql_account_repo.clone(),
        ));
        
        // === Payment UseCase初期化 ===
        let create_payment_usecase = Arc::new(CreatePaymentUseCase::new(
            mysql_payment_repo.clone(),
            mysql_payment_repo.clone(), // Query repository
        ));
        
        // 以下同様にprocess_payment_usecase, refund_payment_usecase初期化...
        
        // === Controller初期化 ===
        let account_controller = Arc::new(AccountController::new(
            create_account_usecase.clone(),
            get_account_usecase.clone(),
        ));
        
        let payment_controller = Arc::new(PaymentController::new(
            create_payment_usecase.clone(),
            process_payment_usecase.clone(),
            refund_payment_usecase.clone(),
            mysql_payment_repo.clone(),
        ));
        
        Ok(DIContainer {
            // 既存フィールド...
            mysql_account_repository: mysql_account_repo,
            create_account_usecase,
            get_account_usecase,
            account_controller,
            mysql_payment_repository: mysql_payment_repo,
            create_payment_usecase,
            process_payment_usecase,
            refund_payment_usecase,
            payment_controller,
        })
    }
}
```

### Phase 3: APIハンドラー実装切り替え

#### Step 3-1: プレースホルダー削除
`src/infrastructure/web/api_router.rs`のプレースホルダー実装を実際のController呼び出しに変更：

```rust
async fn create_account_handler(
    State(di): State<Arc<DIContainer>>,
    Json(payload): Json<AccountCreateRequest>,
) -> impl IntoResponse {
    // プレースホルダー削除 → 実際のController呼び出し
    match di.account_controller.create_account(Json(payload)).await {
        Ok(response) => response.into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": error.to_string()
            }))
        ).into_response(),
    }
}
```

#### Step 3-2: 全ハンドラー実装
13個すべてのハンドラーでプレースホルダー → 実装切り替え：

```rust
// Account handlers
async fn create_account_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn get_account_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn update_account_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn delete_account_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn get_account_balance_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn credit_account_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn debit_account_handler(...) -> impl IntoResponse { /* 実装 */ }

// Payment handlers  
async fn create_payment_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn get_payment_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn process_payment_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn refund_payment_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn list_payments_handler(...) -> impl IntoResponse { /* 実装 */ }
async fn get_payment_stats_handler(...) -> impl IntoResponse { /* 実装 */ }
```

---

## 🧪 テスト計画

### Phase 4: 統合テスト

#### Step 4-1: コンパイルテスト
```bash
cargo check
cargo build
```

#### Step 4-2: 単体テスト実行
```bash
cargo test integration_test
cargo test database_test  
cargo test real_db_test
```

#### Step 4-3: APIエンドポイントテスト
```bash
# サーバー起動
cargo run

# Account API テスト
curl -X POST http://localhost:3000/api/account \
  -H "Content-Type: application/json" \
  -d '{"merchant_name":"Test Merchant","email":"test@example.com","currency_code":"USD"}'

# Payment API テスト
curl -X POST http://localhost:3000/api/payment \
  -H "Content-Type: application/json" \
  -d '{"account_id":"acc_123","amount":100.0,"currency_code":"USD","payment_method":"CARD"}'
```

---

## ⚠️ 予想される問題と解決策

### 問題1: Async Trait エラー
```rust
// 解決策: async_trait クレートの使用確認
#[async_trait]
pub trait CreateAccountUsecaseInterface: Send + Sync {
    async fn execute(&self, request_dto: CreateAccountRequestDto) -> ApplicationResult<AccountResponseDto>;
}
```

### 問題2: Repository Interface不一致
```rust
// 解決策: Repository実装でInterface継承確認
impl AccountCommandRepositoryInterface for MySqlAccountRepository {
    async fn save(&self, account: &Account) -> Result<(), InfrastructureError> {
        // 実装
    }
}
```

### 問題3: DTO変換エラー
```rust
// 解決策: 適切な変換関数実装
impl AccountResponse {
    pub fn from_app_dto(dto: AccountResponseDto) -> Self {
        Self {
            id: dto.id,
            merchant_name: dto.merchant_name,
            // 他フィールド変換...
        }
    }
}
```

---

## 📋 実装チェックリスト

### Phase 1: 準備
- [ ] Repository Interface確認・修正
- [ ] DTO変換ロジック実装
- [ ] Use Case型名統一確認

### Phase 2: DIContainer
- [ ] DIContainer構造体拡張
- [ ] Repository初期化追加
- [ ] UseCase初期化追加
- [ ] Controller初期化追加

### Phase 3: API Handler
- [ ] create_account_handler実装
- [ ] get_account_handler実装
- [ ] update_account_handler実装
- [ ] delete_account_handler実装
- [ ] get_account_balance_handler実装
- [ ] credit_account_handler実装
- [ ] debit_account_handler実装
- [ ] create_payment_handler実装
- [ ] get_payment_handler実装
- [ ] process_payment_handler実装
- [ ] refund_payment_handler実装
- [ ] list_payments_handler実装
- [ ] get_payment_stats_handler実装

### Phase 4: テスト
- [ ] コンパイルテスト通過
- [ ] 単体テスト通過
- [ ] 統合テスト通過
- [ ] APIエンドポイントテスト通過

---

## 🎯 完了後の状態

### 実現されること
- ✅ 13個の金融系APIエンドポイントが完全動作
- ✅ MySQL/Redisでの永続化
- ✅ 適切なエラーハンドリング
- ✅ ビジネスロジック実行
- ✅ 既存テストスイートとの統合

### API利用例
```bash
# アカウント作成
POST /api/account → 実際のDBに保存

# 決済作成  
POST /api/payment → 実際の決済処理実行

# 決済処理
PUT /api/payment/123/process → ステータス更新

# 残高操作
POST /api/account/123/credit → DB残高更新
```

---

## 📚 参考資料

- [Clean Architecture実装パターン](./CLAUDE.md#architecture)
- [CQRS実装ガイド](./CLAUDE.md#cqrs--distributed-storage)
- [テスト実装レビュー](./tests/)
- [finance-api.md計画書](./finance-api.md)

## 🚀 次のステップ

このガイドに従って実装完了後：
1. パフォーマンステスト実行
2. セキュリティテスト追加
3. 監査ログ機能統合
4. API認証・認可実装

---

**実装推定時間: 4-6時間**  
**難易度: 中級（型システムとDI理解が必要）**  
**優先度: 最高（金融API動作に必須）**