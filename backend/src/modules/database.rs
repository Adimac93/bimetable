use crate::config::{ConnectionPrep, PostgresSettings};
use sqlx::migrate;
pub use sqlx::PgPool;
use tracing::info;

pub async fn get_postgres_pool(config: PostgresSettings) -> PgPool {
    info!("Connecting to Postgres database");
    let pool = PgPool::connect(&config.get_connection_string())
        .await
        .expect("Cannot establish postgres connection");
    if config.is_migrating() {
        migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Auto migration failed");
    }
    info!("Postgres Connection established");
    pool
}
