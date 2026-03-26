use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Config {
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

    /// Logs in a user DIRECTLY by exchanging credentials for a JWT token.
    /// This bypasses the browser redirect and is useful for mobile apps or CLI tools.
    pub async fn login_user(&self, username: &str, password: &str) -> Result<TokenResponse> {
        let client = Client::new();
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("grant_type", &"password".to_string()),
            ("username", &username.to_string()),
            ("password", &password.to_string()),
            ("scope", &"openid".to_string()),
        ];

        let response = client
            .post(self.token_endpoint())
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(response)
    }

    /// Registers a user DIRECTLY using the Keycloak Admin API.
    /// Note: This requires the 'beacon-backend' client to have 'manage-users' permissions
    /// in Keycloak Service Account Roles.
    pub async fn register_user(&self, username: &str, email: &str, password: &str) -> Result<()> {
        let client = Client::new();

        // 1. Get an Admin Access Token first (Client Credentials Flow)
        let admin_token = self.get_admin_token().await?;

        // 2. Create the user payload
        let user_payload = serde_json::json!({
            "username": username,
            "email": email,
            "enabled": true,
            "credentials": [{
                "type": "password",
                "value": password,
                "temporary": false
            }]
        });

        // 3. POST to Keycloak Admin API
        let admin_url = format!("{}/admin/realms/{}/users", self.server_url, self.realm);
        let res = client
            .post(admin_url)
            .bearer_auth(admin_token)
            .json(&user_payload)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err_text = res.text().await?;
            anyhow::bail!("Keycloak Admin API error: {}", err_text)
        }
    }

    /// Internal helper to get a client credentials token for administrative tasks
    async fn get_admin_token(&self) -> Result<String> {
        let client = Client::new();
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("grant_type", &"client_credentials".to_string()),
        ];

        let response = client
            .post(self.token_endpoint())
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(response.access_token)
    }

    pub fn token_endpoint(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/token", self.server_url, self.realm)
    }

    pub fn login_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid",
            self.server_url, self.realm, self.client_id, self.redirect_uri
        )
    }

    pub fn register_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/registrations?client_id={}&redirect_uri={}&response_type=code&scope=openid",
            self.server_url, self.realm, self.client_id, self.redirect_uri
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakClaims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
}