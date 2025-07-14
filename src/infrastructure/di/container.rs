//infrastructure/di/container.rs
// DIコンテナ - CQRS対応
// 2025/7/8

use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecase;
use crate::application::usecases::delete_user_sqlx_usecase::DeleteUserSqlxUsecase;
use crate::application::usecases::update_user_sqlx_usecase::UpdateUserSqlxUsecase;

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
    pub create_user_usecase: CreateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub update_user_usecase: UpdateUserSqlxUsecase<
        SyncUserSqlxRepository<TiDBUserSqlxRepository, RedisUserSqlxRepository>,
    >,
    pub delete_user_usecase: DeleteUserSqlxUsecase<
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
        let uuid_gen = UuidGenerator;
        let id_gen: Box<dyn Fn() -> String + Send + Sync> = Box::new(move || uuid_gen.generate());
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
        Self {
            create_user_usecase,
            update_user_usecase,
            delete_user_usecase,
        }
    }
}
