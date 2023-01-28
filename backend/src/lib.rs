pub mod app_errors;
pub mod config;
pub mod modules;
pub mod routes;
pub mod utils;

use crate::modules::Modules;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Extension, Router};

pub async fn app(modules: Modules) -> Router {
    Router::new()
        .route("/", get(handler))
        .layer(Extension(modules.pool))
}

async fn handler() -> impl IntoResponse {
    Html("Hello")
}
