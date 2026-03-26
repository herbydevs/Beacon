use axum::Router;

pub mod auth;

pub fn api_router() -> Router {
    Router::new()
}