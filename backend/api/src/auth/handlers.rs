use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

use crate::{
    auth::jwt::create_access_token,
    middleware::auth_guard::AuthUser,
    utils::{
        errors::{AppError, AppResult},
        password::verify_password,
    },
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub institution_id: String,
    pub avatar_url: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row = sqlx::query(
        r#"SELECT id, email, password_hash, role, first_name, last_name,
                  institution_id::text, avatar_url, is_active
           FROM "User" WHERE email = $1"#,
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    use sqlx::Row;
    let is_active: bool = row.try_get("is_active")?;
    if !is_active {
        return Err(AppError::Unauthorized("Account is deactivated".into()));
    }

    let password_hash: String = row.try_get("password_hash")?;
    let valid = verify_password(&body.password, &password_hash)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Password verification failed")))?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let id: Uuid = row.try_get("id")?;
    let email: String = row.try_get("email")?;
    let role: String = row.try_get("role")?;
    let institution_id: String = row.try_get("institution_id")?;
    let first_name: String = row.try_get("first_name")?;
    let last_name: String = row.try_get("last_name")?;
    let avatar_url: Option<String> = row.try_get("avatar_url")?;

    let token = create_access_token(&id.to_string(), &email, &role, &institution_id, &state.jwt_config)
        .map_err(AppError::Internal)?;

    sqlx::query(r#"UPDATE "User" SET last_login_at = NOW() WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok((StatusCode::OK, Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserInfo { id: id.to_string(), email, first_name, last_name, role, institution_id, avatar_url },
    })))
}

pub async fn me(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    use sqlx::Row;
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;

    let row = sqlx::query(
        r#"SELECT id, email, first_name, last_name, role,
                  institution_id::text, avatar_url
           FROM "User" WHERE id = $1 AND is_active = true"#,
    )
    .bind(uid)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(UserInfo {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        email: row.try_get("email")?,
        first_name: row.try_get("first_name")?,
        last_name: row.try_get("last_name")?,
        role: row.try_get("role")?,
        institution_id: row.try_get("institution_id")?,
        avatar_url: row.try_get("avatar_url")?,
    }))
}

pub async fn logout() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"message": "Logged out successfully"})))
}
