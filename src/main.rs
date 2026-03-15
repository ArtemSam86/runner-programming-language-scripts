mod app_state;
mod error;
mod models;
mod db;
mod handlers;
mod script_runner;
mod utils;
pub mod migrations;
pub mod jwt;
pub mod auth_middleware;

use utoipa::{OpenApi, Modify};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use axum::{Router, routing::{get, post}, middleware};
use tower_http::cors::{CorsLayer, AllowOrigin};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::db::ensure_superadmin;
use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::register,
        handlers::login,
        handlers::list_scripts,
        handlers::create_script,
        handlers::get_script,
        handlers::update_script,
        handlers::delete_script,
        handlers::run_scripts,
        handlers::run_single_script,
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            LoginResponse,
            ScriptMetadata,
            CreateScriptRequest,
            UpdateScriptRequest,
            RunRequest,
            RunQuery,
            ScriptResult,
            RunResponse,
            SearchQuery,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "scripts", description = "Script management"),
        (name = "execution", description = "Script execution"),
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // MongoDB
    let mongo_uri = std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "script_manager".into());
    let db = db::init_db(&mongo_uri, &db_name).await.expect("Failed to connect to MongoDB");

    // Запуск миграций
    info!("Running database migrations...");
    if let Err(e) = migrations::run_migrations(&db).await {
        error!("Migration failed: {}", e);
        std::process::exit(1);
    }
    info!("Migrations completed successfully");

    // Проверка и создание суперадмина, если нет пользователей
    if let Err(e) = ensure_superadmin(&db).await {
        error!("Failed to ensure superadmin: {}", e);
        std::process::exit(1);
    }

    // Директория скриптов
    let scripts_dir = PathBuf::from("./scripts");
    if !scripts_dir.exists() {
        tokio::fs::create_dir_all(&scripts_dir).await.expect("Failed to create scripts directory");
    }

    let state = Arc::new(app_state::AppState::new(
        scripts_dir,
        db,
        4,
        Duration::from_secs(30),
    ));

    // Первичная синхронизация
    script_runner::scan_scripts(state.clone()).await;

    // Фоновое сканирование
    let scanner_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            script_runner::scan_scripts(scanner_state.clone()).await;
        }
    });

    // CORS
    let origins = std::env::var("ALLOWED_ORIGINS").ok();
    let (allow_origin, is_any) = if let Some(origins_str) = origins {
        if origins_str == "*" {
            (AllowOrigin::any(), true)
        } else {
            let origins: Vec<_> = origins_str
                .split(',')
                .map(|s| s.trim().parse().expect("Invalid origin format"))
                .collect();
            (AllowOrigin::list(origins), false)
        }
    } else {
        (AllowOrigin::any(), true)
    };

    let mut cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            axum::http::header::ACCEPT,
            axum::http::header::AUTHORIZATION,
        ]);

    if !is_any && std::env::var("CORS_ALLOW_CREDENTIALS").as_deref() == Ok("true") {
        cors = cors.allow_credentials(true);
    }

    let protected_routes = Router::new()
        .route("/scripts", get(handlers::list_scripts).post(handlers::create_script))
        .route("/scripts/{name}", get(handlers::get_script).put(handlers::update_script).delete(handlers::delete_script))
        .route("/run", post(handlers::run_scripts))
        .route("/run/{name}", post(handlers::run_single_script))
        .layer(middleware::from_fn(auth_middleware::auth_middleware));

    let public_routes = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login));

    // Создаём OpenApiRouter из обычного роутера (через .into())
    let (openapi_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(public_routes.into())
        .merge(protected_routes.into())
        .split_for_parts();

    let app = openapi_router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}