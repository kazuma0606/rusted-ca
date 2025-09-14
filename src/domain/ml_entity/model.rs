// src/domain/ml_entity/model.rs

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFormat {
    Safetensors,
    GGUF,
    Pickle,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelStatus {
    Pending,  // Validation or processing is pending
    Active,   // Ready for inference
    Archived, // Not actively used but kept for records
    Error,    // Something went wrong
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub format: ModelFormat,
    pub status: ModelStatus,
    pub file_path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Model {
    pub fn new(
        name: String,
        version: String,
        format: ModelFormat,
        file_path: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            format,
            status: ModelStatus::Pending,
            file_path,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
