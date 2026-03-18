use std::path::Path;

use serde::Deserialize;
use tracing::instrument;
use url::Url;

use crate::error::ConfigError;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

fn default_redirect_uri() -> Url {
    Url::parse("http://localhost:4200/auth/callback").expect("default redirect URI is valid")
}

fn default_required_org() -> String {
    "bodymindarts".to_string()
}

fn default_required_team() -> String {
    "ops-team".to_string()
}

fn default_session_ttl_hours() -> i64 {
    24
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    pub github_client_id: String,
    #[serde(skip)]
    pub github_client_secret: String,
    #[serde(default = "default_redirect_uri")]
    pub github_redirect_uri: Url,
    #[serde(default = "default_required_org")]
    pub required_org: String,
    #[serde(default = "default_required_team")]
    pub required_team: String,
    #[serde(default = "default_session_ttl_hours")]
    pub session_ttl_hours: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
}

/// Secrets loaded from environment variables, kept separate from YAML config.
#[derive(Debug, Clone, Default)]
pub struct EnvSecrets {
    pub github_client_secret: Option<String>,
}

impl EnvSecrets {
    pub fn from_env() -> Self {
        Self {
            github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").ok(),
        }
    }
}

impl Config {
    /// Load config from a YAML file and inject secrets.
    #[instrument(name = "core.config.try_new", skip_all)]
    pub fn try_new(path: impl AsRef<Path>, secrets: EnvSecrets) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        let mut config: Config = serde_yaml::from_str(&contents)?;

        if let Some(ref mut auth) = config.auth {
            auth.github_client_secret = secrets.github_client_secret.ok_or_else(|| {
                ConfigError::Validation(
                    "GITHUB_CLIENT_SECRET env var required when auth is configured".into(),
                )
            })?;
        }

        Ok(config)
    }

    /// Load config from environment variables only (fallback when no config file).
    #[instrument(name = "core.config.from_env", skip_all)]
    pub fn from_env(secrets: EnvSecrets) -> Result<Self, ConfigError> {
        let port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok());
        let host = std::env::var("SERVER_HOST").ok();

        let mut config = Config::default();

        if let Some(port) = port {
            config.server.port = port;
        }
        if let Some(host) = host {
            config.server.host = host;
        }

        if let Ok(client_id) = std::env::var("GITHUB_CLIENT_ID") {
            let redirect_uri = std::env::var("GITHUB_REDIRECT_URI")
                .ok()
                .map(|u| {
                    Url::parse(&u).map_err(|_| {
                        ConfigError::Validation(format!("invalid GITHUB_REDIRECT_URI: {u}"))
                    })
                })
                .transpose()?
                .unwrap_or_else(default_redirect_uri);
            let required_org =
                std::env::var("GITHUB_REQUIRED_ORG").unwrap_or_else(|_| default_required_org());
            let required_team =
                std::env::var("GITHUB_REQUIRED_TEAM").unwrap_or_else(|_| default_required_team());
            let session_ttl_hours = std::env::var("SESSION_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_session_ttl_hours);
            let client_secret = secrets.github_client_secret.ok_or_else(|| {
                ConfigError::Validation(
                    "GITHUB_CLIENT_SECRET env var required when GITHUB_CLIENT_ID is set".into(),
                )
            })?;

            config.auth = Some(AuthConfig {
                github_client_id: client_id,
                github_client_secret: client_secret,
                github_redirect_uri: redirect_uri,
                required_org,
                required_team,
                session_ttl_hours,
            });
        }

        Ok(config)
    }
}
