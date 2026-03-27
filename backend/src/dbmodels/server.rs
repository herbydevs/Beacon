use serde::{Deserialize,Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct ServerSettings {
    pub pvp: bool,
    pub max_players: i32,
    pub motd: String,
}

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub version: String,          // e.g., "1.20.1"
    pub server_type: String,      // e.g., "PAPER", "FORGE", "VANILLA"
    pub memory: Option<String>,   // e.g., "2G"
    pub online_mode: Option<bool>,
}


#[derive(serde::Deserialize)]
pub struct startServerRequest{
    pub name: String,
}


#[derive(serde::Deserialize)]
pub struct deleteServerRequest{
    pub id: Uuid,
}


#[derive(Debug, Serialize, FromRow)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub container_id: String,
    pub version: String,
    pub status: String,    // "running", "exited", etc.
    pub server_type: String, // e.g., "Spigot"
    #[sqlx(skip)]          // These aren't in Postgres, we'll add them live
    pub cpu_usage: Option<f64>,
    #[sqlx(skip)]
    pub address: Option<String>,
}
