use axum::{
    extract::{Extension, Json, Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    middleware::auth_guard::AuthUser,
    utils::{errors::{AppError, AppResult}, pagination::{PaginatedResponse, PaginationMeta}},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct FacultyDto {
    pub id: String,
    pub faculty_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department_id: Option<String>,
    pub department_name: Option<String>,
    pub designation: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct FacultyFilter {
    pub department_id: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_faculty(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<FacultyFilter>,
) -> AppResult<impl IntoResponse> {
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;

    let rows = sqlx::query(
        r#"SELECT f.id, f.faculty_id, u.first_name, u.last_name, u.email,
                  f.department_id::text, d.name AS department_name,
                  f.designation, f.is_active,
                  COUNT(*) OVER () AS total_count
           FROM "Faculty" f
           JOIN "User" u ON u.id = f.user_id
           LEFT JOIN "Department" d ON d.id = f.department_id
           WHERE u.institution_id = $1
             AND ($2::uuid IS NULL OR f.department_id = $2::uuid)
           ORDER BY u.first_name
           LIMIT $3 OFFSET $4"#,
    )
    .bind(institution_id)
    .bind(filter.department_id.as_deref().and_then(|s| Uuid::parse_str(s).ok()))
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = rows.first()
        .and_then(|r| r.try_get::<Option<i64>, _>("total_count").ok().flatten())
        .unwrap_or(0) as u64;

    let faculty: Vec<FacultyDto> = rows.into_iter().map(|r| FacultyDto {
        id: r.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        faculty_id: r.try_get("faculty_id").unwrap_or_default(),
        first_name: r.try_get("first_name").unwrap_or_default(),
        last_name: r.try_get("last_name").unwrap_or_default(),
        email: r.try_get("email").unwrap_or_default(),
        department_id: r.try_get::<Option<String>, _>("department_id").unwrap_or_default(),
        department_name: r.try_get("department_name").unwrap_or_default(),
        designation: r.try_get("designation").unwrap_or_default(),
        is_active: r.try_get("is_active").unwrap_or(true),
    }).collect();

    Ok(axum::Json(PaginatedResponse {
        data: faculty,
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

pub async fn get_faculty(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let faculty_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Faculty not found".into()))?;
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;

    let row = sqlx::query(
        r#"SELECT f.id, f.faculty_id, u.first_name, u.last_name, u.email,
                  f.department_id::text, d.name AS department_name,
                  f.designation, f.is_active
           FROM "Faculty" f
           JOIN "User" u ON u.id = f.user_id
           LEFT JOIN "Department" d ON d.id = f.department_id
           WHERE f.id = $1 AND u.institution_id = $2"#,
    )
    .bind(faculty_uuid)
    .bind(institution_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Faculty not found".into()))?;

    Ok(axum::Json(FacultyDto {
        id: row.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        faculty_id: row.try_get("faculty_id").unwrap_or_default(),
        first_name: row.try_get("first_name").unwrap_or_default(),
        last_name: row.try_get("last_name").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        department_id: row.try_get::<Option<String>, _>("department_id").unwrap_or_default(),
        department_name: row.try_get("department_name").unwrap_or_default(),
        designation: row.try_get("designation").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
    }))
}
