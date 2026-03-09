use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    middleware::auth_guard::AuthUser,
    utils::{
        errors::AppResult,
        pagination::{PaginatedResponse, PaginationParams},
    },
    AppState,
};

pub async fn get_notifications(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<impl IntoResponse> {
    let offset = params.offset() as i64;
    let limit = params.limit_clamped() as i64;

    let rows = sqlx::query!(
        r#"
        SELECT id, type::text AS notification_type, title, message, is_read, link, created_at,
               COUNT(*) OVER () AS total_count
        FROM "Notification"
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        auth_user.id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total = rows.first().and_then(|r| r.total_count).unwrap_or(0) as u64;
    let notifications: Vec<_> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id,
        "type": r.notification_type,
        "title": r.title,
        "message": r.message,
        "is_read": r.is_read,
        "link": r.link,
        "created_at": r.created_at
    })).collect();

    Ok(Json(PaginatedResponse::new(notifications, total, &params)))
}

pub async fn mark_notifications_read(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    sqlx::query!(
        r#"UPDATE "Notification" SET is_read = true WHERE user_id = $1 AND is_read = false"#,
        auth_user.id
    )
    .execute(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"message": "All notifications marked as read"}))))
}

pub async fn get_unread_count(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM "Notification" WHERE user_id = $1 AND is_read = false"#,
        auth_user.id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    Ok(Json(serde_json::json!({"unread_count": count})))
}
