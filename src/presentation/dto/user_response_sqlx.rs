use crate::domain::entity_sqlx::user_sqlx::{UserRole, UserSqlx, UserStatus};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserResponseSqlx {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<UserSqlx> for UserResponseSqlx {
    fn from(user: UserSqlx) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            status: user.status,
            phone: user.phone,
            birth_date: user.birth_date,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
