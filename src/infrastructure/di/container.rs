//infrastructure/di/container.rs
// DIコンテナ - CQRS対応
// 2025/7/8

use std::sync::Arc;

use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecase;
use crate::application::usecases::delete_user_sqlx_usecase::DeleteUserSqlxUsecase;
use crate::application::usecases::update_user_sqlx_usecase::UpdateUserSqlxUsecase;
use crate::application::usecases::logging::{
    collect_log_usecase::CollectLogUsecase,
    search_logs_usecase::SearchLogsUsecase,
    manage_log_usecase::ManageLogUsecase,
};
use crate::presentation::controller::log_controller::LogController;

use crate::domain::repository::log_repository::LogRepositoryInterface;
use crate::infrastructure::repository::{
    redis_user_sqlx_repository::RedisUserSqlxRepository,
    sync_user_sqlx_repository::SyncUserSqlxRepository,
    tidb_user_sqlx_repository::TiDBUserSqlxRepository,
};
use crate::infrastructure::logging::mongodb_log_repository::MongoDbLogRepository;
use crate::shared::utils::password_hasher;
use crate::shared::utils::uuid_generator::{IdGeneratorInterface as SharedIdGeneratorInterface, UuidGenerator};
use crate::shared::metrics::collector::{MetricsCollectorInterface, MetricsCollector};
use crate::shared::error::infrastructure_error::InfrastructureError;
use deadpool_redis::Pool;
use sqlx::MySqlPool;
use mongodb::Client;
pub struct DIContainer {
    // 既存のユーザー管理
    pub create_user_usecase: CreateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub update_user_usecase: UpdateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub delete_user_usecase: DeleteUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,

    // 新規追加：MongoDB関連
    pub mongodb_client: Client,
    pub log_repository: Arc<dyn LogRepositoryInterface>,
    
    // 新規追加：ログ関連ユースケース
    pub collect_log_usecase: Arc<CollectLogUsecase>,
    pub search_logs_usecase: Arc<SearchLogsUsecase>,
    pub manage_log_usecase: Arc<ManageLogUsecase>,
    
    // 新規追加：共通ユーティリティ
    pub uuid_generator: Arc<UuidGenerator>,
    
    // 新規追加：ログコントローラー
    pub log_controller: Arc<LogController>,
}

impl DIContainer {
    pub async fn new(
        tidb_pool: MySqlPool, 
        redis_pool: Pool,
    ) -> Result<Self, InfrastructureError> {
        // 既存のユーザー管理システム初期化
        let tidb_repo = TiDBUserSqlxRepository { pool: tidb_pool };
        let redis_repo = RedisUserSqlxRepository { pool: redis_pool };
        let sync_repo = SyncUserSqlxRepository {
            tidb: tidb_repo,
            redis: redis_repo,
        };
        let uuid_gen = UuidGenerator::new();
        let id_gen: Box<dyn Fn() -> String + Send + Sync> = Box::new(move || {
            use crate::shared::utils::uuid_generator::IdGeneratorInterface;
            uuid_gen.generate()
        });
        let pass_hasher: Box<
            dyn Fn(&str) -> crate::shared::error::infrastructure_error::PasswordHasherResult<String>
                + Send
                + Sync,
        > = Box::new(password_hasher::encode);

        let create_user_usecase = CreateUserSqlxUsecase {
            repository: sync_repo.clone(),
            id_generator: id_gen,
            password_hasher: pass_hasher,
        };
        let update_user_usecase = UpdateUserSqlxUsecase {
            repository: sync_repo.clone(),
        };
        let delete_user_usecase = DeleteUserSqlxUsecase {
            repository: sync_repo,
        };

        // MongoDB接続初期化
        let mongodb_client = Client::with_uri_str("mongodb://admin:admin123@localhost:27017/")
            .await
            .map_err(|e| InfrastructureError::DatabaseConnection { message: e.to_string() })?;

        // ログリポジトリ初期化
        let log_repository = Arc::new(
            MongoDbLogRepository::new(
                mongodb_client.clone(),
                "rusted_ca_logs",
                "log_entries"
            ).await?
        ) as Arc<dyn LogRepositoryInterface>;

        // 共通ユーティリティ初期化
        let uuid_generator = Arc::new(UuidGenerator::new());

        // メトリクス収集器初期化
        let metrics_collector = Arc::new(MetricsCollector::new()) as Arc<dyn MetricsCollectorInterface>;

        // IDジェネレーター（ログ用）初期化
        let log_id_generator = Arc::new(crate::application::usecases::logging::collect_log_usecase::UuidGenerator::new()) as Arc<dyn crate::application::usecases::logging::collect_log_usecase::IdGeneratorInterface>;

        // ログ関連ユースケース初期化
        let collect_log_usecase = Arc::new(CollectLogUsecase::new(
            log_repository.clone(),
            metrics_collector,
            log_id_generator,
        ));

        let search_logs_usecase = Arc::new(SearchLogsUsecase::new(
            log_repository.clone(),
        ));

        let manage_log_usecase = Arc::new(ManageLogUsecase::new(
            log_repository.clone(),
        ));

        // ログコントローラー初期化
        let log_controller = Arc::new(LogController::new(
            search_logs_usecase.clone(),
            manage_log_usecase.clone(),
            collect_log_usecase.clone(),
        ));

        Ok(Self {
            create_user_usecase,
            update_user_usecase,
            delete_user_usecase,
            mongodb_client,
            log_repository,
            collect_log_usecase,
            search_logs_usecase,
            manage_log_usecase,
            uuid_generator,
            log_controller,
        })
    }
}
