use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use axum::extract::State;

use crate::{auth::jwt::{verify_token, JwtConfig}, utils::errors::AppError, AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub role: String,
    pub institution_id: String,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

    let claims = verify_token(token, &state.jwt_config)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    let auth_user = AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
        institution_id: claims.institution_id,
    };

    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}

/// Macro-like helper: guard a handler to specific roles
pub async fn require_roles(
    auth_user: &AuthUser,
    allowed: &[&str],
) -> Result<(), AppError> {
    if allowed.contains(&auth_user.role.as_str()) {
        Ok(())
    } else {
        Err(AppError::Forbidden("Insufficient permissions".to_string()))
    }
}
