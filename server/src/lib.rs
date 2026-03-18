pub mod auth;
pub mod error;
pub mod graphql;

use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, response::Html, routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use tracing::{info, instrument};

use agentic_ops_core::config::Config;

use crate::auth::SessionUser;
use crate::error::ServerError;
use crate::graphql::AppSchema;

async fn graphql_handler(
    State(schema): State<AppSchema>,
    session: tower_sessions::Session,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();
    if let Ok(Some(user)) = session.get::<SessionUser>("user").await {
        request = request.data(user);
    }
    schema.execute(request).await.into()
}

/// Minimal GraphiQL playground served at GET /graphql.
async fn graphql_playground() -> Html<&'static str> {
    Html(GRAPHIQL_HTML)
}

const GRAPHIQL_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>Agentic Ops - GraphQL</title>
  <link rel="stylesheet" href="https://unpkg.com/graphiql/graphiql.min.css" />
</head>
<body style="margin:0">
  <div id="graphiql" style="height:100vh"></div>
  <script crossorigin src="https://unpkg.com/react/umd/react.production.min.js"></script>
  <script crossorigin src="https://unpkg.com/react-dom/umd/react-dom.production.min.js"></script>
  <script crossorigin src="https://unpkg.com/graphiql/graphiql.min.js"></script>
  <script>
    const fetcher = GraphiQL.createFetcher({ url: '/graphql' });
    ReactDOM.render(
      React.createElement(GraphiQL, { fetcher }),
      document.getElementById('graphiql'),
    );
  </script>
</body>
</html>"#;

fn frontend_router() -> Option<Router> {
    let dist_dir = std::path::Path::new("frontend/dist");
    if !dist_dir.exists() {
        info!("frontend/dist not found, skipping static file serving");
        return None;
    }

    info!("Serving frontend from frontend/dist/");
    let index_file = dist_dir.join("index.html");
    let serve_dir = ServeDir::new(dist_dir).fallback(ServeFile::new(index_file));
    Some(Router::new().fallback_service(serve_dir))
}

#[instrument(name = "server.run", skip_all)]
pub async fn run(config: Config) -> Result<(), ServerError> {
    let schema = crate::graphql::schema();

    // TODO: MemoryStore is dev-only — sessions are lost on restart.
    // Replace with a persistent store (e.g. SQLite) for production.
    let session_store = MemoryStore::default();
    let session_ttl_hours = config
        .auth
        .as_ref()
        .map(|a| a.session_ttl_hours)
        .unwrap_or(24);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(session_ttl_hours)));

    let graphql_router = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(schema);

    let mut app = if let Some(ref auth_config) = config.auth {
        let oauth_client = auth::build_oauth_client(auth_config)?;
        let auth_state = auth::AuthState {
            oauth_client,
            config: Arc::new(auth_config.clone()),
            http_client: reqwest::Client::new(),
        };

        let auth_router = auth::router().with_state(auth_state);
        info!("Auth enabled for org={}", auth_config.required_org);

        Router::new()
            .merge(graphql_router)
            .merge(auth_router)
            .layer(session_layer)
    } else {
        info!("Auth disabled (no auth config)");
        Router::new().merge(graphql_router).layer(session_layer)
    };

    if let Some(frontend) = frontend_router() {
        app = app.merge(frontend);
    }

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    info!("Shutdown signal received, stopping server");
}
