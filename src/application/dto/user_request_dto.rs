//application/dto/user_request_dto.rs
// HTTP Request DTO
// 2025/7/8

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequestDto {
    pub email: String,
    pub name: String,
    pub password: String,
    pub phone: Option<String>,
    pub birth_date: Option<String>,
}

impl CreateUserRequestDto {
    /// バリューオブジェクト変換＋バリデーション
    pub fn to_value_objects(&self) -> (String, String, String, Option<String>, Option<String>) {
        (
            self.email.clone(),
            self.name.clone(),
            self.password.clone(),
            self.phone.clone(),
            self.birth_date.clone(),
        )
    }
}
