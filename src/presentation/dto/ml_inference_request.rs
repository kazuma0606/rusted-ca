// src/presentation/dto/ml_inference_request.rs

use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct MLInferenceRequest {
    pub model_id: Uuid,
    // For MNIST, this will be a flattened array of 784 (28*28) pixel values.
    pub input_data: Vec<f32>,
}
