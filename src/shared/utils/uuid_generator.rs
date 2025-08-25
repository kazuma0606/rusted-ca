//shared/utils/uuid_generator.rs
// idGenerator
// 2025/7/8

use crate::domain::value_object::{log_id::LogId, request_id::RequestId};

pub trait IdGeneratorInterface {
    fn generate(&self) -> String;
}

pub struct UuidGenerator;

impl IdGeneratorInterface for UuidGenerator {
    fn generate(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl UuidGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_log_id(&self) -> LogId {
        LogId::generate()
    }

    pub fn generate_request_id(&self) -> RequestId {
        RequestId::generate()
    }
}
