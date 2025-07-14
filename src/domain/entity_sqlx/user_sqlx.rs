use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR")] // DBはVARCHAR
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserRole::Admin => "ADMIN",
            UserRole::User => "USER",
            UserRole::Guest => "GUEST",
        };
        write!(f, "{}", s)
    }
}
impl FromStr for UserRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADMIN" => Ok(UserRole::Admin),
            "USER" => Ok(UserRole::User),
            "GUEST" => Ok(UserRole::Guest),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR")] // DBはVARCHAR
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserStatus::Active => "ACTIVE",
            UserStatus::Suspended => "SUSPENDED",
            UserStatus::Deleted => "DELETED",
        };
        write!(f, "{}", s)
    }
}
impl FromStr for UserStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(UserStatus::Active),
            "SUSPENDED" => Ok(UserStatus::Suspended),
            "DELETED" => Ok(UserStatus::Deleted),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSqlx {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
