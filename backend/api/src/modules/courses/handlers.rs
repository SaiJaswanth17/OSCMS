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
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct CourseDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub credits: i32,
    pub semester: i32,
    pub capacity: i32,
    pub enrolled_count: i64,
    pub department_name: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CourseFilter {
    pub department_id: Option<String>,
    pub semester: Option<i32>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub credits: Option<i32>,
    pub semester: i32,
    pub capacity: Option<i32>,
    pub department_id: Option<String>,
}

pub async fn list_courses(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<CourseFilter>,
) -> AppResult<impl IntoResponse> {
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        r#"SELECT c.id, c.code, c.name, c.description, c.credits, c.semester, c.capacity,
                  d.name AS department_name, c.is_active,
                  (SELECT COUNT(*) FROM "Enrollment" e WHERE e.course_id = c.id AND e.status = 'ACTIVE') AS enrolled_count,
                  COUNT(*) OVER () AS total_count
           FROM "Course" c
           LEFT JOIN "Department" d ON d.id = c.department_id
           WHERE c.institution_id = $1
             AND ($2::uuid IS NULL OR c.department_id = $2::uuid)
             AND ($3::int IS NULL OR c.semester = $3)
           ORDER BY c.name
           LIMIT $4 OFFSET $5"#,
    )
    .bind(institution_id)
    .bind(filter.department_id.as_deref().and_then(|s| Uuid::parse_str(s).ok()))
    .bind(filter.semester)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let courses: Vec<CourseDto> = rows.into_iter().map(|r| CourseDto {
        id: r.try_get::<Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
        code: r.try_get("code").unwrap_or_default(),
        name: r.try_get("name").unwrap_or_default(),
        description: r.try_get("description").unwrap_or_default(),
        credits: r.try_get("credits").unwrap_or(3),
        semester: r.try_get("semester").unwrap_or(1),
        capacity: r.try_get("capacity").unwrap_or(60),
        enrolled_count: r.try_get("enrolled_count").unwrap_or(0),
        department_name: r.try_get("department_name").unwrap_or_default(),
        is_active: r.try_get("is_active").unwrap_or(true),
    }).collect();

    Ok(Json(serde_json::json!({ "data": courses })))
}

pub async fn create_course(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateCourseRequest>,
) -> AppResult<impl IntoResponse> {
    let institution_id = Uuid::parse_str(&auth_user.institution_id)
        .map_err(|_| AppError::Unauthorized("Bad institution id".into()))?;
    let dept_id = body.department_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let row = sqlx::query(
        r#"INSERT INTO "Course" (institution_id, department_id, code, name, description, credits, semester, capacity)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id::text"#,
    )
    .bind(institution_id)
    .bind(dept_id)
    .bind(&body.code)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.credits.unwrap_or(3))
    .bind(body.semester)
    .bind(body.capacity.unwrap_or(60))
    .fetch_one(&state.db)
    .await?;

    let id: String = row.try_get("id").unwrap_or_default();
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn enroll_student(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(course_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let uid = Uuid::parse_str(&auth_user.id)
        .map_err(|_| AppError::Unauthorized("Bad user id".into()))?;
    let course_uuid = Uuid::parse_str(&course_id)
        .map_err(|_| AppError::NotFound("Course not found".into()))?;

    // Get student id for this user
    let row = sqlx::query(r#"SELECT id FROM "Student" WHERE user_id = $1"#)
        .bind(uid)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Student profile not found".into()))?;

    let student_id: Uuid = row.try_get("id")?;

    // Check capacity
    let cap_row = sqlx::query(
        r#"SELECT capacity,
                  (SELECT COUNT(*) FROM "Enrollment" WHERE course_id = $1 AND status = 'ACTIVE') AS enrolled
           FROM "Course" WHERE id = $1"#,
    )
    .bind(course_uuid)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Course not found".into()))?;

    let capacity: i32 = cap_row.try_get("capacity").unwrap_or(60);
    let enrolled: i64 = cap_row.try_get("enrolled").unwrap_or(0);
    if enrolled >= capacity as i64 {
        return Err(AppError::BadRequest("Course is at full capacity".into()));
    }

    sqlx::query(
        r#"INSERT INTO "Enrollment" (student_id, course_id, status)
           VALUES ($1, $2, 'ACTIVE')
           ON CONFLICT (student_id, course_id) DO UPDATE SET status = 'ACTIVE'"#,
    )
    .bind(student_id)
    .bind(course_uuid)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"message": "Enrolled successfully"})))
}
