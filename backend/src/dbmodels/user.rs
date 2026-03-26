use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,        // Keycloak UUID
    pub username: String,
    pub join_date: i64,
    pub servers: Vec<String>,
}

impl User {
    pub fn new(id: String, username: String, servers: Option<Vec<String>>) -> Self {
        Self {
            id,
            username,
            join_date: Utc::now().timestamp(),
            servers: servers.unwrap_or_default(),
        }
    }
}

// Structs for API interaction
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}