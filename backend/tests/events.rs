use bimetable::{
    routes::events::models::{CreateEvent, GetEventsQuery},
    utils::events::models::{EndsAt, Event, EventRules, TimeRules},
};
use http::StatusCode;
use serde_json::json;
use sqlx::{query, query_as, PgPool};
use time::macros::datetime;
use tracing_test::traced_test;
use uuid::{uuid, Uuid};

use crate::tools::AppData;

mod tools;

#[traced_test]
#[sqlx::test(fixtures("users"))]
fn create_event(pool: PgPool) {
    let user_id = uuid!("32190025-7c15-4adb-82fd-9acc3dc8e7b6");
    let body = CreateEvent {
        name: "abc".into(),
        starts_at: Some(datetime!(2023-02-05 10:00 +1)),
        ends_at: Some(datetime!(2023-02-05 12:00 +1)),
        recurrence_rule: Some(EventRules::Weekly {
            time_rules: TimeRules {
                ends_at: Some(EndsAt::Until(datetime!(2040-02-05 12:00 +1))),
                interval: 1,
            },
            week_map: 0b0010011,
        }),
    };
    let rec = query_as!(
        Event,
        r#"
            INSERT INTO events (name, owner_id, starts_at, ends_at , recurrence_rule)
            VALUES
            ($1, $2, $3, $4, $5)
            RETURNING id, owner_id, name, starts_at, ends_at, recurrence_rule as "recurrence_rule: _";
        "#,
        body.name,
        user_id,
        body.starts_at,
        body.ends_at,
        sqlx::types::Json(body.recurrence_rule) as _
    )
    .fetch_one(&pool)
    .await.unwrap();

    println!("{rec:#?}");
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn get_events_in_time_range(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let query = GetEventsQuery {
        starts_at: Some(datetime!(2023-02-06 8:00 +1)),
        ends_at: Some(datetime!(2023-02-06 9:35 +1)),
    };

    let payload = json!({
        "login": "pkbpkp",
        "password": "#strong#_#pass#",
    });

    let res = client
        .post(app.api("/auth/login"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get(app.api("/events"))
        .query(&query)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let expected = vec![
        Event {
            id: uuid!("248c5f26-e48e-4ada-bace-384b1badb95c"),
            owner_id: uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972"),
            starts_at: Some(datetime!(2023-02-06 8:00 +1)),
            ends_at: Some(datetime!(2023-02-06 8:45 +1)),
            recurrence_rule: None,
            name: "Matematyka".into(),
        },
        Event {
            id: uuid!("73cb2256-80ce-4c0b-b753-34fdd2c7f5e5"),
            owner_id: uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972"),
            starts_at: Some(datetime!(2023-02-06 8:50 +1)),
            ends_at: Some(datetime!(2023-02-06 9:35 +1)),
            recurrence_rule: None,
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
