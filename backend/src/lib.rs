pub mod app_errors;
pub mod config;
mod doc;
pub mod modules;
pub mod routes;
pub mod utils;
pub mod validation;

use crate::config::Environment;
use crate::modules::{AppState, Modules};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Router};
use http::{StatusCode, Uri};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

const SWAGGER_URI: &str = "/swagger-ui";

pub async fn app(modules: Modules) -> Router {
    let mut router = Router::new();
    let state = modules.state();
    let extensions = modules.extensions();

    info!("Environment: {}", state.environment);
    if state.environment.is_dev() {
        info!("Enabling Swagger UI");
        router = router.merge(
            SwaggerUi::new(SWAGGER_URI).url("/api-doc/openapi.json", doc::ApiDoc::openapi()),
        );
    }

    info!("Spawning main router with:\n - state: {state}\n - extensions: {extensions}");

    router
        .nest("/auth", routes::auth::router())
        .nest("/ex", routes::example::router())
        .nest(
            "/events",
            routes::events::router().nest("/invitations", routes::invitations::router()),
        )
        .layer(Extension(extensions.jwt))
        .fallback(not_found)
        .with_state(state)
}

async fn not_found(
    State(environment): State<Environment>,
    uri: Uri,
) -> Result<Redirect, (StatusCode, &'static str)> {
    if environment.is_dev() && uri.path() == "/" {
        return Ok(Redirect::to(SWAGGER_URI));
    }
    Err((StatusCode::NOT_FOUND, "404 Not Found"))
}
