//infrastructure/di/container.rs
// DIコンテナ - CQRS対応
// 2025/7/8

use std::sync::Arc;

use crate::application::ml_usecases::inference_usecase::{
    InferenceUsecase, InferenceUsecaseImpl,
};
use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecase;
use crate::application::usecases::delete_user_sqlx_usecase::DeleteUserSqlxUsecase;
use crate::application::usecases::update_user_sqlx_usecase::UpdateUserSqlxUsecase;
use crate::domain::ml_repository::model_repository::ModelRepository;
use crate::infrastructure::logging::log_collector_service::LogCollectorService;
use crate::infrastructure::ml_engine::candle_engine::CandleEngine;
use crate::infrastructure::repository::in_memory_ml_repository::InMemoryModelRepository;
use crate::infrastructure::repository::{
    redis_user_sqlx_repository::RedisUserSqlxRepository,
    sync_user_sqlx_repository::SyncUserSqlxRepository,
    tidb_user_sqlx_repository::TiDBUserSqlxRepository,
};
use crate::shared::utils::password_hasher;
use crate::shared::utils::uuid_generator::IdGeneratorInterface;
use crate::shared::utils::uuid_generator::UuidGenerator;
use deadpool_redis::Pool;
use sqlx::MySqlPool;

pub struct DIContainer {
    // User use cases
    pub create_user_usecase: CreateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub update_user_usecase: UpdateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub delete_user_usecase: DeleteUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,

    // ML use cases
    pub inference_usecase: Arc<dyn InferenceUsecase>,
}

impl DIContainer {
    pub fn new(
        tidb_pool: MySqlPool,
        redis_pool: Pool,
        log_collector: Arc<tokio::sync::Mutex<LogCollectorService>>,
    ) -> Self {
        // --- Repositories ---
        let tidb_repo = TiDBUserSqlxRepository { pool: tidb_pool };
        let redis_repo = RedisUserSqlxRepository { pool: redis_pool };
        let sync_repo = SyncUserSqlxRepository {
            tidb: tidb_repo,
            redis: redis_repo,
        };
        let model_repository: Arc<dyn ModelRepository> = Arc::new(InMemoryModelRepository::new());

        // --- Engines & Services ---
        let candle_engine = Arc::new(CandleEngine::new().expect("Failed to create CandleEngine"));

        // --- Utilities ---
        let uuid_gen = UuidGenerator;
        let id_gen: Box<dyn Fn() -> String + Send + Sync> = Box::new(move || uuid_gen.generate());
        let pass_hasher: Box<
            dyn Fn(&str) -> crate::shared::error::infrastructure_error::PasswordHasherResult<String>
                + Send
                + Sync,
        > = Box::new(password_hasher::encode);

        // --- Use Cases ---
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
        let inference_usecase: Arc<dyn InferenceUsecase> = Arc::new(InferenceUsecaseImpl::new(
            candle_engine,
            model_repository,
            log_collector,
        ));

        Self {
            create_user_usecase,
            update_user_usecase,
            delete_user_usecase,
            inference_usecase,
        }
    }
}
