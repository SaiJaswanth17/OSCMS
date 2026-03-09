use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    middleware::auth_guard::{AuthUser, require_roles},
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct ExamResultDto {
    pub exam_id: String,
    pub exam_title: String,
    pub course_name: Option<String>,
    pub total_marks: f64,
    pub marks_obtained: f64,
    pub grade: Option<String>,
    pub percentage: f64,
    pub is_published: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UploadResultRequest {
    pub exam_id: String,
    pub results: Vec<ResultEntry>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResultEntry {
    pub student_id: String,
    #[validate(range(min = 0.0))]
    pub marks_obtained: f64,
    pub remarks: Option<String>,
}

pub async fn upload_results(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<UploadResultRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["FACULTY", "ADMIN"]).await?;

    for entry in &body.results {
        // Compute grade
        let exam = sqlx::query!(
            r#"SELECT total_marks, passing_marks FROM "Exam" WHERE id = $1"#,
            body.exam_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Exam not found".to_string()))?;

        let pct = (entry.marks_obtained / exam.total_marks) * 100.0;
        let grade = compute_grade(pct);

        sqlx::query!(
            r#"
            INSERT INTO "ExamResult" (id, exam_id, student_id, marks_obtained, grade, remarks, is_published, created_at, updated_at)
            VALUES (gen_random_uuid()::text, $1, $2, $3, $4, $5, false, NOW(), NOW())
            ON CONFLICT (exam_id, student_id) DO UPDATE
            SET marks_obtained = EXCLUDED.marks_obtained,
                grade = EXCLUDED.grade,
                remarks = EXCLUDED.remarks,
                updated_at = NOW()
            "#,
            body.exam_id,
            entry.student_id,
            entry.marks_obtained,
            grade,
            entry.remarks
        )
        .execute(&state.db)
        .await?;
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Results uploaded successfully"}))))
}

pub async fn get_student_results(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(student_id): Path<String>,
) -> AppResult<impl IntoResponse> {
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
        SELECT er.exam_id, e.title AS exam_title, c.name AS course_name,
               e.total_marks, er.marks_obtained, er.grade, er.is_published
        FROM "ExamResult" er
        JOIN "Exam" e ON e.id = er.exam_id
        JOIN "Course" c ON c.id = e.course_id
        WHERE er.student_id = $1 AND er.is_published = true
        ORDER BY e.scheduled_at DESC
        "#,
        student_id
    )
    .fetch_all(&state.db)
    .await?;

    let results: Vec<ExamResultDto> = rows.into_iter().map(|r| {
        let pct = (r.marks_obtained / r.total_marks) * 100.0;
        ExamResultDto {
            exam_id: r.exam_id,
            exam_title: r.exam_title,
            course_name: r.course_name,
            total_marks: r.total_marks,
            marks_obtained: r.marks_obtained,
            grade: r.grade,
            percentage: (pct * 100.0).round() / 100.0,
            is_published: r.is_published,
        }
    }).collect();

    Ok(Json(results))
}

pub async fn publish_results(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(exam_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN", "FACULTY"]).await?;

    sqlx::query!(
        r#"UPDATE "ExamResult" SET is_published = true, published_at = NOW() WHERE exam_id = $1"#,
        exam_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"message": "Results published"})))
}

fn compute_grade(percentage: f64) -> String {
    match percentage as u32 {
        90..=100 => "A+".to_string(),
        80..=89  => "A".to_string(),
        70..=79  => "B+".to_string(),
        60..=69  => "B".to_string(),
        50..=59  => "C".to_string(),
        40..=49  => "D".to_string(),
        _        => "F".to_string(),
    }
}
