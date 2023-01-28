use self::database::get_postgres_pool;
use crate::config::get_config;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::{error, info};

pub mod database;

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
        }
    }

    pub fn use_custom(pool: PgPool, addr: SocketAddr, origin: String) -> Self {
        Self {
            pool,
            core: Core::new(addr, origin),
        }
    }
}
