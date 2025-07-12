use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSqlx {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub phone: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
