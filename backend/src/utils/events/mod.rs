use crate::modules::database::{PgPool, PgQuery};
use crate::routes::events::models::{
    CreateEvent, Entry, Event, EventFilter, EventPayload, EventPrivileges, Events,
    OptionalEventData, Override, OverrideEvent,
};
use crate::utils::events::models::{RecurrenceRule, TimeRange};
use sqlx::types::{time::OffsetDateTime, Json};
use sqlx::{query, query_as};
use std::collections::HashMap;
use uuid::Uuid;

use self::errors::EventError;

pub mod additions;
pub mod calculations;
pub mod count_to_until;
pub mod errors;
pub mod event_range;
pub mod models;

pub struct QOverride {
    event_id: Uuid,
    override_starts_at: OffsetDateTime,
    override_ends_at: OffsetDateTime,
    created_at: OffsetDateTime,
    name: Option<String>,
    description: Option<String>,
    starts_at: Option<OffsetDateTime>,
    ends_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
}

pub struct QOwnedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<RecurrenceRule>>,
}

pub struct QSharedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<RecurrenceRule>>,
    can_edit: bool,
}

pub struct QEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    time_range: TimeRange,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<RecurrenceRule>>,
    privileges: EventPrivileges,
}

pub struct EventQuery {
    user_id: Uuid,
}

impl EventQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl<'c> PgQuery<'c, EventQuery> {
    pub async fn create_event(&mut self, event: CreateEvent) -> Result<Uuid, EventError> {
        let id = query!(
            r#"
                INSERT INTO events (owner_id, name, description, starts_at, ends_at, recurrence_rule)
                VALUES
                ($1, $2, $3, $4, $5, $6)
                RETURNING id
            "#,
            self.payload.user_id,
            event.data.payload.name,
            event.data.payload.description,
            event.data.starts_at,
            event.data.ends_at,
            sqlx::types::Json(event.recurrence_rule) as _
        )
            .fetch_one(&mut *self.conn)
            .await?
            .id;

        Ok(id)
    }

    pub async fn get_event(&mut self, event_id: Uuid) -> Result<Option<Event>, EventError> {
        let event = query!(
            r#"
                SELECT id, owner_id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<RecurrenceRule>" 
                FROM events
                WHERE id = $1 AND deleted_at IS NULL
            "#,
            event_id,
        )
            .fetch_optional(&mut *self.conn)
            .await?;

        if let Some(event) = event {
            let payload = EventPayload::new(event.name, event.description);
            if event.owner_id == self.payload.user_id {
                return Ok(Some(Event::new(EventPrivileges::Owned, payload)));
            }

            let shared = query!(
                r#"
                SELECT * from user_events
                WHERE user_id = $1 AND event_id = $2
            "#,
                self.payload.user_id,
                event_id,
            )
            .fetch_optional(&mut *self.conn)
            .await?;

            if let Some(shared) = shared {
                return Ok(Some(Event::new(
                    EventPrivileges::Shared {
                        can_edit: shared.can_edit,
                    },
                    payload,
                )));
            }
        }

        Ok(None)
    }

    pub async fn get_owned_event(&mut self, event_id: Uuid) -> Result<QOwnedEvent, EventError> {
        let event = query_as!(
            QOwnedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<RecurrenceRule>" 
                FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            self.payload.user_id,
            event_id
        )
            .fetch_one(&mut *self.conn)
            .await?;

        Ok(event)
    }

    pub async fn get_owned_events(
        &mut self,
        search_range: TimeRange,
    ) -> Result<Vec<QEvent>, EventError> {
        let events = query!(
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<RecurrenceRule>" 
                FROM events
                WHERE owner_id = $1 AND starts_at > $2 AND ends_at < $3 AND deleted_at IS NULL
                ORDER BY starts_at ASC
            "#,
            self.payload.user_id,
            search_range.start,
            search_range.end
        )
            .fetch_all(&mut *self.conn)
            .await?;

        let events = events
            .into_iter()
            .map(|event| QEvent {
                id: event.id,
                name: event.name,
                description: event.description,
                time_range: TimeRange::new(event.starts_at, event.ends_at),
                deleted_at: event.deleted_at,
                recurrence_rule: event.recurrence_rule,
                privileges: EventPrivileges::Owned,
            })
            .collect();

        Ok(events)
    }

    pub async fn get_shared_events(
        &mut self,
        search_range: TimeRange,
    ) -> Result<Vec<QEvent>, EventError> {
        let shared_events = query!(
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<RecurrenceRule>", can_edit FROM user_events
                JOIN events ON user_events.event_id = events.id
                WHERE user_id = $1 AND starts_at > $2 AND ends_at < $3
                ORDER BY events.starts_at ASC
            "#,
            self.payload.user_id,
            search_range.start,
            search_range.end
        )
            .fetch_all(&mut *self.conn)
            .await?;

        let shared_events = shared_events
            .into_iter()
            .map(|event| QEvent {
                id: event.id,
                name: event.name,
                description: event.description,
                time_range: TimeRange::new(event.starts_at, event.ends_at),
                deleted_at: event.deleted_at,
                recurrence_rule: event.recurrence_rule,
                privileges: EventPrivileges::Shared {
                    can_edit: event.can_edit,
                },
            })
            .collect();

        Ok(shared_events)
    }

    async fn get_overrides(&mut self, event_ids: Vec<Uuid>) -> Result<Vec<QOverride>, EventError> {
        let overrides = query_as!(
            QOverride,
            r#"
                SELECT event_id, override_starts_at, override_ends_at, created_at, name, description, starts_at, ends_at, deleted_at
                FROM event_overrides
                WHERE event_id = any($1)
                ORDER BY override_starts_at ASC
            "#,
            event_ids as _
        )
            .fetch_all(&mut *self.conn)
            .await?;

        Ok(overrides)
    }

    pub async fn is_owned_event(&mut self, event_id: Uuid) -> Result<bool, EventError> {
        let res = query!(
            r#"
                SELECT * FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            self.payload.user_id,
            event_id
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .is_some();

        Ok(res)
    }
    pub async fn create_override(
        &mut self,
        event_id: Uuid,
        ovr: OverrideEvent,
    ) -> Result<(), EventError> {
        query!(
            r#"
                INSERT INTO event_overrides (event_id, override_starts_at, override_ends_at, name, description, starts_at, ends_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            event_id,
            ovr.override_starts_at,
            ovr.override_ends_at,
            ovr.data.name,
            ovr.data.description,
            ovr.data.starts_at,
            ovr.data.ends_at
        ).execute(&mut *self.conn).await?;

        Ok(())
    }
    pub async fn update_event(
        &mut self,
        event_id: Uuid,
        event: OptionalEventData,
    ) -> Result<(), EventError> {
        query!(
            r#"
                UPDATE events
                SET
                name = $1,
                description = $2,
                starts_at = $3,
                ends_at = $4
                WHERE owner_id = $5 AND id = $6
            "#,
            event.name,
            event.description,
            event.starts_at,
            event.ends_at,
            self.payload.user_id,
            event_id,
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }

    pub async fn temp_delete(&mut self, event_id: Uuid) -> Result<(), EventError> {
        let now = OffsetDateTime::now_utc();
        query!(
            r#"
                UPDATE events
                SET
                deleted_at = $1
                WHERE owner_id = $2 AND id = $3
            "#,
            now,
            self.payload.user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
    pub async fn perm_delete(&mut self, event_id: Uuid) -> Result<(), EventError> {
        query!(
            r#"
                DELETE FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            self.payload.user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}

async fn get_owned(
    search_range: TimeRange,
    query: &mut PgQuery<'_, EventQuery>,
) -> Result<Events, EventError> {
    let owned_events = query.get_owned_events(search_range).await?;
    let owned_events_overrides = query
        .get_overrides(owned_events.iter().map(|ev| ev.id).collect())
        .await?;

    Ok(map_events(
        owned_events_overrides,
        owned_events,
        search_range,
    ))
}

async fn get_shared(
    search_range: TimeRange,
    query: &mut PgQuery<'_, EventQuery>,
) -> Result<Events, EventError> {
    let shared_events = query.get_shared_events(search_range).await?;
    let shared_events_overrides = query
        .get_overrides(shared_events.iter().map(|ev| ev.id).collect())
        .await?;

    Ok(map_events(
        shared_events_overrides,
        shared_events,
        search_range,
    ))
}

pub async fn get_many_events(
    user_id: Uuid,
    search_range: TimeRange,
    filter: EventFilter,
    pool: PgPool,
) -> Result<Events, EventError> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery { user_id }, &mut *conn);
    return match filter {
        EventFilter::All => {
            let owned_events = get_owned(search_range, &mut q).await?;
            let shared_events = get_shared(search_range, &mut q).await?;

            Ok(owned_events.merge(shared_events))
        }
        EventFilter::Owned => Ok(get_owned(search_range, &mut q).await?),
        EventFilter::Shared => Ok(get_shared(search_range, &mut q).await?),
    };
}

fn map_events(overrides: Vec<QOverride>, events: Vec<QEvent>, search_range: TimeRange) -> Events {
    let mut ovrs = group_overrides(overrides);
    let mut entries: Vec<Entry> = vec![];

    let events: HashMap<Uuid, Event> = events
        .into_iter()
        .map(|event| {
            if let Some(Json(rule)) = event.recurrence_rule {
                let mut entry_ranges = rule
                    .get_event_range(search_range, event.time_range)
                    .unwrap();

                add_entries(event.id, &mut entry_ranges, &mut ovrs, &mut entries);
            }

            (
                event.id,
                Event::new(
                    event.privileges,
                    EventPayload::new(event.name, event.description),
                ),
            )
        })
        .collect();

    Events::new(events, entries)
}

fn group_overrides(overrides: Vec<QOverride>) -> HashMap<Uuid, Vec<(TimeRange, Override)>> {
    let mut ovrs: HashMap<Uuid, Vec<(TimeRange, Override)>> = HashMap::new();
    overrides.into_iter().for_each(|ovr| {
        let range = TimeRange::new(ovr.override_starts_at, ovr.override_ends_at);
        let entry_override = Override {
            name: ovr.name,
            description: ovr.description,
            deleted_at: ovr.deleted_at,
            created_at: ovr.created_at,
        };

        ovrs.entry(ovr.event_id)
            .and_modify(|ranges| ranges.push((range, entry_override.clone())))
            .or_insert(vec![(range, entry_override)]);
    });
    ovrs
}

fn add_entries(
    event_id: Uuid,
    entry_ranges: &mut Vec<TimeRange>,
    overrides: &mut HashMap<Uuid, Vec<(TimeRange, Override)>>,
    entries: &mut Vec<Entry>,
) {
    if let Some(range_overrides) = overrides.remove(&event_id) {
        let event_entries = get_entries_for_event(event_id, entry_ranges, range_overrides);
        entries.extend(event_entries);
    } else {
        entries.extend(
            entry_ranges
                .iter()
                .map(|range| Entry::new(event_id, range.start, range.end, None)),
        );
    }
}

fn get_entries_for_event(
    event_id: Uuid,
    entry_ranges: &mut Vec<TimeRange>,
    overrides: Vec<(TimeRange, Override)>,
) -> Vec<Entry> {
    let mut entries: Vec<Entry> = vec![];
    for (ovr_range, ovr_payload) in overrides {
        while let Some(entry_range) = entry_ranges.last() {
            if entry_range.is_contained(&entry_range) {
                entries.push(Entry::new(
                    event_id,
                    ovr_range.start,
                    ovr_range.end,
                    Some(ovr_payload.clone()),
                ));
                entry_ranges.pop();
            } else if entry_range.is_before(&ovr_range) {
                entries.push(Entry::new(
                    event_id,
                    entry_range.start,
                    entry_range.end,
                    None,
                ));
                entry_ranges.pop();
            } else {
                break;
            }
        }
    }
    entries
}
