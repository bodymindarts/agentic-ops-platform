use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("AuthError - InvalidState: CSRF state mismatch")]
    InvalidState,
    #[error("AuthError - TokenExchange: {0}")]
    TokenExchange(String),
    #[error("AuthError - GitHubApi: {0}")]
    GitHubApi(String),
    #[error("AuthError - TeamCheckFailed: user is not a member of the required team")]
    TeamCheckFailed,
    #[error("AuthError - Unauthorized")]
    Unauthorized,
    #[error("AuthError - SessionError: {0}")]
    SessionError(String),
    #[error("AuthError - ConfigError: {0}")]
    ConfigError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AuthError::InvalidState => (StatusCode::BAD_REQUEST, "Invalid OAuth state"),
            AuthError::TokenExchange(e) => {
                tracing::error!(error = %e, "Token exchange failed");
                (StatusCode::BAD_GATEWAY, "Failed to exchange token")
            }
            AuthError::GitHubApi(e) => {
                tracing::error!(error = %e, "GitHub API error");
                (StatusCode::BAD_GATEWAY, "GitHub API error")
            }
            AuthError::TeamCheckFailed => (StatusCode::FORBIDDEN, "Not a member of required team"),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::SessionError(e) => {
                tracing::error!(error = %e, "Session error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Session error")
            }
            AuthError::ConfigError(e) => {
                tracing::error!(error = %e, "Auth config error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authentication not configured",
                )
            }
        };

        (status, msg).into_response()
    }
}
