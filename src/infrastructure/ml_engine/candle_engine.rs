// src/infrastructure/ml_engine/candle_engine.rs

use candle_core::{Device, Error as CandleError, Tensor};
use candle_nn::{self as nn, Module, VarBuilder};

use crate::domain::ml_entity::model::Model as ModelMetadata;

// Define the MNIST model architecture.
// This is a simple two-layer linear network, matching the pre-trained model.
struct MnistModel {
    linear1: nn::Linear,
    linear2: nn::Linear,
}

impl MnistModel {
    fn new(vs: VarBuilder) -> Result<Self, CandleError> {
        let linear1 = nn::linear(28 * 28, 100, vs.pp("l1"))?;
        let linear2 = nn::linear(100, 10, vs.pp("l2"))?;
        Ok(Self { linear1, linear2 })
    }

    fn forward(&self, xs: &Tensor) -> Result<Tensor, CandleError> {
        let xs = self.linear1.forward(xs)?;
        let xs = xs.relu()?;
        self.linear2.forward(&xs)
    }
}

/// A struct responsible for running ML inference using the Candle framework.
#[derive(Debug, Clone)]
pub struct CandleEngine {
    device: Device,
}

impl CandleEngine {
    /// Creates a new instance of the CandleEngine.
    /// This might be expanded in the future to select devices (CPU/GPU).
    pub fn new() -> Result<Self, CandleError> {
        // For now, we default to CPU to ensure it runs on any laptop.
        let device = Device::Cpu;
        Ok(Self { device })
    }

    /// Runs inference using a specified model and input tensor.
    ///
    /// # Arguments
    ///
    /// * `model_meta`: The metadata of the model to use for inference.
    /// * `input`: The input tensor for the model.
    ///
    /// # Returns
    ///
    /// A `Result` containing the output `Tensor` on success, or a `CandleError` on failure.
    pub async fn run_inference(
        &self,
        model_meta: &ModelMetadata,
        input: &Tensor,
    ) -> Result<Tensor, CandleError> {
        // 1. Create a VarBuilder to load the weights from the specified .safetensors file.
        //    The `unsafe` block is used for performance, as it memory-maps the file,
        //    avoiding loading the entire file into memory at once.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[&model_meta.file_path],
                candle_core::DType::F32,
                &self.device,
            )?
        };

        // 2. Build the model architecture with the loaded weights.
        let model = MnistModel::new(vb)?;

        // 3. Ensure the input tensor is on the correct device.
        let input = input.to_device(&self.device)?;

        // 4. Perform the forward pass (the actual inference).
        let output = model.forward(&input)?;

        Ok(output)
    }
}

impl Default for CandleEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize CandleEngine with default CPU settings")
    }
}