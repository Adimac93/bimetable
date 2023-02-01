use bimetable::routes::events::models::{CreateEvent, Event, GetEventsQuery};
use http::StatusCode;
use sqlx::PgPool;
use time::macros::datetime;
use time::OffsetDateTime;
use tracing_test::traced_test;
use uuid::Uuid;

use crate::tools::AppData;

mod tools;

#[traced_test]
#[sqlx::test]
async fn create_event(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let req = CreateEvent {
        starts_at: OffsetDateTime::from_unix_timestamp(1000).unwrap(),
        ends_at: OffsetDateTime::from_unix_timestamp(2000).unwrap(),
        name: "Foo".into(),
    };

    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("events"))]
async fn get_events_in_time_range(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let query = GetEventsQuery {
        starts_at: datetime!(2023-02-06 8:00 UTC),
        ends_at: datetime!(2023-02-06 9:35 UTC),
    };

    let res = client
        .get(app.api("/events"))
        .query(&query)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let expected = vec![
        CreateEvent {
            starts_at: datetime!(2023-02-06 8:00 UTC),
            ends_at: datetime!(2023-02-06 8:45 UTC),
            name: "Matematyka".into(),
        },
        CreateEvent {
            starts_at: datetime!(2023-02-06 8:50 UTC),
            ends_at: datetime!(2023-02-06 9:35 UTC),
            name: "Fizyka".into(),
        },
    ];

    let actual: Vec<Event> = res.json().await.unwrap();
    assert_eq!(actual.len(), expected.len());

    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.starts_at, expected.starts_at);
        assert_eq!(actual.ends_at, expected.ends_at);
        assert_eq!(actual.name, expected.name);
    }
}
