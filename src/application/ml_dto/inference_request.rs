// src/application/ml_dto/inference_request.rs

use uuid::Uuid;

// This DTO carries the necessary data for an inference request.
pub struct InferenceRequest {
    pub model_id: Uuid,
    // For MNIST, this will be a flattened array of 784 (28*28) pixel values.
    pub input_data: Vec<f32>,
}
