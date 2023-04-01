use sqlx::PgPool;
use tracing_test::traced_test;

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn fixture_test(pool: PgPool) {}
