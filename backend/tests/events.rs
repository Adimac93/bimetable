use std::collections::HashMap;

use bimetable::{
    modules::database::PgQuery,
    routes::events::models::{
        CreateEvent, Entry, Event, EventData, EventFilter, EventPayload, Events, GetEventsQuery,
        OptionalEventData, UpdateEditPrivilege, UpdateEvent,
    },
    utils::events::{
        errors::EventError,
        exe::{
            delete_one_event_permanently, delete_owner_from_event, delete_user_event,
            get_many_events, set_event_ownership, update_user_editing_privileges,
        },
        models::{RecurrenceRule, TimeRange},
        EventQuery,
    },
};
use http::StatusCode;
use serde_json::json;
use sqlx::{query, query_as, PgPool};

use bimetable::routes::events::create_event;
use bimetable::utils::events::exe::{create_new_event, get_one_event, update_one_event};
use bimetable::utils::events::models::{EntriesSpan, RecurrenceRuleKind};
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
            entries_start: datetime!(2023-03-07 19:00 UTC),
            entries_end: Some(datetime!(2023-03-07 20:00 UTC)),
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

    assert!(create_new_event(&pool, ADIMAC_ID, event).await.is_err())
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
        &pool,
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
                            span: Some(EntriesSpan {
                                end: datetime!(2023-04-27 13:15:00.0 +00:00:00),
                                repetitions: 15,
                            }),
                            interval: 1,
                            kind: RecurrenceRuleKind::Weekly { week_map: 40 },
                        }),
                        entries_start: datetime!(2023-03-07 11:40 UTC),
                        entries_end: Some(datetime!(2023-04-27 13:15 UTC)),
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
                            span: Some(EntriesSpan {
                                end: datetime!(2023-04-27 10:30:00.0 +00:00:00),
                                repetitions: 15,
                            }),
                            interval: 1,
                            kind: RecurrenceRuleKind::Weekly { week_map: 24 },
                        }),
                        entries_start: datetime!(2023-03-08 09:45 +00:00:00),
                        entries_end: Some(datetime!(2023-04-27 10:30 UTC)),
                        payload: EventPayload {
                            name: "Fizyka".to_string(),
                            description: Some("fizyka kwantowa :O".to_string()),
                        }
                    }
                ),
                (
                    uuid!("374ae0ab-d473-4752-b77f-cae55c69245c"),
                    Event {
                        can_edit: true,
                        is_owned: false,
                        recurrence_rule: None,
                        entries_start: datetime!(2023-03-07 11:30:00.0 +00:00:00),
                        entries_end: Some(datetime!(2023-03-07 13:15:00.0 +00:00:00)),
                        payload: EventPayload {
                            name: "Infa".to_string(),
                            description: None,
                        }
                    }
                )
            ]),
            entries: vec![
                // Entry {
                //     event_id: uuid!("374ae0ab-d473-4752-b77f-cae55c69245c"),
                //     starts_at: datetime!(2023-03-07 11:30 UTC),
                //     ends_at: datetime!(2023-03-07 13:15 UTC),
                //     recurrence_override: None,
                // },
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-07 11:40 UTC),
                        datetime!(2023-03-07 13:15 UTC)
                    ),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-08 09:45 UTC),
                        datetime!(2023-03-08 10:30 UTC)
                    ),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-09 09:45 UTC),
                        datetime!(2023-03-09 10:30 UTC)
                    ),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-09 11:40 UTC),
                        datetime!(2023-03-09 13:15 UTC)
                    ),
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
        &pool,
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
                        span: Some(EntriesSpan {
                            end: datetime!(2023-04-27 13:15:00.0 +00:00:00),
                            repetitions: 15,
                        }),
                        interval: 1,
                        kind: RecurrenceRuleKind::Weekly { week_map: 40 },
                    }),
                    entries_start: datetime!(2023-03-07 11:40 +00:00:00),
                    entries_end: Some(datetime!(2023-04-27 13:15 UTC)),
                    payload: EventPayload {
                        name: "Informatyka".to_string(),
                        description: None,
                    }
                }
            ),]),
            entries: vec![
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-07 11:40 UTC),
                        datetime!(2023-03-07 13:15 UTC)
                    ),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-09 11:40 UTC),
                        datetime!(2023-03-09 13:15 UTC)
                    ),
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
        &pool,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Events {
            events: HashMap::from([
                (
                    uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    Event {
                        can_edit: true,
                        is_owned: false,
                        recurrence_rule: Some(RecurrenceRule {
                            span: Some(EntriesSpan {
                                end: datetime!(2023-04-27 10:30:00.0 +00:00:00),
                                repetitions: 15,
                            }),
                            interval: 1,
                            kind: RecurrenceRuleKind::Weekly { week_map: 24 },
                        }),
                        entries_start: datetime!(2023-03-08 09:45 +00:00:00),
                        entries_end: Some(datetime!(2023-04-27 10:30 UTC)),
                        payload: EventPayload {
                            name: "Fizyka".to_string(),
                            description: Some("fizyka kwantowa :O".to_string()),
                        }
                    }
                ),
                (
                    uuid!("374ae0ab-d473-4752-b77f-cae55c69245c"),
                    Event {
                        can_edit: true,
                        is_owned: false,
                        recurrence_rule: None,
                        entries_start: datetime!(2023-03-07 11:30:00.0 +00:00:00),
                        entries_end: Some(datetime!(2023-03-07 13:15:00.0 +00:00:00)),
                        payload: EventPayload {
                            name: "Infa".to_string(),
                            description: None,
                        }
                    }
                )
            ]),
            entries: vec![
                // Entry {
                //     event_id: uuid!("374ae0ab-d473-4752-b77f-cae55c69245c"),
                //     starts_at: datetime!(2023-03-07 11:30 UTC),
                //     ends_at: datetime!(2023-03-07 13:15 UTC),
                //     recurrence_override: None,
                // },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-08 09:45 UTC),
                        datetime!(2023-03-08 10:30 UTC)
                    ),
                    recurrence_override: None,
                },
                Entry {
                    event_id: uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1"),
                    time_range: TimeRange::new(
                        datetime!(2023-03-09 09:45 UTC),
                        datetime!(2023-03-09 10:30 UTC)
                    ),
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

    let update_data = UpdateEvent { data };
    update_one_event(&pool, PKBPMJ_ID, update_data, event_id)
        .await
        .unwrap();

    assert_eq!(
        get_one_event(&pool, PKBPMJ_ID, event_id).await.unwrap(),
        Event {
            can_edit: true,
            is_owned: true,
            recurrence_rule: Some(RecurrenceRule {
                span: Some(EntriesSpan {
                    end: datetime!(2024-01-07 9:35:00.0 +00:00:00),
                    repetitions: 10,
                }),
                interval: 1,
                kind: RecurrenceRuleKind::Monthly { is_by_day: true },
            }),
            entries_start: datetime!(2023-03-07 08:00 +00:00:00),
            entries_end: Some(datetime!(2024-01-07 9:35:00.0 +00:00:00)),
            payload: EventPayload {
                name: "Polski".to_string(),
                description: Some("niespodzianka!!".to_string()),
            },
        }
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

    let update_data = UpdateEvent { data };

    assert!(update_one_event(
        &pool,
        MABI19_ID,
        update_data,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b")
    )
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
    assert!(delete_one_event_permanently(
        &pool,
        ADIMAC_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn update_edit_privileges_test(pool: PgPool) {
    update_user_editing_privileges(
        &pool,
        PKBPMJ_ID,
        UpdateEditPrivilege {
            user_id: ADIMAC_ID,
            can_edit: true,
        },
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let mut query = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);
    assert_eq!(
        query
            .can_edit(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
            .await
            .unwrap(),
        true
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_update_privileges_without_ownership(pool: PgPool) {
    assert!(update_user_editing_privileges(
        &pool,
        ADIMAC_ID,
        UpdateEditPrivilege {
            user_id: PKBPMJ_ID,
            can_edit: false,
        },
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err());
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_self_update_privileges(pool: PgPool) {
    assert!(update_user_editing_privileges(
        &pool,
        PKBPMJ_ID,
        UpdateEditPrivilege {
            user_id: PKBPMJ_ID,
            can_edit: false,
        },
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err());
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn update_event_owner_test(pool: PgPool) {
    set_event_ownership(
        &pool,
        PKBPMJ_ID,
        ADIMAC_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let mut q1 = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);

    assert_eq!(
        q1.is_owner(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
            .await
            .unwrap(),
        true
    );

    let mut q2 = PgQuery::new(EventQuery::new(PKBPMJ_ID), &mut conn);

    assert_eq!(
        q2.is_owner(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
            .await
            .unwrap(),
        false
    );

    assert_eq!(
        q2.can_edit(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
            .await
            .unwrap(),
        true
    );
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_update_owner_without_ownership(pool: PgPool) {
    assert!(set_event_ownership(
        &pool,
        ADIMAC_ID,
        PKBPMJ_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_self_update_ownership(pool: PgPool) {
    assert!(set_event_ownership(
        &pool,
        PKBPMJ_ID,
        PKBPMJ_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn disconnect_user_from_event_test(pool: PgPool) {
    delete_user_event(
        &pool,
        ADIMAC_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .unwrap();

    assert!(query!(
        r#"
            SELECT FROM user_events
            WHERE user_id = $1
            AND event_id = $2
        "#,
        ADIMAC_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_none())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn cannot_disconnect_owner_from_event(pool: PgPool) {
    assert!(delete_user_event(
        &pool,
        PKBPMJ_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .await
    .is_err())
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn disconnect_owner_from_event_test(pool: PgPool) {
    delete_owner_from_event(
        &pool,
        PKBPMJ_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
        ADIMAC_ID,
    )
    .await
    .unwrap();

    let user_exists = query!(
        r#"
            SELECT FROM user_events
            WHERE user_id = $1
            AND event_id = $2
        "#,
        PKBPMJ_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    let mut conn = pool.acquire().await.unwrap();
    let mut q = PgQuery::new(EventQuery::new(ADIMAC_ID), &mut conn);

    assert!(!user_exists);
    assert_eq!(
        q.is_owner(uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"))
            .await
            .unwrap(),
        true
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events"))]
async fn does_not_disconnect_user_as_owner(pool: PgPool) {
    assert!(delete_owner_from_event(
        &pool,
        ADIMAC_ID,
        uuid!("6d185de5-ddec-462a-aeea-7628f03d417b"),
        PKBPMJ_ID,
    )
    .await
    .is_err())
}
