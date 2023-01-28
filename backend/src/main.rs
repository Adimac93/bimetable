use bimetable::app;
use bimetable::config::get_config;
use dotenv::dotenv;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bimetable=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting server");

    let config = get_config().expect("Failed to read configuration");
    info!("Configuration loaded");

    let addr = config.app.get_addr();

    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(
            app(config)
                .await
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Failed to run axum server");
}
