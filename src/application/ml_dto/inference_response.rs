// src/application/ml_dto/inference_response.rs

// This DTO carries the result of an inference.
pub struct InferenceResponse {
    // For MNIST, this will be the 10 probabilities for each digit.
    pub output_data: Vec<f32>,
}
