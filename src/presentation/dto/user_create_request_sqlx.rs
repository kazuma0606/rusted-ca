use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserCreateRequestSqlx {
    pub email: String,
    pub name: String,
    pub password: String,
    pub phone: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
}
