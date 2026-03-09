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

#[derive(Debug, Deserialize)]
pub struct AttendanceRecord {
    pub student_id: String,
    pub status: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MarkAttendanceRequest {
    pub course_id: String,
    pub topic: Option<String>,
    pub records: Vec<AttendanceRecord>,
}

pub async fn mark_attendance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<MarkAttendanceRequest>,
) -> AppResult<impl IntoResponse> {
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;
    let course_uuid = Uuid::parse_str(&body.course_id)
        .map_err(|_| AppError::NotFound("Course not found".into()))?;

    // Get faculty id
    let fac_row = sqlx::query(r#"SELECT id FROM "Faculty" WHERE user_id = $1"#)
        .bind(uid)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a faculty member".into()))?;
    let faculty_id: Uuid = fac_row.try_get("id")?;

    // Create session
    let sess_row = sqlx::query(
        r#"INSERT INTO "AttendanceSession" (course_id, faculty_id, topic)
           VALUES ($1, $2, $3) RETURNING id"#,
    )
    .bind(course_uuid)
    .bind(faculty_id)
    .bind(&body.topic)
    .fetch_one(&state.db)
    .await?;
    let session_id: Uuid = sess_row.try_get("id")?;

    // Insert records
    for record in &body.records {
        if let Ok(student_uuid) = Uuid::parse_str(&record.student_id) {
            let _ = sqlx::query(
                r#"INSERT INTO "Attendance" (session_id, student_id, status, remarks)
                   VALUES ($1, $2, $3::\"AttendanceStatus\", $4)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(session_id)
            .bind(student_uuid)
            .bind(&record.status)
            .bind(&record.remarks)
            .execute(&state.db)
            .await;
        }
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({"session_id": session_id.to_string()}))))
}

pub async fn get_student_attendance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(student_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let student_uuid = Uuid::parse_str(&student_id)
        .map_err(|_| AppError::NotFound("Student not found".into()))?;

    let rows = sqlx::query(
        r#"SELECT sess.id::text AS session_id, c.name AS course_name,
                  sess.session_date::text, a.status::text, a.remarks
           FROM "Attendance" a
           JOIN "AttendanceSession" sess ON sess.id = a.session_id
           JOIN "Course" c ON c.id = sess.course_id
           WHERE a.student_id = $1
           ORDER BY sess.session_date DESC"#,
    )
    .bind(student_uuid)
    .fetch_all(&state.db)
    .await?;

    let data: Vec<_> = rows.into_iter().map(|r| serde_json::json!({
        "session_id": r.try_get::<String, _>("session_id").unwrap_or_default(),
        "course_name": r.try_get::<String, _>("course_name").unwrap_or_default(),
        "session_date": r.try_get::<String, _>("session_date").unwrap_or_default(),
        "status": r.try_get::<String, _>("status").unwrap_or_default(),
        "remarks": r.try_get::<Option<String>, _>("remarks").unwrap_or_default(),
    })).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}
