pub mod handlers;
use std::sync::Arc;
//use axum::Router;
use crate::state::AppState;
use axum::{routing::post, routing::get,Router};

pub fn create_router() -> Router<Arc<AppState >>{
    Router::new()
            .route("/create", post(handlers::handle_create_server))
            .route("/start", post(handlers::handle_start_server))
            .route("/stop", post(handlers::handle_stop_server))
            .route("/get", get(handlers::handle_get_servers))
            .route("/:name/logs", get(handlers::handle_server_logs))
}