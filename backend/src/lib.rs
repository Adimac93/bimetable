pub mod app_errors;
pub mod config;
mod doc;
pub mod modules;
pub mod routes;
pub mod utils;

use crate::modules::{AppExtensions, AppState};
use axum::{Extension, Router};
use modules::extensions::jwt::{JwtAccessSecret, JwtRefreshSecret, TokenSecrets};
use secrecy::Secret;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub async fn app(state: AppState, extensions: AppExtensions) -> Router {
    info!("Spawning main router with:\n - state: {state}\n - extensions: {extensions}");

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", doc::ApiDoc::openapi()))
        .nest("/auth", routes::auth::router())
        .nest("/ex", routes::example::router())
        .nest("/events", routes::events::router())
        .layer(Extension(extensions.jwt))
        .with_state(state)
}

struct BRouter {
    router: Router,
}

impl BRouter {
    fn add_extension<T>(mut self, extension: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.router = self.router.layer(Extension(extension));
        self
    }

    fn add_extensions<T>(mut self, extensions: Vec<T>)
    where
        T: Clone + Send + Sync + 'static,
    {
        for extension in extensions {
            println!("Adding ext");
            self = self.add_extension(extension)
        }
    }
}

#[test]
fn brouter() {
    let ext = AppExtensions {
        jwt: TokenSecrets::new(
            JwtAccessSecret(Secret::from(String::from("A"))),
            JwtRefreshSecret(Secret::from(String::from("B"))),
        ),
    };
    let router = BRouter {
        router: Router::new(),
    };
    router.add_extensions(vec![ext.jwt.clone(), ext.jwt.clone()])
}
