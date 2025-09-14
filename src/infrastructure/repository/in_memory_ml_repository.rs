// src/infrastructure/repository/in_memory_ml_repository.rs

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::ml_entity::model::{Model, ModelFormat, ModelStatus};
use crate::domain::ml_repository::model_repository::ModelRepository;
use crate::shared::error::domain_error::DomainError;

#[derive(Clone, Debug)]
pub struct InMemoryModelRepository {
    models: Arc<HashMap<Uuid, Model>>,
}

impl InMemoryModelRepository {
    pub fn new() -> Self {
        let mut models = HashMap::new();

        // Pre-populate with the MNIST model info for testing purposes.
        let mnist_model_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let mut mnist_model = Model::new(
            "MNIST Classifier".to_string(),
            "1.0".to_string(),
            ModelFormat::Safetensors,
            "models/mnist.safetensors".to_string(), // Path relative to project root
            Some("A simple MLP model for classifying MNIST digits.".to_string()),
        );
        // Overwrite the randomly generated ID with a fixed one for predictable testing.
        mnist_model.id = mnist_model_id;
        // Since this is a pre-loaded, ready-to-use model, set its status to Active.
        mnist_model.status = ModelStatus::Active;


        models.insert(mnist_model_id, mnist_model);

        Self {
            models: Arc::new(models),
        }
    }
}

#[async_trait]
impl ModelRepository for InMemoryModelRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Model>, DomainError> {
        // Clone the model if found, simulating a database read.
        let model = self.models.get(&id).cloned();
        Ok(model)
    }
}

impl Default for InMemoryModelRepository {
    fn default() -> Self {
        Self::new()
    }
}
