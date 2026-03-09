use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    trace::TraceLayer,
};
use tracing::info;

mod auth;
mod middleware as mw;
mod modules;
mod utils;

use auth::jwt::JwtConfig;
use mw::auth_guard::auth_middleware;

// ─────────────────────────────────────────────
// App State
// ─────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_config: Arc<JwtConfig>,
}

// ─────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("LOG_LEVEL")
                .add_directive("api=debug".parse()?)
                .add_directive("tower_http=debug".parse()?),
        )
        .json()
        .init();

    info!("Starting OCMS API server...");

    // Database pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = sqlx::PgPool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("../../prisma/migrations").run(&db).await?;

    let state = AppState {
        db,
        jwt_config: Arc::new(JwtConfig::from_env()),
    };

    // Router
    let app = build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("OCMS API listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::handlers::login))
        .route("/api/health", get(health_check));

    // Protected routes
    let protected_routes = Router::new()
        // Auth
        .route("/api/auth/me", get(auth::handlers::me))
        .route("/api/auth/logout", post(auth::handlers::logout))
        // Students
        .route("/api/students", get(modules::students::handlers::list_students))
        .route("/api/students/:id", get(modules::students::handlers::get_student))
        .route("/api/students/:id", delete(modules::students::handlers::delete_student))
        // Faculty
        .route("/api/faculty", get(modules::faculty::handlers::list_faculty))
        .route("/api/faculty/:id", get(modules::faculty::handlers::get_faculty))
        // Courses
        .route("/api/courses", get(modules::courses::handlers::list_courses))
        .route("/api/courses", post(modules::courses::handlers::create_course))
        .route("/api/courses/:id/enroll", post(modules::courses::handlers::enroll_student))
        // Attendance
        .route("/api/attendance/mark", post(modules::attendance::handlers::mark_attendance))
        .route("/api/attendance/student/:id", get(modules::attendance::handlers::get_student_attendance))
        // Exams
        .route("/api/results/upload", post(modules::exams::handlers::upload_results))
        .route("/api/results/student/:id", get(modules::exams::handlers::get_student_results))
        .route("/api/results/exam/:id/publish", post(modules::exams::handlers::publish_results))
        // Notifications
        .route("/api/notifications", get(modules::notifications::handlers::get_notifications))
        .route("/api/notifications/read-all", post(modules::notifications::handlers::mark_notifications_read))
        .route("/api/notifications/unread-count", get(modules::notifications::handlers::get_unread_count))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "ocms-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
