use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

mod graphql;

use graphql::{Mutation, Query};

type AppSchema = Schema<Query, Mutation, EmptySubscription>;

async fn graphql_handler(State(schema): State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>Agentic Ops Platform - GraphQL Playground</title>
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
</html>"#,
    )
}

fn spa_router(frontend_dir: &str) -> Router {
    let index_file = format!("{}/index.html", frontend_dir);
    Router::new().fallback_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(index_file)))
}

fn check_frontend_dir() -> Option<String> {
    let candidates = ["frontend/dist", "../frontend/dist"];
    for dir in candidates {
        let index = format!("{}/index.html", dir);
        if std::path::Path::new(&index).exists() {
            info!("Serving frontend from {}", dir);
            return Some(dir.to_string());
        }
    }
    None
}

async fn frontend_not_available() -> Response {
    Html(
        r#"<!DOCTYPE html>
<html>
<head><title>Agentic Ops Platform</title></head>
<body style="font-family: system-ui; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #0f1117; color: #e1e4ed;">
  <div style="text-align: center;">
    <h1>Agentic Ops Platform</h1>
    <p style="color: #8b8fa3;">Frontend not built. Run <code>pnpm --dir frontend install && pnpm --dir frontend build</code></p>
    <p style="margin-top: 16px;"><a href="/graphql" style="color: #6366f1;">GraphQL Playground</a></p>
  </div>
</body>
</html>"#,
    )
    .into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = agentic_ops_core::config::Config::load().expect("Failed to load config");

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
        .finish();

    let api = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(schema);

    let app = if let Some(frontend_dir) = check_frontend_dir() {
        api.merge(spa_router(&frontend_dir))
    } else {
        info!("Frontend build not found — serving placeholder");
        api.route("/", get(frontend_not_available))
    };

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}
