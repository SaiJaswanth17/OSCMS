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
pub struct ResultEntry {
    pub student_id: String,
    pub marks: f64,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadResultsRequest {
    pub exam_id: String,
    pub results: Vec<ResultEntry>,
}

#[derive(Debug, Serialize)]
pub struct ExamResultDto {
    pub exam_id: String,
    pub exam_title: String,
    pub course_name: String,
    pub total_marks: f64,
    pub marks: f64,
    pub grade: Option<String>,
    pub is_published: bool,
}

fn calc_grade(marks: f64, total: f64) -> &'static str {
    let pct = (marks / total) * 100.0;
    match pct as u32 {
        90..=100 => "A+", 80..=89 => "A", 70..=79 => "B+",
        60..=69 => "B", 50..=59 => "C", 40..=49 => "D", _ => "F",
    }
}

pub async fn upload_results(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<UploadResultsRequest>,
) -> AppResult<impl IntoResponse> {
    let exam_uuid = Uuid::parse_str(&body.exam_id)
        .map_err(|_| AppError::NotFound("Exam not found".into()))?;

    let exam_row = sqlx::query(
        r#"SELECT total_marks, passing_marks FROM "Exam" WHERE id = $1"#,
    )
    .bind(exam_uuid)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Exam not found".into()))?;

    let total_marks: f64 = exam_row.try_get::<f64, _>("total_marks")
        .unwrap_or(100.0);

    for entry in &body.results {
        if let Ok(student_uuid) = Uuid::parse_str(&entry.student_id) {
            let grade = calc_grade(entry.marks, total_marks);
            let _ = sqlx::query(
                r#"INSERT INTO "ExamResult" (exam_id, student_id, marks, grade, remarks)
                   VALUES ($1, $2, $3, $4, $5)
                   ON CONFLICT (exam_id, student_id) DO UPDATE
                   SET marks = $3, grade = $4, remarks = $5"#,
            )
            .bind(exam_uuid)
            .bind(student_uuid)
            .bind(entry.marks)
            .bind(grade)
            .bind(&entry.remarks)
            .execute(&state.db)
            .await;
        }
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({"message": "Results uploaded"}))))
}

pub async fn get_student_results(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(student_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let student_uuid = Uuid::parse_str(&student_id)
        .map_err(|_| AppError::NotFound("Student not found".into()))?;

    let rows = sqlx::query(
        r#"SELECT er.exam_id::text, e.title AS exam_title, c.name AS course_name,
                  e.total_marks::float8, er.marks::float8, er.grade, er.is_published
           FROM "ExamResult" er
           JOIN "Exam" e ON e.id = er.exam_id
           JOIN "Course" c ON c.id = e.course_id
           WHERE er.student_id = $1
           ORDER BY e.exam_date DESC"#,
    )
    .bind(student_uuid)
    .fetch_all(&state.db)
    .await?;

    let data: Vec<_> = rows.into_iter().map(|r| serde_json::json!({
        "exam_id": r.try_get::<String, _>("exam_id").unwrap_or_default(),
        "exam_title": r.try_get::<String, _>("exam_title").unwrap_or_default(),
        "course_name": r.try_get::<String, _>("course_name").unwrap_or_default(),
        "total_marks": r.try_get::<f64, _>("total_marks").unwrap_or(100.0),
        "marks": r.try_get::<f64, _>("marks").unwrap_or(0.0),
        "grade": r.try_get::<Option<String>, _>("grade").unwrap_or_default(),
        "is_published": r.try_get::<bool, _>("is_published").unwrap_or(false),
    })).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

pub async fn publish_results(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(exam_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let exam_uuid = Uuid::parse_str(&exam_id)
        .map_err(|_| AppError::NotFound("Exam not found".into()))?;

    sqlx::query(
        r#"UPDATE "ExamResult" SET is_published = true WHERE exam_id = $1"#,
    )
    .bind(exam_uuid)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"message": "Results published"})))
}
