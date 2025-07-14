use crate::domain::entity_sqlx::user_sqlx::{UserRole, UserSqlx, UserStatus};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserDeletedResponse {
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub status: UserStatus,
}

impl UserDeletedResponse {
    pub fn from_user(user: &UserSqlx) -> Self {
        Self {
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            status: UserStatus::Deleted,
        }
    }
}
