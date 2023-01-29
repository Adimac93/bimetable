pub mod app_errors;
pub mod config;
pub mod modules;
pub mod routes;
pub mod utils;

use crate::modules::AppState;
use axum::Router;

pub async fn app(state: AppState) -> Router {
    Router::new()
        .nest("/ex", routes::example::router())
        .with_state(state)
}
