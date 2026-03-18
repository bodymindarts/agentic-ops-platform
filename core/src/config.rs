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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

impl Config {
    pub fn load() -> Result<Self, CoreError> {
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

        Ok(config)
    }
}
