// src/domain/ml_repository/model_repository.rs

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::ml_entity::model::Model;
use crate::shared::error::domain_error::DomainError;

#[async_trait]
pub trait ModelRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Model>, DomainError>;
}
