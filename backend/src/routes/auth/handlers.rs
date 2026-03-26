use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::state::AppState;
use crate::dbmodels::user::{User, LoginRequest, RegisterRequest};

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.keycloak_config.login_user(&payload.username, &payload.password).await {
        Ok(token_response) => {
            let kc_id = "extracted-from-jwt".to_string();
            let user = User::new(kc_id, payload.username.clone(), None);

            let res = sqlx::query(
                "INSERT INTO users (id, username, join_date, servers) VALUES ($1, $2, $3, $4)
     ON CONFLICT (id) DO UPDATE SET username = EXCLUDED.username"
            )
                .bind(user.id)
                .bind(user.username)
                .bind(user.join_date)
                .bind(&user.servers[..])
                .execute(&state.pool) // Use your actual database pool variable name here
                .await;

            match res {
                Ok(_) => (StatusCode::OK, Json(token_response)).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB sync failed").into_response(),
            }
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state.keycloak_config.register_user(&payload.username, &payload.email, &payload.password).await {
        Ok(_) => (StatusCode::CREATED, "User registered").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}