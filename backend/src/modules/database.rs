use crate::config::database::PostgresSettings;
pub use sqlx::PgPool;
use sqlx::{migrate, PgConnection};
use tracing::info;

pub async fn get_postgres_pool(config: PostgresSettings) -> PgPool {
    info!("Connecting to Postgres database");
    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Cannot establish postgres connection");
    if config.is_migrating {
        migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Auto migration failed");
    }
    info!("Postgres Connection established");
    pool
}

pub struct PgQuery<'c, T> {
    pub payload: T,
    pub conn: &'c mut PgConnection,
}

impl<'c, T> PgQuery<'c, T>
where
    T: Send + Sync,
{
    pub fn new(payload: T, conn: &'c mut PgConnection) -> Self {
        Self { payload, conn }
    }
}
