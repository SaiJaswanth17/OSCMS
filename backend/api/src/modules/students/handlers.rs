use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    middleware::auth_guard::AuthUser,
    utils::{errors::{AppError, AppResult}, pagination::{PaginatedResponse, PaginationMeta, PaginationParams}},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct StudentDto {
    pub id: String,
    pub student_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department_id: Option<String>,
    pub department_name: Option<String>,
    pub current_semester: i32,
    pub gpa: Option<f64>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct StudentFilter {
    pub department_id: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_students(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<StudentFilter>,
) -> AppResult<impl IntoResponse> {
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;

    let rows = sqlx::query(
        r#"SELECT s.id, s.student_id, u.first_name, u.last_name, u.email,
                  s.department_id::text, d.name AS department_name,
                  s.current_semester, s.gpa, s.is_active,
                  COUNT(*) OVER () AS total_count
           FROM "Student" s
           JOIN "User" u ON u.id = s.user_id
           LEFT JOIN "Department" d ON d.id = s.department_id
           WHERE u.institution_id = $1
             AND ($2::text IS NULL OR u.first_name ILIKE $2 OR u.last_name ILIKE $2 OR u.email ILIKE $2 OR s.student_id ILIKE $2)
             AND ($3::bool IS NULL OR s.is_active = $3)
           ORDER BY u.first_name
           LIMIT $4 OFFSET $5"#,
    )
    .bind(institution_id)
    .bind(filter.search.as_deref().map(|s| format!("%{s}%")))
    .bind(filter.is_active)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = rows.first()
        .and_then(|r| r.try_get::<Option<i64>, _>("total_count").ok().flatten())
        .unwrap_or(0) as u64;

    let students: Vec<StudentDto> = rows.into_iter().map(|r| StudentDto {
        id: r.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        student_id: r.try_get("student_id").unwrap_or_default(),
        first_name: r.try_get("first_name").unwrap_or_default(),
        last_name: r.try_get("last_name").unwrap_or_default(),
        email: r.try_get("email").unwrap_or_default(),
        department_id: r.try_get::<Option<String>, _>("department_id").unwrap_or_default(),
        department_name: r.try_get("department_name").unwrap_or_default(),
        current_semester: r.try_get("current_semester").unwrap_or(1),
        gpa: r.try_get::<Option<f64>, _>("gpa").unwrap_or_default(),
        is_active: r.try_get("is_active").unwrap_or(true),
    }).collect();

    Ok(Json(PaginatedResponse {
        data: students,
        meta: PaginationMeta {
            total,
            page: page as u64,
            limit: per_page as u64,
            total_pages: total.div_ceil(per_page as u64),
            has_next: (page as u64) < total.div_ceil(per_page as u64),
            has_prev: page > 1,
        },
    }))
}

pub async fn get_student(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let student_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Student not found".into()))?;
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;

    let row = sqlx::query(
        r#"SELECT s.id, s.student_id, u.first_name, u.last_name, u.email,
                  s.department_id::text, d.name AS department_name,
                  s.current_semester, s.gpa, s.is_active
           FROM "Student" s
           JOIN "User" u ON u.id = s.user_id
           LEFT JOIN "Department" d ON d.id = s.department_id
           WHERE s.id = $1 AND u.institution_id = $2"#,
    )
    .bind(student_uuid)
    .bind(institution_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Student not found".into()))?;

    Ok(Json(StudentDto {
        id: row.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        student_id: row.try_get("student_id").unwrap_or_default(),
        first_name: row.try_get("first_name").unwrap_or_default(),
        last_name: row.try_get("last_name").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        department_id: row.try_get::<Option<String>, _>("department_id").unwrap_or_default(),
        department_name: row.try_get("department_name").unwrap_or_default(),
        current_semester: row.try_get("current_semester").unwrap_or(1),
        gpa: row.try_get::<Option<f64>, _>("gpa").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
    }))
}

pub async fn delete_student(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let student_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Student not found".into()))?;

    sqlx::query(r#"UPDATE "Student" SET is_active = false WHERE id = $1"#)
        .bind(student_uuid)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
