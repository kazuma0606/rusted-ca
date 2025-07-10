//presentation/controller/auth_controller.rs
use crate::presentation::dto::login_request::LoginRequest;
use crate::presentation::dto::login_response::LoginResponse;
use crate::shared::middleware::auth_middleware::{AuthError, JwtClaims, JwtConfig, JwtService};
use axum::{Json, http::StatusCode};
use dotenvy::dotenv;
use std::env;

#[derive(Debug, serde::Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UserLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

pub async fn login(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<UserLoginResponse>, AuthError> {
    dotenv().ok();
    let auth_user = env::var("AUTH_USER").unwrap_or_else(|_| "auth_user".to_string());
    let auth_pass = env::var("AUTH_PASS").unwrap_or_else(|_| "auth_password".to_string());

    if payload.username != auth_user || payload.password != auth_pass {
        return Err(AuthError::WrongCredentials);
    }

    // ダミー情報でJwtClaimsを生成（必要に応じてemail/name/roleを.envやDBから取得）
    let user_id = "dummy-id".to_string();
    let name = "Dummy User".to_string();
    let role = "user".to_string();

    let jwt_service = JwtService::new();
    let (access_token, refresh_token) = jwt_service.generate_token_pair(
        user_id.clone(),
        payload.username.clone(),
        name.clone(),
        role.clone(),
    )?;

    Ok(Json(UserLoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: UserInfo {
            id: user_id,
            email: payload.username,
            name,
            role,
        },
    }))
}
