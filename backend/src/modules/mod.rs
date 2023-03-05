use self::database::get_postgres_pool;
use crate::config::app::ApplicationSettings;
use crate::config::environment::Environment;
use crate::config::get_config;
use crate::config::tokens::JwtSettings;
use axum::extract::FromRef;
use core::fmt::Display;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::{error, info};

pub mod database;
pub mod extensions;

pub struct Modules {
    pub app: ApplicationSettings,
    pool: PgPool,
    jwt: JwtSettings,
    environment: Environment,
}

impl Modules {
    pub async fn load_from_settings() -> Self {
        let settings = get_config()
            .map_err(|e| error!("Failed to load settings {e:#?}"))
            .unwrap();
        info!("Settings loaded");
        info!("Loading modules");
        let pool = get_postgres_pool(settings.postgres).await;
        info!("Modules loaded");
        Self {
            pool,
            app: settings.app,
            jwt: settings.jwt,
            environment: settings.environment,
        }
    }

    pub fn use_custom(
        pool: PgPool,
        addr: SocketAddr,
        origin: String,
        access: &str,
        refresh: &str,
        environment: Environment,
    ) -> Self {
        Self {
            pool,
            app: ApplicationSettings::new(addr, origin),
            jwt: JwtSettings::new(access, refresh),
            environment,
        }
    }

    pub fn state(&self) -> AppState {
        AppState::new(self)
    }

    pub fn extensions(&self) -> AppExtensions {
        AppExtensions::new(self)
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub environment: Environment,
    pub pool: PgPool,
}

impl AppState {
    fn new(modules: &Modules) -> Self {
        Self {
            environment: modules.environment.clone(),
            pool: modules.pool.clone(),
        }
    }
}

impl Display for AppState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "postgres pool")
    }
}

pub struct AppExtensions {
    pub jwt: JwtSettings,
}

impl AppExtensions {
    fn new(modules: &Modules) -> Self {
        Self {
            jwt: modules.jwt.clone(),
        }
    }
}

impl Display for AppExtensions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "token secrets")
    }
}
