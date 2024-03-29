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

    info!("Starting server on {} machine", machine_kind());
    info!("Listening on {}", &modules.app.addr);
    axum::Server::bind(&modules.app.addr)
        .serve(
            app(modules)
                .await
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Failed to run axum server");
}

fn machine_kind<'s>() -> &'s str {
    if cfg!(unix) {
        "unix"
    } else if cfg!(windows) {
        "windows"
    } else {
        "unknown"
    }
}
