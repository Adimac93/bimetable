use bimetable::modules::database::PgQuery;
use bimetable::routes::events::models::EventFilter;
use bimetable::routes::search::models::SearchEvents;
use bimetable::utils::events::models::{RecurrenceRule, TimeRange};
use bimetable::utils::search::{search_many_events, QueryEvent, QueryUser, Search};
use sqlx::PgPool;
use time::macros::datetime;
use tracing_test::traced_test;
use uuid::{uuid, Uuid};

const ADIMAC_ID: Uuid = uuid!("910e81a9-56df-4c24-965a-13eff739f469");
const PKBPMJ_ID: Uuid = uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972");
const HUBERT_ID: Uuid = uuid!("a9c5900e-a445-4888-8612-4a5c8cadbd9e");

#[derive(Debug, PartialEq)]
struct SimpleEvent {
    id: Uuid,
    name: String,
}

impl From<QueryEvent> for SimpleEvent {
    fn from(val: QueryEvent) -> Self {
        Self {
            id: val.id,
            name: val.name,
        }
    }
}

#[sqlx::test(fixtures("users"))]
#[traced_test]
async fn search_users_test(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let mut q = PgQuery::new(Search::new("ad".to_string()), &mut conn);
    let res = q.search_users(None).await.unwrap();

    assert_eq!(
        res,
        vec![QueryUser {
            id: uuid!("910e81a9-56df-4c24-965a-13eff739f469"),
            username: "adimac93".to_string(),
            tag: 0000,
        }]
    )
}

#[sqlx::test(fixtures("users"))]
#[traced_test]
async fn search_users_test_case_insensitive(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let mut q = PgQuery::new(Search::new("hU".to_string()), &mut conn);
    let res = q.search_users(None).await.unwrap();

    assert_eq!(
        res,
        vec![QueryUser {
            id: uuid!("a9c5900e-a445-4888-8612-4a5c8cadbd9e"),
            username: "hubertk".to_string(),
            tag: 0000,
        }]
    )
}

#[sqlx::test(fixtures("users", "events", "user_events"))]
#[traced_test]
async fn search_owned_events_test(pool: PgPool) {
    let res: Vec<SimpleEvent> = search_many_events(
        &pool,
        SearchEvents {
            text: "ma".to_string(),
            user_id: PKBPMJ_ID,
            filter: EventFilter::Owned,
        },
    )
    .await
    .unwrap()
    .into_iter()
    .map(|x| SimpleEvent::from(x))
    .collect();

    assert_eq!(
        res,
        vec![SimpleEvent {
            id: uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
            name: "Matematyka".to_string(),
        }]
    )
}

#[sqlx::test(fixtures("users", "events", "user_events"))]
#[traced_test]
async fn search_shared_events_test(pool: PgPool) {
    let res: Vec<SimpleEvent> = search_many_events(
        &pool,
        SearchEvents {
            text: "ma".to_string(),
            user_id: ADIMAC_ID,
            filter: EventFilter::Shared,
        },
    )
    .await
    .unwrap()
    .into_iter()
    .map(|x| SimpleEvent::from(x))
    .collect();

    assert_eq!(
        res,
        vec![SimpleEvent {
            id: uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
            name: "Matematyka".to_string(),
        }]
    )
}

#[sqlx::test(fixtures("users", "events", "user_events"))]
#[traced_test]
async fn search_many_events_test(pool: PgPool) {
    let mut res: Vec<SimpleEvent> = search_many_events(
        &pool,
        SearchEvents {
            text: "in".to_string(),
            user_id: HUBERT_ID,
            filter: EventFilter::All,
        },
    )
    .await
    .unwrap()
    .into_iter()
    .map(|x| SimpleEvent::from(x))
    .collect();

    res.sort_by_key(|x| x.name.to_owned());
    assert_eq!(
        res,
        vec![
            SimpleEvent {
                id: uuid!("374ae0ab-d473-4752-b77f-cae55c69245c"),
                name: "Infa".to_string(),
            },
            SimpleEvent {
                id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                name: "Informatyka".to_string(),
            },
        ]
    )
}
