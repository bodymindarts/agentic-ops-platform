use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, response::Html, routing::get, Router};
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

    let app = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(schema);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}
