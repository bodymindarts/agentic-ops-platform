use agentic_ops_core::config::{Config, EnvSecrets};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let secrets = EnvSecrets::from_env();
    let config = match std::env::var("CONFIG_FILE") {
        Ok(path) => Config::try_new(path, secrets).expect("Failed to load config from file"),
        Err(_) => Config::from_env(secrets).expect("Failed to load config from env"),
    };

    server::run(config).await.expect("Server error");
}
