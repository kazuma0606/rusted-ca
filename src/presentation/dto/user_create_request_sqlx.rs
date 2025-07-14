use crate::domain::entity_sqlx::user_sqlx::{UserRole, UserStatus};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserCreateRequestSqlx {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub phone: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
}
