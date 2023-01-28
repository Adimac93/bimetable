pub mod config;
pub mod modules;

use crate::config::Settings;
use crate::modules::database::get_postgres_pool;

use axum::{Extension, Router};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use sqlx::PgPool;



pub async fn app(config: Settings) -> Router {
    let pgpool = get_postgres_pool(config.postgres).await;

    Router::new().route("/",get(handler)).layer(Extension(pgpool))
}

async fn handler() -> impl IntoResponse {
    Html("Hello")
}