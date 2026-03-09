use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    auth::jwt::create_access_token,
    middleware::auth_guard::AuthUser,
    utils::{errors::{AppError, AppResult}, password::{hash_password, verify_password}},
    AppState,
};

// ─────────────────────────────────────────────
// Request / Response types
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub role: String,
    pub institution_id: String,
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

// ─────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Fetch user by email
    let row = sqlx::query!(
        r#"
        SELECT id, email, password_hash, role, first_name, last_name, 
               institution_id, avatar_url, is_active
        FROM "User"
        WHERE email = $1
        "#,
        body.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !row.is_active {
        return Err(AppError::Unauthorized("Account is deactivated".to_string()));
    }

    // Verify password
    let valid = verify_password(&body.password, &row.password_hash)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Password verification failed")))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    // Create JWT
    let token = create_access_token(
        &row.id,
        &row.email,
        &row.role,
        &row.institution_id,
        &state.jwt_config,
    )
    .map_err(|e| AppError::Internal(e))?;

    // Update last login
    sqlx::query!(
        r#"UPDATE "User" SET last_login_at = NOW() WHERE id = $1"#,
        row.id
    )
    .execute(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            user: UserInfo {
                id: row.id,
                email: row.email,
                first_name: row.first_name,
                last_name: row.last_name,
                role: row.role,
                institution_id: row.institution_id,
                avatar_url: row.avatar_url,
            },
        }),
    ))
}

pub async fn me(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let row = sqlx::query!(
        r#"
        SELECT id, email, first_name, last_name, role, institution_id, avatar_url
        FROM "User"
        WHERE id = $1 AND is_active = true
        "#,
        auth_user.id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserInfo {
        id: row.id,
        email: row.email,
        first_name: row.first_name,
        last_name: row.last_name,
        role: row.role,
        institution_id: row.institution_id,
        avatar_url: row.avatar_url,
    }))
}

pub async fn logout() -> impl IntoResponse {
    // Client-side: drop token. Server-side token revocation can be added here.
    (StatusCode::OK, Json(serde_json::json!({"message": "Logged out successfully"})))
}
