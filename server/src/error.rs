use thiserror::Error;

use crate::auth::AuthError;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("ServerError - Auth: {0}")]
    Auth(#[from] AuthError),
    #[error("ServerError - Config: {0}")]
    Config(#[from] agentic_ops_core::error::ConfigError),
    #[error("ServerError - Io: {0}")]
    Io(#[from] std::io::Error),
}
