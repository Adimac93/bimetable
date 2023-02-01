pub mod app_errors;
pub mod config;
pub mod modules;
pub mod routes;
pub mod utils;

use crate::modules::{AppExtensions, AppState};
use axum::{Extension, Router, middleware};
use utils::auth::{middleware::auth_access_middleware, models::Claims};

pub async fn app(state: AppState, extensions: AppExtensions) -> Router {
    Router::new()
        .nest("/events", routes::events::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_access_middleware))
        .nest("/ex", routes::example::router())
        .nest("/auth", routes::auth::router(state.clone()))
        .layer(Extension(extensions.jwt))
        .with_state(state)
}
