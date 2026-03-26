use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Config {
    /// Loads Keycloak configuration from environment variables.
    /// Expects: KEYCLOAK_SERVER_URL, KEYCLOAK_REALM, KEYCLOAK_CLIENT_ID, 
    /// KEYCLOAK_CLIENT_SECRET, KEYCLOAK_REDIRECT_URI
    pub fn load_from_env() -> Self {
        Self {
            server_url: env::var("KEYCLOAK_SERVER_URL")
                .unwrap_or_else(|_| "http://sso.beacon.local:8080".to_string()),
            realm: env::var("KEYCLOAK_REALM")
                .unwrap_or_else(|_| "beacon".to_string()),
            client_id: env::var("KEYCLOAK_CLIENT_ID")
                .unwrap_or_else(|_| "beacon-backend".to_string()),
            client_secret: env::var("KEYCLOAK_CLIENT_SECRET")
                .expect("KEYCLOAK_CLIENT_SECRET must be set"),
            redirect_uri: env::var("KEYCLOAK_REDIRECT_URI")
                .unwrap_or_else(|_| "http://api.beacon.local/api/v1/auth/callback".to_string()),
        }
    }

    /// Generates the full discovery URL for the OIDC well-known endpoint
    pub fn discovery_url(&self) -> String {
        format!(
            "{}/realms/{}/.well-known/openid-configuration",
            self.server_url, self.realm
        )
    }

    /// Helper to get the token endpoint
    pub fn token_endpoint(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/token", self.server_url, self.realm)
    }

    /// Generates the URL to redirect users to for logging in
    pub fn login_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid",
            self.server_url, self.realm, self.client_id, self.redirect_uri
        )
    }

    /// Generates the URL to redirect users to for registration
    pub fn register_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/registrations?client_id={}&redirect_uri={}&response_type=code&scope=openid",
            self.server_url, self.realm, self.client_id, self.redirect_uri
        )
    }
}

/// Represents the claims we expect from a Keycloak JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakClaims {
    pub sub: String,          // User ID
    pub preferred_username: String,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}