use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    middleware::auth_guard::{AuthUser, require_roles},
    utils::{
        errors::{AppError, AppResult},
        pagination::{PaginatedResponse, PaginationParams},
    },
    AppState,
};

#[derive(Debug, Serialize)]
pub struct FacultyDto {
    pub id: String,
    pub faculty_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department_id: String,
    pub department_name: Option<String>,
    pub designation: String,
    pub specialization: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct FacultyFilterParams {
    pub department_id: Option<String>,
    pub is_active: Option<bool>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

pub async fn list_faculty(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(params): Query<FacultyFilterParams>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN", "DEPT_HEAD"]).await?;

    let offset = params.pagination.offset() as i64;
    let limit = params.pagination.limit_clamped() as i64;

    let rows = sqlx::query!(
        r#"
        SELECT f.id, f.faculty_id, u.first_name, u.last_name, u.email,
               f.department_id, d.name AS department_name,
               f.designation, f.specialization, f.is_active,
               COUNT(*) OVER () AS total_count
        FROM "Faculty" f
        JOIN "User" u ON u.id = f.user_id
        JOIN "Department" d ON d.id = f.department_id
        WHERE u.institution_id = $1
          AND ($2::text IS NULL OR f.department_id = $2)
          AND ($3::boolean IS NULL OR f.is_active = $3)
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
    let faculty: Vec<FacultyDto> = rows
        .into_iter()
        .map(|r| FacultyDto {
            id: r.id,
            faculty_id: r.faculty_id,
            first_name: r.first_name,
            last_name: r.last_name,
            email: r.email,
            department_id: r.department_id,
            department_name: r.department_name,
            designation: r.designation,
            specialization: r.specialization,
            is_active: r.is_active,
        })
        .collect();

    Ok(Json(PaginatedResponse::new(faculty, total, &params.pagination)))
}

pub async fn get_faculty(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN", "DEPT_HEAD", "FACULTY"]).await?;

    let row = sqlx::query!(
        r#"
        SELECT f.id, f.faculty_id, u.first_name, u.last_name, u.email,
               f.department_id, d.name AS department_name,
               f.designation, f.specialization, f.is_active
        FROM "Faculty" f
        JOIN "User" u ON u.id = f.user_id
        JOIN "Department" d ON d.id = f.department_id
        WHERE f.id = $1 AND u.institution_id = $2
        "#,
        id,
        auth_user.institution_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Faculty member not found".to_string()))?;

    Ok(Json(FacultyDto {
        id: row.id,
        faculty_id: row.faculty_id,
        first_name: row.first_name,
        last_name: row.last_name,
        email: row.email,
        department_id: row.department_id,
        department_name: row.department_name,
        designation: row.designation,
        specialization: row.specialization,
        is_active: row.is_active,
    }))
}
