use bimetable::modules::database::PgQuery;
use bimetable::routes::events::models::{
    Entry, EventFilter, Events, Override, OverrideEvent, OverrideEventData,
};
use bimetable::utils::events::exe::{create_one_event_override, get_many_events};
use bimetable::utils::events::models::TimeRange;
use bimetable::utils::events::{EventQuery, QOverride};
use sqlx::PgPool;
use time::macros::datetime;
use time::Duration;
use tracing_test::traced_test;
use uuid::{uuid, Uuid};

const ADIMAC_ID: Uuid = uuid!("910e81a9-56df-4c24-965a-13eff739f469");
const PKBPMJ_ID: Uuid = uuid!("29e40c2a-7595-42d3-98e8-9fe93ce99972");
const MABI19_ID: Uuid = uuid!("32190025-7c15-4adb-82fd-9acc3dc8e7b6");
const HUBERT_ID: Uuid = uuid!("a9c5900e-a445-4888-8612-4a5c8cadbd9e");
const INFORMATYKA_ID: Uuid = uuid!("d63a1036-e59d-4b7c-a009-9b90a0e703d1");
const FIZYKA_ID: Uuid = uuid!("fd1dcdf7-de06-4aad-ba6e-f2097217a5b1");
const MATEMATYKA_ID: Uuid = uuid!("6d185de5-ddec-462a-aeea-7628f03d417b");

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn create_override_test(pool: PgPool) {
    let body = OverrideEvent {
        override_starts_at: datetime!(2023-03-14 11:40 UTC),
        override_ends_at: datetime!(2023-03-15 13:15 UTC),
        data: OverrideEventData {
            name: None,
            description: Some("new desc".into()),
            starts_at: None,
            ends_at: None,
        },
    };
    create_one_event_override(&pool, HUBERT_ID, body, INFORMATYKA_ID)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let mut q = PgQuery::new(EventQuery::new(HUBERT_ID), &mut conn);
    let res = q.get_overrides(vec![INFORMATYKA_ID]).await.unwrap();
    assert_eq!(res.len(), 1)
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn does_not_create_override_with_wrong_range(pool: PgPool) {
    let body = OverrideEvent {
        override_starts_at: datetime!(2023-03-15 11:40 UTC),
        override_ends_at: datetime!(2023-03-14 13:15 UTC),
        data: OverrideEventData {
            name: None,
            description: Some("new desc".into()),
            starts_at: None,
            ends_at: None,
        },
    };
    assert!(
        create_one_event_override(&pool, HUBERT_ID, body, INFORMATYKA_ID)
            .await
            .is_err()
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn does_not_create_override_without_editing_privileges(pool: PgPool) {
    let body = OverrideEvent {
        override_starts_at: datetime!(2023-03-14 11:40 UTC),
        override_ends_at: datetime!(2023-03-15 13:15 UTC),
        data: OverrideEventData {
            name: None,
            description: Some("new desc".into()),
            starts_at: None,
            ends_at: None,
        },
    };

    assert!(
        create_one_event_override(&pool, MABI19_ID, body, INFORMATYKA_ID)
            .await
            .is_err()
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn get_entries_with_override_1(pool: PgPool) {
    let events = get_many_events(
        PKBPMJ_ID,
        TimeRange::new(
            datetime!(2023-03-13 0:00 UTC),
            datetime!(2023-03-26 23:59 UTC),
        ),
        EventFilter::Owned,
        &pool,
    )
    .await
    .unwrap();
    let res: Vec<Entry> = events
        .entries
        .into_iter()
        .filter(|entry| entry.event_id == FIZYKA_ID)
        .collect();

    assert_eq!(
        res,
        vec![
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-15 9:45 UTC),
                    end: datetime!(2023-03-15 10:30 UTC),
                },
                recurrence_override: Some(Override {
                    name: None,
                    description: Some("Blok fizyki".into()),
                    starts_at: Some(Duration::minutes(-55)),
                    ends_at: Some(Duration::minutes(50)),
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-16 9:45 UTC),
                    end: datetime!(2023-03-16 10:30 UTC),
                },
                recurrence_override: Some(Override {
                    name: None,
                    description: Some("Blok fizyki".into()),
                    starts_at: Some(Duration::minutes(-55)),
                    ends_at: Some(Duration::minutes(50)),
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-22 9:45 UTC),
                    end: datetime!(2023-03-22 10:30 UTC),
                },
                recurrence_override: None,
            },
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-23 9:45 UTC),
                    end: datetime!(2023-03-23 10:30 UTC),
                },
                recurrence_override: None,
            }
        ]
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn get_entries_with_override_2(pool: PgPool) {
    let events = get_many_events(
        PKBPMJ_ID,
        TimeRange::new(
            datetime!(2023-05-07 0:00 UTC),
            datetime!(2024-01-07 23:59 UTC),
        ),
        EventFilter::Owned,
        &pool,
    )
    .await
    .unwrap();
    let res: Vec<Entry> = events
        .entries
        .into_iter()
        .filter(|entry| entry.event_id == MATEMATYKA_ID)
        .collect();

    assert_eq!(
        res,
        vec![
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-05-07 8:00 UTC),
                    end: datetime!(2023-05-07 9:35 UTC),
                },
                recurrence_override: None,
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-06-07 8:00 UTC),
                    end: datetime!(2023-06-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Polski".into()),
                    description: None,
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-07-07 8:00 UTC),
                    end: datetime!(2023-07-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Polski".into()),
                    description: None,
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-08-07 8:00 UTC),
                    end: datetime!(2023-08-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Polski".into()),
                    description: None,
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-09-07 8:00 UTC),
                    end: datetime!(2023-09-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Polski".into()),
                    description: None,
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-10-07 8:00 UTC),
                    end: datetime!(2023-10-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Geografia".into()),
                    description: Some("Wyciagamy kartelinki".into()),
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:01 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-11-07 8:00 UTC),
                    end: datetime!(2023-11-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Geografia".into()),
                    description: Some("Wyciagamy kartelinki".into()),
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:01 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-12-07 8:00 UTC),
                    end: datetime!(2023-12-07 9:35 UTC),
                },
                recurrence_override: Some(Override {
                    name: Some("Geografia".into()),
                    description: Some("Wyciagamy kartelinki".into()),
                    starts_at: None,
                    ends_at: None,
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:01 UTC),
                }),
            },
            Entry {
                event_id: MATEMATYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2024-01-07 8:00 UTC),
                    end: datetime!(2024-01-07 9:35 UTC),
                },
                recurrence_override: None,
            },
        ]
    )
}

#[traced_test]
#[sqlx::test(fixtures("users", "events", "user_events", "overrides"))]
async fn override_with_range_overlapping_with_search_test(pool: PgPool) {
    let events = get_many_events(
        PKBPMJ_ID,
        TimeRange::new(
            datetime!(2023-03-13 0:00 UTC),
            datetime!(2023-03-16 8:51 UTC),
        ),
        EventFilter::Owned,
        &pool,
    )
    .await
    .unwrap();
    let res: Vec<Entry> = events
        .entries
        .into_iter()
        .filter(|entry| entry.event_id == FIZYKA_ID)
        .collect();

    assert_eq!(
        res,
        vec![
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-15 9:45 UTC),
                    end: datetime!(2023-03-15 10:30 UTC),
                },
                recurrence_override: Some(Override {
                    name: None,
                    description: Some("Blok fizyki".into()),
                    starts_at: Some(Duration::minutes(-55)),
                    ends_at: Some(Duration::minutes(50)),
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            },
            Entry {
                event_id: FIZYKA_ID,
                time_range: TimeRange {
                    start: datetime!(2023-03-16 9:45 UTC),
                    end: datetime!(2023-03-16 10:30 UTC),
                },
                recurrence_override: Some(Override {
                    name: None,
                    description: Some("Blok fizyki".into()),
                    starts_at: Some(Duration::minutes(-55)),
                    ends_at: Some(Duration::minutes(50)),
                    deleted_at: None,
                    created_at: datetime!(2023-04-01 8:00 UTC),
                }),
            }
        ]
    )
}
