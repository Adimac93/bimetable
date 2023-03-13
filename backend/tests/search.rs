use bimetable::modules::database::PgQuery;
use bimetable::utils::search::{QueryUser, Search};
use sqlx::PgPool;
use tracing_test::traced_test;
use uuid::uuid;

#[sqlx::test(fixtures("users"))]
#[traced_test]
async fn search_users_test(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let mut q = PgQuery::new(Search::new("ad".to_string()), &mut conn);
    let res = q.search_users().await.unwrap();

    assert_eq!(
        res,
        vec![QueryUser {
            id: uuid!("910e81a9-56df-4c24-965a-13eff739f469"),
            username: "adimac93".to_string(),
            tag: 0000,
        }]
    )
}
