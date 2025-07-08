//domain/value_object/birth_date.rs
// BirthDate バリューオブジェクト
// 2025/7/8

use crate::shared::error::domain_error::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq)]
pub struct BirthDate(pub String);

impl BirthDate {
    pub fn new(value: String) -> DomainResult<Self> {
        // 簡易な日付フォーマットチェック（YYYY-MM-DD）
        if !value.chars().all(|c| c.is_ascii() || c == '-') || value.len() != 10 {
            return Err(DomainError::EntityValidationFailed {
                entity: "User".to_string(),
                field: "birth_date".to_string(),
                message: "Invalid birth date format".to_string(),
            });
        }
        Ok(Self(value))
    }
}
