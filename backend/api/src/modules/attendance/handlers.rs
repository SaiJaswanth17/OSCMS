use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

use crate::{
    middleware::auth_guard::{AuthUser, require_roles},
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct AttendanceSessionDto {
    pub id: String,
    pub course_id: String,
    pub course_name: Option<String>,
    pub date: NaiveDateTime,
    pub topic: Option<String>,
    pub present_count: Option<i64>,
    pub total_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MarkAttendanceRequest {
    pub course_id: String,
    pub date: String,
    pub topic: Option<String>,
    pub records: Vec<AttendanceRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceRecord {
    pub student_id: String,
    pub status: String, // PRESENT | ABSENT | LATE | EXCUSED
    pub remarks: Option<String>,
}

pub async fn mark_attendance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<MarkAttendanceRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["FACULTY", "ADMIN"]).await?;

    // Get faculty id
    let faculty_id = sqlx::query_scalar!(
        r#"SELECT id FROM "Faculty" WHERE user_id = $1"#,
        auth_user.id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Forbidden("Only faculty can mark attendance".to_string()))?;

    let date = body.date.parse::<NaiveDateTime>()
        .map_err(|_| AppError::Validation("Invalid date format. Use ISO 8601".to_string()))?;

    // Create session
    let session_id = sqlx::query_scalar!(
        r#"
        INSERT INTO "AttendanceSession" (id, course_id, faculty_id, date, topic, created_at)
        VALUES (gen_random_uuid()::text, $1, $2, $3, $4, NOW())
        RETURNING id
        "#,
        body.course_id,
        faculty_id,
        date,
        body.topic
    )
    .fetch_one(&state.db)
    .await?;

    // Insert attendance records
    for record in &body.records {
        sqlx::query!(
            r#"
            INSERT INTO "Attendance" (id, session_id, student_id, status, remarks, marked_at)
            VALUES (gen_random_uuid()::text, $1, $2, $3::\"AttendanceStatus\", $4, NOW())
            ON CONFLICT (session_id, student_id) DO UPDATE
            SET status = EXCLUDED.status, remarks = EXCLUDED.remarks
            "#,
            session_id,
            record.student_id,
            record.status,
            record.remarks
        )
        .execute(&state.db)
        .await?;
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({"session_id": session_id}))))
}

pub async fn get_student_attendance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(student_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    // Students can only see their own attendance
    if auth_user.role == "STUDENT" {
        let is_own = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM "Student" s JOIN "User" u ON u.id = s.user_id WHERE s.id = $1 AND u.id = $2)"#,
            student_id,
            auth_user.id
        )
        .fetch_one(&state.db)
        .await?
        .unwrap_or(false);

        if !is_own {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            sess.id AS session_id,
            sess.date,
            sess.topic,
            c.name AS course_name,
            c.code AS course_code,
            a.status::text AS status
        FROM "Attendance" a
        JOIN "AttendanceSession" sess ON sess.id = a.session_id
        JOIN "Course" c ON c.id = sess.course_id
        WHERE a.student_id = $1
        ORDER BY sess.date DESC
        "#,
        student_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows.into_iter().map(|r| serde_json::json!({
        "session_id": r.session_id,
        "date": r.date,
        "topic": r.topic,
        "course_name": r.course_name,
        "course_code": r.course_code,
        "status": r.status
    })).collect::<Vec<_>>()))
}
