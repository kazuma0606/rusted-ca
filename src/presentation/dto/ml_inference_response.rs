// src/presentation/dto/ml_inference_response.rs

use serde::Serialize;

#[derive(Serialize)]
pub struct MLInferenceResponse {
    // For MNIST, this will be the 10 probabilities for each digit.
    pub output_data: Vec<f32>,
}
