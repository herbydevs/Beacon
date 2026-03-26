use axum::{routing::get, Router};
use bollard::Docker;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use http::{header, Method};

pub mod routes;
pub mod keycloak;
pub mod dbmodels;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cors = CorsLayer::new()
        // Explicitly allow your local dev domains
        .allow_origin([
            "http://beacon.local".parse()?,
            "http://api.beacon.local".parse()?,
            "http://localhost:5173".parse()?, // Common for Vue/Vite dev
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE
        ]);

    // 1. Initialize Logging & Environment Variables
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "beacon_backend=debug,axum=info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load Infrastructure Configurations
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let keycloak_realm = env::var("KEYCLOAK_REALM")
        .unwrap_or_else(|_| "beacon".to_string());

    // 3. Initialize Database with Retry Logic
    // This prevents the "PoolTimedOut" panic during Docker startup
    let mut pool = None;
    let max_retries = 10;

    for i in 1..=max_retries {
        tracing::info!("🔄 Connecting to DB (Attempt {}/{})...", i, max_retries);
        match PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                pool = Some(p);
                break;
            }
            Err(e) => {
                if i == max_retries {
                    tracing::error!("❌ Database connection failed permanently: {}", e);
                    anyhow::bail!("Could not connect to database");
                }
                tracing::warn!("⚠️ Database not ready, retrying in 2s...");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    let pool = pool.unwrap();

    // 4. Initialize Docker Engine Client
    // Ensure /var/run/docker.sock is mounted in docker-compose.yml
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            tracing::error!("❌ Docker socket unreachable. Check volume mounts.");
            e
        })?;

    tracing::info!("✅ Infrastructure (DB & Docker) Ready");

    // 5. Shared Application State
    let app_state = Arc::new(state::AppState {
        pool,
        docker,
        keycloak_realm,
        keycloak_config: keycloak::Keycloak::Config::load_from_env(),
    });

    // 6. Security & Routing
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Beacon Control Plane API v1.0" }))
        .route("/health", get(|| async { "OK" }))
        .nest("/api/v1", routes::api_router())
        .layer(cors)
        .with_state(app_state);

    // 7. Start the Axum Server
    let port = env::var("BACKEND_PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    tracing::info!("🚀 Beacon Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}