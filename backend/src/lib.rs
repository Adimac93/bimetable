pub mod app_errors;
pub mod config;
pub mod modules;
pub mod routes;
pub mod utils;

use crate::modules::{AppExtensions, AppState};
use axum::{Extension, Router};

pub async fn app(state: AppState, extensions: AppExtensions) -> Router {
    Router::new()
        .nest("/auth", routes::auth::router())
        .nest("/ex", routes::example::router())
        .layer(Extension(extensions.jwt))
        .with_state(state)
}
