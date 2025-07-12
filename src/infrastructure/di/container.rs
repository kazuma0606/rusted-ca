//infrastructure/di/container.rs
// DIコンテナ - CQRS対応
// 2025/7/8

use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecase;
use crate::infrastructure::repository::{
    redis_user_sqlx_repository::RedisUserSqlxRepository,
    sync_user_sqlx_repository::SyncUserSqlxRepository,
    tidb_user_sqlx_repository::TiDBUserSqlxRepository,
};
use deadpool_redis::Pool;
use sqlx::MySqlPool;

pub struct DIContainer {
    pub create_user_usecase: CreateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
}

impl DIContainer {
    pub fn new(tidb_pool: MySqlPool, redis_pool: Pool) -> Self {
        let tidb_repo = TiDBUserSqlxRepository { pool: tidb_pool };
        let redis_repo = RedisUserSqlxRepository { pool: redis_pool };
        let sync_repo = SyncUserSqlxRepository {
            tidb: tidb_repo,
            redis: redis_repo,
        };
        let create_user_usecase = CreateUserSqlxUsecase {
            repository: sync_repo,
        };
        Self {
            create_user_usecase,
        }
    }
}
