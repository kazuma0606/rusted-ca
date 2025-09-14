// src/application/ml_usecases/inference_usecase.rs

use std::sync::Arc;
use async_trait::async_trait;

use crate::application::ml_dto::inference_request::InferenceRequest;
use crate::application::ml_dto::inference_response::InferenceResponse;
use crate::domain::ml_repository::model_repository::ModelRepository;
use crate::infrastructure::ml_engine::candle_engine::CandleEngine;
use crate::infrastructure::logging::log_collector_service::LogCollectorService;
use crate::shared::error::application_error::ApplicationError;

#[async_trait]
pub trait InferenceUsecase: Send + Sync {
    async fn execute(&self, request: InferenceRequest) -> Result<InferenceResponse, ApplicationError>;
}

pub struct InferenceUsecaseImpl {
    candle_engine: Arc<CandleEngine>,
    model_repository: Arc<dyn ModelRepository>,
    log_collector: Arc<tokio::sync::Mutex<LogCollectorService>>,
}

impl InferenceUsecaseImpl {
    pub fn new(
        candle_engine: Arc<CandleEngine>,
        model_repository: Arc<dyn ModelRepository>,
        log_collector: Arc<tokio::sync::Mutex<LogCollectorService>>,
    ) -> Self {
        Self {
            candle_engine,
            model_repository,
            log_collector,
        }
    }
}

#[async_trait]
impl InferenceUsecase for InferenceUsecaseImpl {
    async fn execute(&self, request: InferenceRequest) -> Result<InferenceResponse, ApplicationError> {
        // Log the start of inference operation
        {
            let log_collector = self.log_collector.lock().await;
            log_collector.log_ml_operation(
                &request.model_id.to_string(),
                "inference_start",
                &format!("Starting inference for model: {}", request.model_id)
            ).await;
        }

        // 1. Find model metadata from repository
        let model_metadata = match self.model_repository.find_by_id(request.model_id).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error_msg = format!("Model not found: {}", request.model_id);
                // Log error
                {
                    let log_collector = self.log_collector.lock().await;
                    log_collector.log_ml_operation(
                        &request.model_id.to_string(),
                        "inference_error", 
                        &error_msg
                    ).await;
                }
                return Err(ApplicationError::NotFound { 
                    resource: "Model".to_string(), 
                    id: request.model_id.to_string() 
                });
            },
            Err(e) => {
                let error_msg = format!("Error accessing model repository: {:?}", e);
                {
                    let log_collector = self.log_collector.lock().await;
                    log_collector.log_ml_operation(
                        &request.model_id.to_string(),
                        "inference_error",
                        &error_msg
                    ).await;
                }
                return Err(ApplicationError::InternalError(error_msg));
            }
        };

        // 2. Convert input Vec<f32> to Tensor
        let input_tensor = match candle_core::Tensor::from_vec(
            request.input_data.clone(), 
            &[request.input_data.len()], 
            &candle_core::Device::Cpu
        ) {
            Ok(tensor) => tensor,
            Err(e) => {
                let error_msg = format!("Failed to create input tensor: {:?}", e);
                {
                    let log_collector = self.log_collector.lock().await;
                    log_collector.log_ml_operation(
                        &request.model_id.to_string(),
                        "inference_error",
                        &error_msg
                    ).await;
                }
                return Err(ApplicationError::InternalError(error_msg));
            }
        };

        // 3. Run inference using Candle engine
        let output_tensor = match self.candle_engine.run_inference(&model_metadata, &input_tensor).await {
            Ok(output) => output,
            Err(e) => {
                let error_msg = format!("Inference failed: {:?}", e);
                {
                    let log_collector = self.log_collector.lock().await;
                    log_collector.log_ml_operation(
                        &request.model_id.to_string(),
                        "inference_error",
                        &error_msg
                    ).await;
                }
                return Err(ApplicationError::InternalError(error_msg));
            }
        };

        // 4. Convert output Tensor to Vec<f32>
        let output_data = match output_tensor.to_vec1::<f32>() {
            Ok(vec) => vec,
            Err(e) => {
                let error_msg = format!("Failed to convert output tensor to vec: {:?}", e);
                {
                    let log_collector = self.log_collector.lock().await;
                    log_collector.log_ml_operation(
                        &request.model_id.to_string(),
                        "inference_error",
                        &error_msg
                    ).await;
                }
                return Err(ApplicationError::InternalError(error_msg));
            }
        };

        // Log successful inference
        {
            let log_collector = self.log_collector.lock().await;
            log_collector.log_ml_operation(
                &request.model_id.to_string(),
                "inference_success",
                &format!("Inference completed successfully. Output length: {}", output_data.len())
            ).await;
        }

        Ok(InferenceResponse { output_data })
    }
}
