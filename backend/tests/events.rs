use std::collections::HashMap;

use bimetable::{
    modules::database::PgQuery,
    routes::events::models::{
        CreateEvent, Entry, Event, EventData, EventFilter, EventPayload, Events, GetEventsQuery,
        OptionalEventData, UpdateEvent,
    },
    utils::events::{
        errors::EventError,
        get_many_events,
        models::{RecurrenceEndsAt, RecurrenceRule, TimeRange, TimeRules},
        EventQuery,
    },
};
use http::StatusCode;
use serde_json::json;
use sqlx::{query, query_as, PgPool};

use bimetable::utils::events::models::RecurrenceRuleKind;
use time::{macros::datetime, OffsetDateTime};
use tracing::{debug, trace};
use tracing_test::traced_test;
use uuid::{uuid, Uuid};

use crate::tools::AppData;

mod tools;

const ADIMAC_ID: Uuid = uuid!("910e81a9-56df-4c24-965a-13eff739f469");
const PKBPMJ_ID: Uuid = uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972");
const MABI19_ID: Uuid = uuid!("32190025-7c15-4adb-82fd-9acc3dc8e7b6");
const HUBERT_ID: Uuid = uuid!("a9c5900e-a445-4888-8612-4a5c8cadbd9e");

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn create_event_test(pool: PgPool) {
    let event = CreateEvent {
        data: EventData {
            starts_at: datetime!(2023-03-07 19:00 UTC),
            ends_at: datetime!(2023-03-07 20:00 UTC),
            payload: EventPayload {
                name: "New event".to_string(),
                description: None,
            },
        },
        recurrence_rule: None,
    };

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);

    let event_id = query.create_event(event).await;
    trace!("{:?}", event_id);
    let event_id = event_id.unwrap();

    let get_result = query.get_event(event_id).await;
    trace!("{:?}", get_result);
    let get_result = get_result.unwrap();

    assert_eq!(
        get_result,
        Some(Event {
            can_edit: true,
            is_owned: true,
            payload: EventPayload {
                name: "New event".to_string(),
                description: None
            },
            recurrence_rule: None,
        })
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events"))]
async fn does_not_create_event_with_wrong_time(pool: PgPool) {
    let event = CreateEvent {
        data: EventData {
            starts_at: datetime!(2023-03-07 19:00 UTC),
            ends_at: datetime!(2023-03-07 18:59 UTC),
            payload: EventPayload {
                name: "New event".to_string(),
                description: None,
            },
        },
        recurrence_rule: None,
    };

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);

    assert!(query.create_event(event).await.is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn get_many_events_test(pool: PgPool) {
    let res = get_many_events(
        HUBERT_ID,
        TimeRange::new(
            datetime!(2023-03-06 0:00 UTC),
            datetime!(2023-03-13 0:00 UTC),
        ),
        EventFilter::All,
        pool,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Events {
            events: HashMap::from([
                (
                    uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    Event {
                        can_edit: true,
                        is_owned: true,
                        recurrence_rule: Some(RecurrenceRule {
                            kind: RecurrenceRuleKind::Weekly { week_map: 40 },
                            time_rules: TimeRules {
                                ends_at: Some(RecurrenceEndsAt::Count(15)),
                                interval: 1,
                            },
                        }),
                        payload: EventPayload {
                            name: "Informatyka".to_string(),
                            description: None,
                        }
                    }
                ),
                (
                    uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    Event {
                        can_edit: true,
                        is_owned: false,
                        recurrence_rule: Some(RecurrenceRule {
                            kind: RecurrenceRuleKind::Weekly { week_map: 24 },
                            time_rules: TimeRules {
                                ends_at: Some(RecurrenceEndsAt::Count(15)),
                                interval: 1,
                            },
                        }),
                        payload: EventPayload {
                            name: "Fizyka".to_string(),
                            description: Some("fizyka kwantowa :O".to_string()),
                        }
                    }
                )
            ]),
            entries: vec![
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    starts_at: datetime!(2023-03-07 11:40 UTC),
                    ends_at: datetime!(2023-03-07 13:15 UTC),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    starts_at: datetime!(2023-03-08 09:45 UTC),
                    ends_at: datetime!(2023-03-08 10:30 UTC),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    starts_at: datetime!(2023-03-09 09:45 UTC),
                    ends_at: datetime!(2023-03-09 10:30 UTC),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    starts_at: datetime!(2023-03-09 11:40 UTC),
                    ends_at: datetime!(2023-03-09 13:15 UTC),
                    recurrence_override: None,
                },
            ],
        }
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn get_owned_test(pool: PgPool) {
    let res = get_many_events(
        HUBERT_ID,
        TimeRange::new(
            datetime!(2023-03-06 0:00 UTC),
            datetime!(2023-03-13 0:00 UTC),
        ),
        EventFilter::Owned,
        pool,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Events {
            events: HashMap::from([(
                uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                Event {
                    can_edit: true,
                    is_owned: true,
                    recurrence_rule: Some(RecurrenceRule {
                        kind: RecurrenceRuleKind::Weekly { week_map: 40 },
                        time_rules: TimeRules {
                            ends_at: Some(RecurrenceEndsAt::Count(15)),
                            interval: 1,
                        },
                    }),
                    payload: EventPayload {
                        name: "Informatyka".to_string(),
                        description: None,
                    }
                }
            ),]),
            entries: vec![
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    starts_at: datetime!(2023-03-07 11:40 UTC),
                    ends_at: datetime!(2023-03-07 13:15 UTC),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    starts_at: datetime!(2023-03-09 11:40 UTC),
                    ends_at: datetime!(2023-03-09 13:15 UTC),
                    recurrence_override: None,
                },
            ],
        }
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn get_shared_test(pool: PgPool) {
    let res = get_many_events(
        HUBERT_ID,
        TimeRange::new(
            datetime!(2023-03-06 0:00 UTC),
            datetime!(2023-03-13 0:00 UTC),
        ),
        EventFilter::Shared,
        pool,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Events {
            events: HashMap::from([(
                uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                Event {
                    can_edit: true,
                    is_owned: false,
                    recurrence_rule: Some(RecurrenceRule {
                        kind: RecurrenceRuleKind::Weekly { week_map: 24 },
                        time_rules: TimeRules {
                            ends_at: Some(RecurrenceEndsAt::Count(15)),
                            interval: 1,
                        },
                    }),
                    payload: EventPayload {
                        name: "Fizyka".to_string(),
                        description: Some("fizyka kwantowa :O".to_string()),
                    }
                }
            )]),
            entries: vec![
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    starts_at: datetime!(2023-03-08 09:45 UTC),
                    ends_at: datetime!(2023-03-08 10:30 UTC),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    starts_at: datetime!(2023-03-09 09:45 UTC),
                    ends_at: datetime!(2023-03-09 10:30 UTC),
                    recurrence_override: None,
                },
            ],
        }
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn update_event_test(pool: PgPool) {
    let event_id = uuid!("6d185de5-ddec-462a-aeea-7628f03d417b");

    let data = OptionalEventData {
        name: Some("Polski".to_string()),
        description: Some("niespodzianka!!".to_string()),
        starts_at: None,
        ends_at: None,
    };

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(PKBPMJ_ID), &mut conn);

    query.update_event(event_id, data).await.unwrap();

    assert_eq!(
        query.get_event(event_id).await.unwrap(),
        Some(Event {
            can_edit: true,
            is_owned: true,
            recurrence_rule: Some(RecurrenceRule {
                kind: RecurrenceRuleKind::Monthly { is_by_day: true },
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 1,
                },
            }),
            payload: EventPayload {
                name: "Polski".to_string(),
                description: Some("niespodzianka!!".to_string()),
            },
        })
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_update_event_without_permissions(pool: PgPool) {
    let data = OptionalEventData {
        name: Some("Polski".to_string()),
        description: Some("niespodzianka!!".to_string()),
        starts_at: None,
        ends_at: None,
    };

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(MABI19_ID), &mut conn);

    assert!(query
        .update_event(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"), data)
        .await
        .is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn delete_event_test(pool: PgPool) {
    let event_id = uuid!("6d185de5-ddec-462a-aeea-7628f03d417b");

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(PKBPMJ_ID), &mut conn);

    query.perm_delete(event_id).await.unwrap();

    assert!(query.get_event(event_id).await.unwrap().is_none())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_delete_event_if_not_owned(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);

    assert!(query
        .perm_delete(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
        .await
        .is_err())
}
