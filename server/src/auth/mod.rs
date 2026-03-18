use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointSet, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, StandardRevocableToken, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use agentic_ops_core::config::AuthConfig;

use crate::error::AuthError;

const PKCE_VERIFIER_KEY: &str = "pkce_verifier";
const CSRF_STATE_KEY: &str = "csrf_state";
const SESSION_USER_KEY: &str = "user";

type OAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct AuthState {
    pub oauth_client: OAuthClient,
    pub config: Arc<AuthConfig>,
    pub http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub github_username: String,
    pub github_user_id: u64,
    pub avatar_url: String,
    pub display_name: String,
    pub team_verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    name: Option<String>,
}

pub fn router() -> Router<AuthState> {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .route("/auth/me", get(me))
}

pub fn build_oauth_client(config: &AuthConfig) -> Result<OAuthClient, AuthError> {
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
        .map_err(|_| AuthError::ConfigError("GITHUB_CLIENT_SECRET env var not set".into()))?;

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .map_err(|e| AuthError::ConfigError(e.to_string()))?;
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .map_err(|e| AuthError::ConfigError(e.to_string()))?;
    let redirect_url = RedirectUrl::new(config.github_redirect_uri.clone())
        .map_err(|e| AuthError::ConfigError(e.to_string()))?;

    let client = oauth2::Client::new(ClientId::new(config.github_client_id.clone()))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url);

    Ok(client)
}

#[tracing::instrument(name = "server.auth.login", skip_all)]
async fn login(State(state): State<AuthState>, session: Session) -> Result<Response, AuthError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_state) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();

    session
        .insert(PKCE_VERIFIER_KEY, pkce_verifier.secret().to_string())
        .await
        .map_err(|e: tower_sessions::session::Error| AuthError::SessionError(e.to_string()))?;
    session
        .insert(CSRF_STATE_KEY, csrf_state.secret().to_string())
        .await
        .map_err(|e: tower_sessions::session::Error| AuthError::SessionError(e.to_string()))?;

    Ok(Redirect::temporary(auth_url.as_str()).into_response())
}

#[tracing::instrument(name = "server.auth.callback", skip_all)]
async fn callback(
    State(state): State<AuthState>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Result<Response, AuthError> {
    let stored_state: String = session
        .get(CSRF_STATE_KEY)
        .await
        .map_err(|e| AuthError::SessionError(e.to_string()))?
        .ok_or(AuthError::InvalidState)?;

    if params.state != stored_state {
        return Err(AuthError::InvalidState);
    }

    let pkce_secret: String = session
        .get(PKCE_VERIFIER_KEY)
        .await
        .map_err(|e| AuthError::SessionError(e.to_string()))?
        .ok_or(AuthError::InvalidState)?;
    let pkce_verifier = PkceCodeVerifier::new(pkce_secret);

    let http_client = oauth2::reqwest::Client::new();
    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

    let access_token = token_result.access_token().secret();

    let github_user = fetch_github_user(&state.http_client, access_token).await?;

    check_team_membership(
        &state.http_client,
        access_token,
        &state.config.required_org,
        &state.config.required_team,
        &github_user.login,
    )
    .await?;

    let session_user = SessionUser {
        github_username: github_user.login,
        github_user_id: github_user.id,
        avatar_url: github_user.avatar_url,
        display_name: github_user.name.unwrap_or_default(),
        team_verified: true,
    };

    session.remove::<String>(PKCE_VERIFIER_KEY).await.ok();
    session.remove::<String>(CSRF_STATE_KEY).await.ok();

    session
        .insert(SESSION_USER_KEY, &session_user)
        .await
        .map_err(|e: tower_sessions::session::Error| AuthError::SessionError(e.to_string()))?;

    Ok(Redirect::temporary("/graphql").into_response())
}

#[tracing::instrument(name = "server.auth.logout", skip_all)]
async fn logout(session: Session) -> Result<Response, AuthError> {
    session
        .flush()
        .await
        .map_err(|e| AuthError::SessionError(e.to_string()))?;
    Ok(StatusCode::OK.into_response())
}

#[tracing::instrument(name = "server.auth.me", skip_all)]
async fn me(session: Session) -> Result<Json<SessionUser>, AuthError> {
    let user: SessionUser = session
        .get(SESSION_USER_KEY)
        .await
        .map_err(|e| AuthError::SessionError(e.to_string()))?
        .ok_or(AuthError::Unauthorized)?;
    Ok(Json(user))
}

#[tracing::instrument(name = "server.auth.fetch_github_user", skip_all)]
async fn fetch_github_user(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<GitHubUser, AuthError> {
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "agentic-ops-platform")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AuthError::GitHubApi(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(AuthError::GitHubApi(format!(
            "GitHub user API returned {}",
            resp.status()
        )));
    }

    resp.json::<GitHubUser>()
        .await
        .map_err(|e| AuthError::GitHubApi(e.to_string()))
}

#[tracing::instrument(name = "server.auth.check_team_membership", skip_all)]
async fn check_team_membership(
    client: &reqwest::Client,
    access_token: &str,
    org: &str,
    team_slug: &str,
    username: &str,
) -> Result<(), AuthError> {
    let url = format!("https://api.github.com/orgs/{org}/teams/{team_slug}/memberships/{username}");

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "agentic-ops-platform")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AuthError::GitHubApi(e.to_string()))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(AuthError::TeamCheckFailed)
    }
}

/// Extractor that retrieves the authenticated user from the session.
/// Returns 401 if no valid session exists.
#[allow(dead_code)]
pub struct AuthenticatedUser(pub SessionUser);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|(status, msg)| AuthError::SessionError(format!("{status}: {msg}")))?;

        let user: SessionUser = session
            .get(SESSION_USER_KEY)
            .await
            .map_err(|e| AuthError::SessionError(e.to_string()))?
            .ok_or(AuthError::Unauthorized)?;

        Ok(AuthenticatedUser(user))
    }
}
