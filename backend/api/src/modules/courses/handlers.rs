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

#[derive(Debug, Serialize)]
pub struct CourseDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub credits: i32,
    pub max_capacity: i32,
    pub enrolled_count: Option<i64>,
    pub department_id: String,
    pub department_name: Option<String>,
    pub faculty_id: Option<String>,
    pub faculty_name: Option<String>,
    pub semester_id: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCourseRequest {
    #[validate(length(min = 2, max = 20))]
    pub code: String,
    #[validate(length(min = 3, max = 200))]
    pub name: String,
    pub description: Option<String>,
    #[validate(range(min = 1, max = 12))]
    pub credits: i32,
    pub max_capacity: Option<i32>,
    pub department_id: String,
    pub faculty_id: Option<String>,
    pub semester_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CourseFilterParams {
    pub department_id: Option<String>,
    pub semester_id: Option<String>,
    pub faculty_id: Option<String>,
    pub is_active: Option<bool>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

pub async fn list_courses(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(params): Query<CourseFilterParams>,
) -> AppResult<impl IntoResponse> {
    let offset = params.pagination.offset() as i64;
    let limit = params.pagination.limit_clamped() as i64;

    let rows = sqlx::query!(
        r#"
        SELECT c.id, c.code, c.name, c.description, c.credits, c.max_capacity,
               c.department_id, d.name AS department_name,
               c.faculty_id, CONCAT(u.first_name, ' ', u.last_name) AS faculty_name,
               c.semester_id, c.is_active,
               COUNT(DISTINCT e.id) AS enrolled_count,
               COUNT(*) OVER () AS total_count
        FROM "Course" c
        JOIN "Department" d ON d.id = c.department_id
        LEFT JOIN "Faculty" f ON f.id = c.faculty_id
        LEFT JOIN "User" u ON u.id = f.user_id
        LEFT JOIN "Enrollment" e ON e.course_id = c.id AND e.status = 'ACTIVE'
        WHERE c.institution_id = $1
          AND ($2::text IS NULL OR c.department_id = $2)
          AND ($3::text IS NULL OR c.semester_id = $3)
          AND ($4::text IS NULL OR c.faculty_id = $4)
          AND ($5::boolean IS NULL OR c.is_active = $5)
        GROUP BY c.id, d.name, u.first_name, u.last_name
        ORDER BY c.code
        LIMIT $6 OFFSET $7
        "#,
        auth_user.institution_id,
        params.department_id,
        params.semester_id,
        params.faculty_id,
        params.is_active,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total = rows.first().and_then(|r| r.total_count).unwrap_or(0) as u64;
    let courses: Vec<CourseDto> = rows
        .into_iter()
        .map(|r| CourseDto {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
            credits: r.credits,
            max_capacity: r.max_capacity,
            enrolled_count: r.enrolled_count,
            department_id: r.department_id,
            department_name: r.department_name,
            faculty_id: r.faculty_id,
            faculty_name: r.faculty_name,
            semester_id: r.semester_id,
            is_active: r.is_active,
        })
        .collect();

    Ok(Json(PaginatedResponse::new(courses, total, &params.pagination)))
}

pub async fn create_course(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateCourseRequest>,
) -> AppResult<impl IntoResponse> {
    require_roles(&auth_user, &["ADMIN"]).await?;
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO "Course" (id, code, name, description, credits, max_capacity,
                              institution_id, department_id, faculty_id, semester_id,
                              is_active, created_at, updated_at)
        VALUES (gen_random_uuid()::text, $1, $2, $3, $4, $5, $6, $7, $8, $9, true, NOW(), NOW())
        RETURNING id
        "#,
        body.code,
        body.name,
        body.description,
        body.credits,
        body.max_capacity.unwrap_or(60),
        auth_user.institution_id,
        body.department_id,
        body.faculty_id,
        body.semester_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn enroll_student(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(course_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    // Get student record for the current user
    let student_id = sqlx::query_scalar!(
        r#"SELECT id FROM "Student" WHERE user_id = $1"#,
        auth_user.id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Student profile not found".to_string()))?;

    // Check capacity
    let course = sqlx::query!(
        r#"
        SELECT max_capacity,
               (SELECT COUNT(*) FROM "Enrollment" WHERE course_id = $1 AND status = 'ACTIVE') AS enrolled
        FROM "Course"
        WHERE id = $1 AND is_active = true
        "#,
        course_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    if course.enrolled.unwrap_or(0) >= course.max_capacity as i64 {
        return Err(AppError::Conflict("Course is at full capacity".to_string()));
    }

    sqlx::query!(
        r#"
        INSERT INTO "Enrollment" (id, student_id, course_id, status, enrolled_at)
        VALUES (gen_random_uuid()::text, $1, $2, 'ACTIVE', NOW())
        ON CONFLICT (student_id, course_id) DO NOTHING
        "#,
        student_id,
        course_id
    )
    .execute(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"message": "Enrolled successfully"}))))
}
