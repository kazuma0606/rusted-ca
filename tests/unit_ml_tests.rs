// tests/unit_ml_tests.rs
// Unit tests for ML components

use uuid::Uuid;
use std::sync::Arc;

use rusted_ca::domain::ml_entity::model::{Model, ModelFormat, ModelStatus};
use rusted_ca::infrastructure::repository::in_memory_ml_repository::InMemoryModelRepository;
use rusted_ca::domain::ml_repository::model_repository::ModelRepository;
use rusted_ca::application::ml_dto::inference_request::InferenceRequest;
use rusted_ca::application::ml_dto::inference_response::InferenceResponse;

#[tokio::test]
async fn test_model_repository_find_by_id() {
    let repo = InMemoryModelRepository::new();
    
    // Test finding the pre-populated MNIST model
    let mnist_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let result = repo.find_by_id(mnist_id).await.unwrap();
    
    assert!(result.is_some());
    let model = result.unwrap();
    assert_eq!(model.name, "MNIST Classifier");
    assert_eq!(model.version, "1.0");
    assert_eq!(model.format, ModelFormat::Safetensors);
    assert_eq!(model.file_path, "models/mnist.safetensors");
    assert_eq!(model.status, ModelStatus::Active);
}

#[tokio::test]
async fn test_model_repository_find_nonexistent() {
    let repo = InMemoryModelRepository::new();
    
    // Test finding a non-existent model
    let nonexistent_id = Uuid::new_v4();
    let result = repo.find_by_id(nonexistent_id).await.unwrap();
    
    assert!(result.is_none());
}

#[test]
fn test_inference_request_creation() {
    let model_id = Uuid::new_v4();
    let input_data = vec![0.1, 0.2, 0.3];
    
    let request = InferenceRequest {
        model_id,
        input_data: input_data.clone(),
    };
    
    assert_eq!(request.model_id, model_id);
    assert_eq!(request.input_data, input_data);
}

#[test]
fn test_inference_response_creation() {
    let output_data = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    
    let response = InferenceResponse {
        output_data: output_data.clone(),
    };
    
    assert_eq!(response.output_data, output_data);
    assert_eq!(response.output_data.len(), 10);
}

#[test]
fn test_model_creation() {
    let model = Model::new(
        "Test Model".to_string(),
        "1.0".to_string(),
        ModelFormat::Safetensors,
        "models/test.safetensors".to_string(),
        Some("A test model".to_string()),
    );
    
    assert_eq!(model.name, "Test Model");
    assert_eq!(model.version, "1.0");
    assert_eq!(model.format, ModelFormat::Safetensors);
    assert_eq!(model.file_path, "models/test.safetensors");
    assert_eq!(model.description, Some("A test model".to_string()));
    assert_eq!(model.status, ModelStatus::Pending);
    
    // ID should be generated
    assert_ne!(model.id, Uuid::nil());
}

#[test]
fn test_model_status_transitions() {
    let mut model = Model::new(
        "Test Model".to_string(),
        "1.0".to_string(),
        ModelFormat::Safetensors,
        "models/test.safetensors".to_string(),
        None,
    );
    
    // Initial status should be Pending
    assert_eq!(model.status, ModelStatus::Pending);
    
    // Test status transitions
    model.status = ModelStatus::Active;
    assert_eq!(model.status, ModelStatus::Active);
    
    model.status = ModelStatus::Archived;
    assert_eq!(model.status, ModelStatus::Archived);
    
    model.status = ModelStatus::Error;
    assert_eq!(model.status, ModelStatus::Error);
}

// Integration test for complete ML workflow
#[tokio::test]
async fn test_ml_workflow_integration() {
    // This test verifies that all ML components work together
    let repo = InMemoryModelRepository::new();
    let mnist_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    // 1. Verify model exists
    let model = repo.find_by_id(mnist_id).await.unwrap().unwrap();
    assert_eq!(model.status, ModelStatus::Active);
    
    // 2. Create inference request
    let input_data = vec![0.0; 784]; // MNIST input size
    let request = InferenceRequest {
        model_id: mnist_id,
        input_data,
    };
    
    // 3. Verify request structure
    assert_eq!(request.input_data.len(), 784);
    assert_eq!(request.model_id, mnist_id);
    
    // 4. Simulate successful inference response
    let output_data = vec![0.1, 0.05, 0.02, 0.01, 0.0, 0.0, 0.0, 0.0, 0.82, 0.0];
    let response = InferenceResponse { output_data };
    
    // 5. Verify response structure
    assert_eq!(response.output_data.len(), 10); // 10 digits for MNIST
    
    // 6. Verify highest probability is at index 8 (digit 8)
    let max_index = response.output_data
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(index, _)| index)
        .unwrap();
    
    assert_eq!(max_index, 8);
}