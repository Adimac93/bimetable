use self::{
    database::get_postgres_pool,
    extractors::jwt::{JwtAccessSecret, JwtRefreshSecret, TokenExtractors},
};
use crate::config::get_config;
use axum::extract::FromRef;
use secrecy::Secret;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::{error, info};

pub mod database;
pub mod extractors;

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
    pub pool: PgPool,
    pub core: Core,
    pub jwt: TokenExtractors,
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
            jwt: TokenExtractors::from_settings(settings.jwt),
        }
    }

    pub fn use_custom(
        pool: PgPool,
        addr: SocketAddr,
        origin: String,
        access: Secret<String>,
        refresh: Secret<String>,
    ) -> Self {
        Self {
            pool,
            core: Core::new(addr, origin),
            jwt: TokenExtractors::new(JwtAccessSecret(access), JwtRefreshSecret(refresh)),
        }
    }

    pub fn state(self) -> AppState {
        AppState::new(self)
    }
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: TokenExtractors,
}

impl AppState {
    fn new(modules: Modules) -> Self {
        Self {
            pool: modules.pool,
            jwt: modules.jwt,
        }
    }
}

#[derive(Clone, FromRef)]
pub struct AuthState {
    pub pool: PgPool,
    pub jwt: TokenExtractors,
}

impl FromRef<AppState> for AuthState {
    fn from_ref(val: &AppState) -> Self {
        Self {
            pool: val.pool.clone(),
            jwt: val.jwt.clone(),
        }
    }
}
