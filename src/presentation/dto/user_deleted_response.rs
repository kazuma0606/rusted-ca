use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserDeletedResponse {
    pub email: String,
    pub name: String,
    pub status: String,
}

impl UserDeletedResponse {
    pub fn new(email: String, name: String) -> Self {
        Self {
            email,
            name,
            status: "deleted".to_string(),
        }
    }
}
