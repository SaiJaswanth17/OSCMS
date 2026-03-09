use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    middleware::auth_guard::AuthUser,
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct NotificationDto {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub is_read: bool,
    pub link: Option<String>,
    pub created_at: String,
}

pub async fn get_notifications(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;

    let rows = sqlx::query(
        r#"SELECT id, type::text AS notification_type, title, message,
                  is_read, link, created_at::text
           FROM "Notification"
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .bind(uid)
    .fetch_all(&state.db)
    .await?;

    let notifications: Vec<NotificationDto> = rows.into_iter().map(|r| NotificationDto {
        id: r.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        title: r.try_get("title").unwrap_or_default(),
        message: r.try_get("message").unwrap_or_default(),
        notification_type: r.try_get("notification_type").unwrap_or_default(),
        is_read: r.try_get("is_read").unwrap_or(false),
        link: r.try_get("link").unwrap_or_default(),
        created_at: r.try_get("created_at").unwrap_or_default(),
    }).collect();

    Ok(Json(serde_json::json!({ "data": notifications })))
}

pub async fn mark_notifications_read(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;

    sqlx::query(r#"UPDATE "Notification" SET is_read = true WHERE user_id = $1 AND is_read = false"#)
        .bind(uid)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"message": "All notifications marked as read"})))
}

pub async fn get_unread_count(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;

    let row = sqlx::query(
        r#"SELECT COUNT(*) AS count FROM "Notification" WHERE user_id = $1 AND is_read = false"#,
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    let count: i64 = row.try_get("count").unwrap_or(0);
    Ok(Json(serde_json::json!({"unread_count": count})))
}
