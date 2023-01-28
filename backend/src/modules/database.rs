use sqlx::{PgPool, migrate};
use crate::config::{PostgresSettings, ConnectionPrep};

pub async fn get_postgres_pool(config: PostgresSettings) -> PgPool {
    let pool = PgPool::connect(&config.get_connection_string())
        .await
        .expect("Cannot establish postgres connection");
    if config.is_migrating() {
        migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Auto migration failed");
    }
    pool
}
