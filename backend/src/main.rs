use axum::{Router};
use bollard::Docker;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use keycloak::Keycloak::Config;

mod routes;
mod keycloak;

// AppState is shared across all routes via Arc
// This allows handlers to access the DB and Docker client
pub struct AppState {
    pub db: sqlx::PgPool,
    pub docker: Docker,
    pub keycloak_config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging & Environment Variables
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "beacon_backend=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Database Connection Pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ Connected to PostgreSQL");

    // 3. Docker Engine API Client (Unix Socket)
    let docker = Docker::connect_with_local_defaults()?;
    tracing::info!("✅ Docker client connected to /var/run/docker.sock");

    // 4. Initialize Keycloak Config (Placeholder for your keycloak module)
    let keycloak_config = Config::load_from_env();

    // 5. Wrap everything in an Arc for thread-safe sharing
    let state = Arc::new(AppState {
        db: pool,
        docker,
        keycloak_config,
    });

    // 6. Security: CORS Policy
    // In production, you would restrict 'Any' to your frontend domain
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 7. Route Aggregation
    // We nest our modular routers under specific prefixes
    let app = Router::new()
        .nest("/api/v1", routes::api_router())
        .layer(cors)
        .with_state(state);

    // 8. Start the Axum Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("🗼 Beacon Control Plane active on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}