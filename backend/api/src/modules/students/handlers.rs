use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    middleware::auth_guard::{AuthUser, require_roles},
    utils::{
        errors::{AppError, AppResult},
        pagination::{PaginatedResponse, PaginationParams},
    },
    AppState,
};

// ─── DTOs ────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StudentDto {
    pub id: String,
    pub student_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department_id: String,
    pub department_name: Option<String>,
    pub enrollment_year: i32,
    pub current_semester: i32,
    pub gpa: Option<f64>,
    pub credits_earned: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStudentRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub department_id: String,
    pub enrollment_year: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStudentRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub current_semester: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct StudentFilterParams {
    pub department_id: Option<String>,
    pub is_active: Option<bool>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

// ─── Handlers ────────────────────────────────

pub async fn list_students(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(params): Query<StudentFilterParams>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN", "DEPT_HEAD", "FACULTY"]).await?;

    let offset = params.pagination.offset() as i64;
    let limit = params.pagination.limit_clamped() as i64;

    let rows = sqlx::query!(
        r#"
        SELECT
            s.id, s.student_id, u.first_name, u.last_name, u.email,
            s.department_id, d.name AS department_name,
            s.enrollment_year, s.current_semester, s.gpa, s.credits_earned, s.is_active,
            COUNT(*) OVER () AS total_count
        FROM "Student" s
        JOIN "User" u ON u.id = s.user_id
        JOIN "Department" d ON d.id = s.department_id
        WHERE u.institution_id = $1
          AND ($2::text IS NULL OR s.department_id = $2)
          AND ($3::boolean IS NULL OR s.is_active = $3)
        ORDER BY u.last_name, u.first_name
        LIMIT $4 OFFSET $5
        "#,
        auth_user.institution_id,
        params.department_id,
        params.is_active,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total = rows.first().and_then(|r| r.total_count).unwrap_or(0) as u64;
    let students: Vec<StudentDto> = rows
        .into_iter()
        .map(|r| StudentDto {
            id: r.id,
            student_id: r.student_id,
            first_name: r.first_name,
            last_name: r.last_name,
            email: r.email,
            department_id: r.department_id,
            department_name: r.department_name,
            enrollment_year: r.enrollment_year,
            current_semester: r.current_semester,
            gpa: r.gpa,
            credits_earned: r.credits_earned,
            is_active: r.is_active,
        })
        .collect();

    Ok(Json(PaginatedResponse::new(students, total, &params.pagination)))
}

pub async fn get_student(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    // Students can only view their own profile
    if auth_user.role == "STUDENT" {
        let is_own = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM "Student" s JOIN "User" u ON u.id = s.user_id WHERE s.id = $1 AND u.id = $2)"#,
            id,
            auth_user.id
        )
        .fetch_one(&state.db)
        .await?
        .unwrap_or(false);

        if !is_own {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
    }

    let row = sqlx::query!(
        r#"
        SELECT s.id, s.student_id, u.first_name, u.last_name, u.email,
               s.department_id, d.name AS department_name,
               s.enrollment_year, s.current_semester, s.gpa, s.credits_earned, s.is_active
        FROM "Student" s
        JOIN "User" u ON u.id = s.user_id
        JOIN "Department" d ON d.id = s.department_id
        WHERE s.id = $1 AND u.institution_id = $2
        "#,
        id,
        auth_user.institution_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    Ok(Json(StudentDto {
        id: row.id,
        student_id: row.student_id,
        first_name: row.first_name,
        last_name: row.last_name,
        email: row.email,
        department_id: row.department_id,
        department_name: row.department_name,
        enrollment_year: row.enrollment_year,
        current_semester: row.current_semester,
        gpa: row.gpa,
        credits_earned: row.credits_earned,
        is_active: row.is_active,
    }))
}

pub async fn delete_student(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN"]).await?;

    sqlx::query!(
        r#"UPDATE "Student" SET is_active = false WHERE id = $1"#,
        id
    )
    .execute(&state.db)
    .await?;

    Ok((StatusCode::NO_CONTENT, ()))
}
