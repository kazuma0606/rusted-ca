# 🦀 True Rusted Clean Architecture (真のRusted CA)

## 🎯 設計哲学

**「Rustの特性を活かし、CAの美しさを最大化する」**

- ✅ **Lightning Clean Commerce の成功パターン**を基盤とする
- ✅ **In-Memory CQRS**でCAの美しさに集中
- ✅ **層別Result型**でエラー発生源を明確化
- ✅ **ジェネリック制約**による型安全性の徹底
- ✅ **Value Objects**によるドメインルール実装

## 🏗️ アーキテクチャ全体図

```
┌─────────────────────────────────────────────────────────────────┐
│                    Presentation Layer                           │
│  HTTP Request → Controller → PresentationResult<Response>       │
│  PresentationError (400, 401, 403, 404, 500)                  │
├─────────────────────────────────────────────────────────────────┤
│                   Application Layer                             │
│  Command/Query → UseCase → ApplicationResult<DTO>              │
│  ApplicationError (UserNotFound, EmailExists, Validation...)   │
├─────────────────────────────────────────────────────────────────┤
│                     Domain Layer                                │
│  Value Objects → Entity → DomainResult<Entity>                 │
│  DomainError (InvalidEmail, BusinessRuleViolation...)          │
├─────────────────────────────────────────────────────────────────┤
│                Infrastructure Layer                             │
│  In-Memory CQRS → Repository → InfrastructureResult<Data>      │
│  InfrastructureError (Database, Network, ExternalService...)   │
└─────────────────────────────────────────────────────────────────┘
```

## 🔄 CQRS Data Flow

### Command Side (Write)
```
HTTP Request → CreateUserRequest → CreateUserCommand 
    ↓
UseCase<T: CommandRepository, U: IdGenerator> 
    ↓
User Entity (Value Objects) → CommandRepository 
    ↓
In-Memory Normalized Store → UserResponse
```

### Query Side (Read)
```
HTTP Request → GetUserQuery 
    ↓
UseCase<T: QueryRepository> 
    ↓
QueryRepository → In-Memory Denormalized Store 
    ↓
UserResponseDto → HTTP Response
```

## 📋 完全な型定義

### 層別Result型
```rust
// Domain Layer
pub type DomainResult<T> = Result<T, DomainError>;

// Application Layer  
pub type ApplicationResult<T> = Result<T, ApplicationError>;

// Infrastructure Layer
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

// Presentation Layer
pub type PresentationResult<T> = Result<T, PresentationError>;
```

### エラー自動変換チェーン
```rust
DomainError → ApplicationError → PresentationError
InfrastructureError → ApplicationError → PresentationError
```

### ジェネリック制約パターン
```rust
impl<T, U, V> CreateUserUsecaseInterface for CreateUserUseCase<T, U, V>
where
    T: UserCommandRepositoryInterface + Send + Sync,
    U: IdGeneratorInterface + Send + Sync,
    V: EmailServiceInterface + Send + Sync,
```

## 🏛️ Layer Definitions

### Domain Layer
**責務**: ビジネスルールとドメインロジック
```rust
// Value Objects (ドメインルール)
Email(String)     // メールバリデーション
UserName(String)  // 名前バリデーション
Password { hash } // パスワードハッシュ化

// Entity (ビジネスルール集約)
User {
    id: Uuid,
    email: Email,
    name: UserName,
    password: Password,
    // + ビジネスメソッド
}

// Repository Trait (依存性逆転)
trait UserCommandRepositoryInterface
trait UserQueryRepositoryInterface
```

### Application Layer
**責務**: ユースケースとビジネスフロー
```rust
// Command/Query Objects
CreateUserCommand { email, name, password }
GetUserQuery { id }

// UseCase (ビジネスフロー)
CreateUserUseCase<T, U, V> {
    command_repository: Arc<T>,
    id_generator: Arc<U>,
    email_service: Arc<V>,
}

// DTO (層間データ転送)
CreateUserRequestDto → CreateUserResponseDto
```

### Infrastructure Layer
**責務**: 外部システムとの接続
```rust
// In-Memory CQRS Implementation
InMemoryUserCommandRepository {
    // Write最適化: 正規化データ
    store: HashMap<Uuid, User>,
    email_index: HashMap<String, Uuid>,
}

InMemoryUserQueryRepository {
    // Read最適化: 非正規化データ  
    read_models: HashMap<Uuid, UserReadModel>,
    search_index: HashMap<String, Vec<Uuid>>,
}
```

### Presentation Layer  
**責務**: HTTP処理とフォーマット変換
```rust
// HTTP DTO
CreateUserRequest { email, name, password }
UserResponse { id, email, name, created_at }

// Controller
async fn create_user<T: CreateUserUsecaseInterface>(
    State(usecase): State<Arc<T>>,
    Json(request): Json<CreateUserRequest>,
) -> PresentationResult<Json<ApiResponse<UserResponse>>>
```

## 🔧 DI Pattern (Lightning方式)

### 直接注入パターン
```rust
async fn setup_dependencies() -> AppState {
    // Infrastructure Layer
    let command_repository = Arc::new(InMemoryUserCommandRepository::new());
    let query_repository = Arc::new(InMemoryUserQueryRepository::new());
    let id_generator = Arc::new(UuidGenerator::new());
    
    // Application Layer - ジェネリック制約
    let create_user_usecase = Arc::new(CreateUserUseCase::new(
        command_repository.clone(),
        id_generator.clone(),
    ));
    
    let get_user_usecase = Arc::new(GetUserUseCase::new(
        query_repository.clone(),
    ));
    
    AppState {
        create_user_usecase,
        get_user_usecase,
    }
}
```

## 📐 SOLID Principles Implementation

### Single Responsibility Principle
- **Controller**: HTTP処理のみ
- **UseCase**: ビジネスフロー実行のみ  
- **Repository**: データアクセスのみ
- **Entity**: ドメインルールのみ

### Open/Closed Principle
- 新しいRepository実装追加可能
- 新しいUseCase追加可能
- 既存コード変更なしで拡張

### Liskov Substitution Principle
- Repository trait実装の差し替え可能
- TestでMockに差し替え可能

### Interface Segregation Principle
- CommandRepository / QueryRepository 分離
- UseCase別のInterface分離

### Dependency Inversion Principle
- Application → Domain ← Infrastructure
- 抽象に依存、具象に依存しない

## 🎨 Value Objects Pattern

### ドメインルール実装
```rust
impl Email {
    pub fn new(value: String) -> DomainResult<Self> {
        // バリデーションロジック
        // 正規表現チェック
        // ビジネスルール適用
    }
}

impl Password {
    pub fn from_plain(plain: String) -> DomainResult<Self> {
        // 強度チェック
        // bcryptハッシュ化
        // セキュリティルール適用
    }
}
```

## 🔄 Type Conversion Matrix

| From → To | Layer Boundary | Purpose |
|-----------|----------------|---------|
| `CreateUserRequest` → `CreateUserCommand` | Presentation → Application | HTTP境界分離 |
| `CreateUserCommand` → `User` | Application → Domain | ビジネスルール適用 |
| `User` → `UserReadModel` | Domain → Infrastructure | 永続化形式変換 |
| `UserReadModel` → `UserResponse` | Infrastructure → Presentation | HTTP出力変換 |

## 🧪 Testing Strategy

### Unit Tests
```rust
// Domain Layer Test
#[test]
fn email_new_should_validate_format() {
    assert_eq!(
        Email::new("invalid".to_string()),
        Err(DomainError::InvalidEmail { ... })
    );
}

// Application Layer Test (Mock)
#[tokio::test] 
async fn create_user_should_fail_when_email_exists() {
    let mock_repo = MockUserRepository::new();
    mock_repo.expect_find_by_email().returning(|_| Ok(true));
    
    let usecase = CreateUserUseCase::new(Arc::new(mock_repo));
    let result = usecase.execute(command).await;
    
    assert_eq!(result, Err(ApplicationError::EmailAlreadyExists { ... }));
}
```

### Integration Tests
```rust
// Full API Test
#[tokio::test]
async fn create_user_api_should_return_201() {
    let app = create_test_app().await;
    
    let response = app
        .post("/api/users")
        .json(&create_user_request())
        .await;
        
    assert_eq!(response.status(), 201);
    // Response validation
}
```

## 🚀 Implementation Phases

### Phase 1: Domain Foundation (45min)
1. **Error Types**: 層別Result型定義
2. **Value Objects**: Email, UserName, Password
3. **Entity**: User + ビジネスメソッド
4. **Repository Traits**: Command/Query分離

### Phase 2: Application Services (60min)
5. **Command/Query Objects**: CQRS型定義
6. **UseCase Implementation**: ジェネリック制約
7. **DTO Definitions**: 型変換実装
8. **Error Conversion**: 自動変換テスト

### Phase 3: Infrastructure (45min)
9. **In-Memory Command Repository**: Write最適化
10. **In-Memory Query Repository**: Read最適化  
11. **Test Data Seeding**: 初期データ投入
12. **CQRS Synchronization**: Read/Write同期

### Phase 4: Presentation (45min)
13. **HTTP DTOs**: Request/Response
14. **Controllers**: エラーハンドリング
15. **Router Setup**: エンドポイント定義
16. **Status Code Mapping**: HTTP応答

### Phase 5: Integration (30min)
17. **DI Setup**: Lightning方式
18. **End-to-End Test**: 全フロー確認
19. **API Documentation**: エンドポイント仕様
20. **Performance Validation**: 動作速度確認

**Total: 3.5 hours**

## ✨ Unique Features of Rusted CA

### 🦀 **Rustの特性活用**
- **所有権システム**: メモリ安全性
- **型システム**: コンパイル時エラー検出
- **ジェネリクス**: 型安全なDI
- **トレイト**: 適切な抽象化

### 🎨 **CAの美しさ**
- **In-Memory CQRS**: 純粋なアーキテクチャ
- **層別Result型**: エラー発生源明確化
- **Value Objects**: ドメインルール実装
- **依存性逆転**: 完全な層分離

### ⚡ **開発効率**
- **Lightning成功パターン**: 実証済みDI
- **段階的実装**: 確実な進行
- **即座の動作確認**: In-Memory利点
- **Future PostgreSQL**: 容易な移行

---

**これが真のRusted Clean Architecture！**  
**Rustの美しさとCAの理想が融合した、完璧な設計です！** 🎯🦀✨