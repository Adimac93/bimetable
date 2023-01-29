use bimetable::app;
use bimetable::modules::Modules;
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

    let modules = Modules::load_from_settings().await;

    info!("Starting server");
    info!("Listening on {}", modules.core.addr);
    axum::Server::bind(&modules.core.addr)
        .serve(
            app(modules.state())
                .await
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Failed to run axum server");
}
