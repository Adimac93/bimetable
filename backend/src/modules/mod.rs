use self::{
    database::get_postgres_pool,
    extensions::jwt::{JwtAccessSecret, JwtRefreshSecret, TokenSecrets},
};
use crate::config::{get_config, Environment};
use axum::extract::FromRef;
use core::fmt::Display;
use secrecy::Secret;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::{error, info};

pub mod database;
pub mod extensions;

pub struct Core {
    pub addr: SocketAddr,
    pub origin: String,
}

impl Core {
    fn new(addr: SocketAddr, origin: String) -> Self {
        Self { addr, origin }
    }
}

pub struct Modules {
    pub core: Core,
    pool: PgPool,
    jwt: TokenSecrets,
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
        let addr = settings.app.get_addr();
        let origin = settings.app.origin;
        info!("Modules loaded");
        Self {
            pool,
            core: Core::new(addr, origin),
            jwt: TokenSecrets::from_settings(settings.jwt),
            environment: settings.environment,
        }
    }

    pub fn use_custom(
        pool: PgPool,
        addr: SocketAddr,
        origin: String,
        access: Secret<String>,
        refresh: Secret<String>,
        environment: Environment,
    ) -> Self {
        Self {
            pool,
            core: Core::new(addr, origin),
            jwt: TokenSecrets::new(JwtAccessSecret(access), JwtRefreshSecret(refresh)),
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
    pub jwt: TokenSecrets,
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
