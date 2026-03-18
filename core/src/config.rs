use serde::Deserialize;

use crate::error::CoreError;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

fn default_port() -> u16 {
    4200
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_redirect_uri() -> String {
    "http://localhost:4200/auth/callback".to_string()
}

fn default_required_org() -> String {
    "bodymindarts".to_string()
}

fn default_required_team() -> String {
    "ops-team".to_string()
}

fn default_session_ttl_hours() -> u64 {
    24
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub github_client_id: String,
    #[serde(default = "default_redirect_uri")]
    pub github_redirect_uri: String,
    #[serde(default = "default_required_org")]
    pub required_org: String,
    #[serde(default = "default_required_team")]
    pub required_team: String,
    #[serde(default = "default_session_ttl_hours")]
    pub session_ttl_hours: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
}

impl Config {
    pub fn load() -> Result<Self, CoreError> {
        let port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok());

        let host = std::env::var("SERVER_HOST").ok();

        let github_client_id = std::env::var("GITHUB_CLIENT_ID").ok();

        let mut config = Config::default();

        if let Some(port) = port {
            config.server.port = port;
        }
        if let Some(host) = host {
            config.server.host = host;
        }

        if let Some(client_id) = github_client_id {
            let redirect_uri =
                std::env::var("GITHUB_REDIRECT_URI").unwrap_or_else(|_| default_redirect_uri());
            let required_org =
                std::env::var("GITHUB_REQUIRED_ORG").unwrap_or_else(|_| default_required_org());
            let required_team =
                std::env::var("GITHUB_REQUIRED_TEAM").unwrap_or_else(|_| default_required_team());
            let session_ttl_hours = std::env::var("SESSION_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_session_ttl_hours);

            config.auth = Some(AuthConfig {
                github_client_id: client_id,
                github_redirect_uri: redirect_uri,
                required_org,
                required_team,
                session_ttl_hours,
            });
        }

        Ok(config)
    }
}
