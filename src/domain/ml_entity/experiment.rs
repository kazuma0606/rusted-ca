// src/domain/ml_entity/experiment.rs

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub model_id: Uuid,
    pub status: ExperimentStatus,
    pub parameters: Option<JsonValue>, // Flexible JSON for hyperparameters
    pub metrics: Option<JsonValue>,    // Flexible JSON for results (accuracy, loss, etc.)
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Experiment {
    pub fn new(name: String, model_id: Uuid, parameters: Option<JsonValue>, notes: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            model_id,
            status: ExperimentStatus::Running,
            parameters,
            metrics: None,
            notes,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}
