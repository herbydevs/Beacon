use axum::{routing::post, Router};
use std::sync::Arc;
use crate::state::AppState;

pub mod handlers;

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(handlers::login))
        .route("/register", post(handlers::register))
}