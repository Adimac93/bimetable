use bimetable::{
    routes::events::models::{CreateEvent, GetEventsQuery},
    utils::events::models::{Event, EventRules, RecurrenceEndsAt, TimeRules},
};
use http::StatusCode;
use serde_json::json;
use sqlx::{query, query_as, PgPool};

use time::{macros::datetime, OffsetDateTime};
use tracing_test::traced_test;
use uuid::{uuid, Uuid};

use crate::tools::AppData;

mod tools;

// const HUBERT_ID: Uuid = uuid!("a9c5900e-a445-4888-8612-4a5c8cadbd9e");
//
// #[traced_test]
// #[sqlx::test(fixtures("users"))]
// fn create_event(pool: PgPool) {
//     let user_id = uuid!("32190025-7c15-4adb-82fd-9acc3dc8e7b6");
//     let body = CreateEvent {
//         name: "abc".into(),
//         starts_at: Some(datetime!(2023-02-05 10:00 +1)),
//         ends_at: Some(datetime!(2023-02-05 12:00 +1)),
//         recurrence_rule: Some(EventRules::Weekly {
//             time_rules: TimeRules {
//                 ends_at: Some(RecurrenceEndsAt::Until(datetime!(2040-02-05 12:00 +1))),
//                 interval: 1,
//             },
//             week_map: 0b0010011,
//         }),
//         description: "Test description".into(),
//     };
//     let rec = query_as!(
//         Event,
//         r#"
//             INSERT INTO events (name, owner_id, starts_at, ends_at , recurrence_rule, description)
//             VALUES
//             ($1, $2, $3, $4, $5, $6)
//             RETURNING id, owner_id, name, starts_at, ends_at, recurrence_rule as "recurrence_rule: _", description;
//         "#,
//         body.name,
//         user_id,
//         body.starts_at,
//         body.ends_at,
//         sqlx::types::Json(body.recurrence_rule) as _,
//         body.description,
//     )
//     .fetch_one(&pool)
//     .await.unwrap();
//
//     println!("{rec:#?}");
// }

/*
#[traced_test]
#[sqlx::test(fixtures("users"))]
async fn does_not_create_event_with_wrong_time(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let req = CreateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 8:44 +1),
        name: "Matematyka".into(),
        owner_id: Uuid::parse_str(HUBERT_ID).unwrap(),
        recurrence_rules: None,
    };

    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    // which status?
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("events"))]
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

#[traced_test]
#[sqlx::test(fixtures("users"))]
async fn create_recurring_event(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let rules = RecurrenceRules {
        week_map: Some(BitVec::from_bytes(&[0b1111100])),
        interval: Some(21),
        is_by_day: None,
    };

    let req = CreateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 9:35 +1),
        name: "Matematyka".into(),
        owner_id: Uuid::parse_str(HUBERT_ID).unwrap(),
        recurrence_rules: Some(rules),
    };

    // change path probably
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
#[sqlx::test(fixtures("users"))]
async fn does_not_create_event_with_wrong_interval(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let rules = RecurrenceRules {
        week_map: Some(BitVec::from_bytes(&[0b1111100])),
        interval: Some(0),
        is_by_day: None,
    };

    let req = CreateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 9:35 +1),
        name: "Matematyka".into(),
        owner_id: Uuid::parse_str(HUBERT_ID).unwrap(),
        recurrence_rules: Some(rules),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    // which status?
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn get_events_of_user(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();
    let user_id = Uuid::parse_str(HUBERT_ID).unwrap();

    let query = GetEventsQuery {
        starts_at: datetime!(2023-02-06 0:00 +1),
        ends_at: datetime!(2023-02-12 23:59 +1),
        user_id,
    };

    let res = client
        .get(app.api("/events"))
        .query(&query)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let mut expected: Vec<UserEvent> = Vec::new();

    (0..5).for_each(|i| {
        // it may be better to get events with user event data separated from
        // standard event data
        expected.push(UserEvent {
            id: uuid!("248c5f26-e48e-4ada-bace-384b1badb95c"),
            starts_at: datetime!(2023-02-06 8:00 +1) + Duration::days(i),
            ends_at: datetime!(2023-02-06 8:45 +1) + Duration::days(i),
            name: "Matematyka".into(),
            user_id,
            can_edit: true,
        })
    });

    let actual: Vec<Event> = res.json().await.unwrap();
    assert_eq!(actual.len(), expected.len());

    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.starts_at, expected.starts_at);
        assert_eq!(actual.ends_at, expected.ends_at);
        assert_eq!(actual.name, expected.name);
    }
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn update_event(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let rules = RecurrenceRules {
        week_map: Some(BitVec::from_bytes(&[0b1111100])),
        interval: Some(21),
        is_by_day: None,
    };

    let req = UpdateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 9:35 +1),
        name: "Polski".into(),
        recurrence_rules: Some(rules),
        user_id: Uuid::parse_str(HUBERT_ID).unwrap(),
        event_id: uuid!(248c5f26-e48e-4ada-bace-384b1badb95c),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn does_not_update_if_wrong_time(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let rules = RecurrenceRules {
        week_map: Some(BitVec::from_bytes(&[0b1111100])),
        interval: Some(21),
        is_by_day: None,
    };

    let req = UpdateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 8:44 +1),
        name: "Matematyka".into(),
        recurrence_rules: Some(rules),
        user_id: Uuid::parse_str(HUBERT_ID).unwrap(),
        event_id: uuid!(248c5f26-e48e-4ada-bace-384b1badb95c),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    // which status?
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn does_not_update_if_cannot_edit(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let rules = RecurrenceRules {
        week_map: Some(BitVec::from_bytes(&[0b1111100])),
        interval: Some(0),
        is_by_day: None,
    };

    let req = UpdateEvent {
        starts_at: datetime!(2023-02-06 8:45 +1),
        ends_at: datetime!(2023-02-06 9:35 +1),
        name: "Matematyka".into(),
        recurrence_rules: Some(rules),
        user_id: uuid!("32190025-7c15-4adb-82fd-9acc3dc8e7b6"),
        event_id: uuid!("248c5f26-e48e-4ada-bace-384b1badb95c"),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    // which status?
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn remove_event(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let req = RemoveEvent {
        event_id: uuid!("248c5f26-e48e-4ada-bace-384b1badb95c"),
        user_id: Uuid::parse_str(HUBERT_ID).unwrap(),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let _res: Uuid = res.json().await.unwrap();
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn does_not_remove_if_not_owner(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let req = RemoveEvent {
        event_id: uuid!("248c5f26-e48e-4ada-bace-384b1badb95c"),
        user_id: uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972"),
    };

    // change path probably
    let res = client
        .put(app.api("/events"))
        .json(&req)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let _res: Uuid = res.json().await.unwrap();
}
*/
