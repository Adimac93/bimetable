use crate::modules::database::PgQuery;
use crate::routes::events::models::{
    CreateEvent, Entry, Event, EventPayload, EventPrivileges, Events, Override, OverrideEvent,
    UpdateEvent,
};
use crate::utils::events::models::{EventPart, EventRules, RecurrenceEndsAt, TimeRange};
use serde::Serialize;
use sqlx::pool::PoolConnection;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Json;
use sqlx::{query, query_as, Acquire, Connection, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

pub mod additions;
pub mod calculations;
pub mod count_to_until;
pub mod errors;
pub mod event_range;
pub mod models;

struct RangeOverride {
    ovr: Override,
    range: TimeRange,
}

struct QOverride {
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

struct QOwnedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<EventRules>>,
}

struct QSharedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<EventRules>>,
    can_edit: bool,
}

enum QEvent {
    Owned(QOwnedEvent),
    Shared(QSharedEvent),
}

pub struct EventQuery {}

impl<'c> PgQuery<'c, EventQuery> {
    pub async fn create_event(&mut self, user_id: Uuid, event: CreateEvent) -> sqlx::Result<Uuid> {
        let id = query!(
            r#"
                INSERT INTO events (owner_id, name, description, starts_at, ends_at, recurrence_rule)
                VALUES
                ($1, $2, $3, $4, $5, $6)
                RETURNING id
            "#,
            user_id,
            event.data.payload.name,
            event.data.payload.description,
            event.data.starts_at,
            event.data.ends_at,
            event.recurrence_rule as _
        )
            .fetch_one(&mut *self.conn)
            .await?
            .id;

        Ok(id)
    }

    pub async fn get_event(
        &mut self,
        user_id: Uuid,
        event_id: Uuid,
    ) -> sqlx::Result<Option<Event>> {
        let event = query_as!(
            QOwnedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>" 
                FROM events
                WHERE id = $1 AND deleted_at = null
            "#,
            event_id,
        )
            .fetch_optional(&mut *self.conn)
            .await?;

        if let Some(event) = event {
            let payload = EventPayload::new(event.name, event.description);
            let shared = query!(
                r#"
                SELECT * from user_events
                WHERE user_id = $1 AND event_id = $2
            "#,
                user_id,
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
            return Ok(Some(Event::new(EventPrivileges::Owned, payload)));
        }

        Ok(None)
    }

    async fn get_owned_events(
        &mut self,
        user_id: Uuid,
        search_starts_at: OffsetDateTime,
        search_ends_at: OffsetDateTime,
    ) -> sqlx::Result<Vec<QOwnedEvent>> {
        let events = query_as!(
            QOwnedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>" 
                FROM events
                WHERE owner_id = $1 AND starts_at > $2 AND ends_at < $3 AND deleted_at = null
                ORDER BY starts_at ASC
            "#,
            user_id,
            search_starts_at,
            search_ends_at
        )
            .fetch_all(&mut *self.conn)
            .await?;

        Ok(events)
    }

    pub async fn get_shared_events(
        &mut self,
        user_id: Uuid,
        search_starts_at: OffsetDateTime,
        search_ends_at: OffsetDateTime,
    ) -> sqlx::Result<Vec<QSharedEvent>> {
        let shared_events = query_as!(
            QSharedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: _", can_edit FROM user_events
                JOIN events ON user_events.event_id = events.id
                WHERE user_id = $1 AND starts_at > $2 AND ends_at < $3
                ORDER BY events.starts_at ASC
            "#,
            user_id,
            search_starts_at,
            search_ends_at
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(shared_events)
    }

    async fn get_overrides(&mut self, event_ids: Vec<Uuid>) -> sqlx::Result<Vec<QOverride>> {
        let overrides = query_as!(
            QOverride,
            r#"
                SELECT event_id, override_starts_at, override_ends_at, created_at, name, description, starts_at, ends_at, deleted_at
                FROM event_overrides
                WHERE event_id in ($1)
                ORDER BY override_starts_at ASC
            "#,
            event_ids as _
        )
            .fetch_all(&mut *self.conn)
            .await?;

        Ok(overrides)
    }

    pub async fn create_override(&mut self, user_id: Uuid, ovr: OverrideEvent) -> sqlx::Result<()> {
        query!(
            r#"
                INSERT INTO event_overrides (event_id, override_starts_at, override_ends_at, name, description, starts_at, ends_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            ovr.event_id,
            ovr.override_starts_at,
            ovr.override_ends_at,
            ovr.data.name,
            ovr.data.description,
            ovr.data.starts_at,
            ovr.data.ends_at
        ).execute(&mut *self.conn).await?;

        Ok(())
    }
    pub async fn update_event(&mut self, user_id: Uuid, event: UpdateEvent) -> sqlx::Result<()> {
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
            event.data.name,
            event.data.description,
            event.data.starts_at,
            event.data.ends_at,
            user_id,
            event.id,
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }

    pub async fn temp_delete(&mut self, user_id: Uuid, event_id: Uuid) -> sqlx::Result<()> {
        let now = OffsetDateTime::now_utc();
        query!(
            r#"
                UPDATE events
                SET
                deleted_at = $1
                WHERE owner_id = $2 AND id = $3
            "#,
            now,
            user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
    pub async fn perm_delete(&mut self, user_id: Uuid, event_id: Uuid) -> sqlx::Result<()> {
        query!(
            r#"
                DELETE FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}

fn merge_sorted_vectors_ascending(a: Events, b: Events) -> Events {
    let mut entries = vec![];

    let a_entries = a.entries;
    let b_entries = b.entries;

    let mut a_events = a.events;
    let b_events = b.events;
    a_events.extend(b_events);
    let events = a_events;

    let mut i = 0;
    let mut j = 0;

    while i < a_entries.len() && j < b_entries.len() {
        if a_entries[i].starts_at < b_entries[j].starts_at {
            entries.push(a_entries[i].clone());
            i += 1;
        } else {
            entries.push(b_entries[j].clone());
            j += 1;
        }
    }

    // result.extend(a.split_at(i).1.iter());
    // result.extend(b.split_at(j).1.iter());

    while i < a_entries.len() {
        entries.push(a_entries[i].clone());
        i += 1;
    }

    while j < b_entries.len() {
        entries.push(b_entries[j].clone());
        j += 1;
    }

    Events::new(events, entries)
}

pub async fn get_many_events(
    user_id: Uuid,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<Events> {
    let mut q = PgQuery::new(EventQuery {}, conn);
    let owned_events = q.get_owned_events(user_id, starts_at, ends_at).await?;
    let shared_events = q.get_shared_events(user_id, starts_at, ends_at).await?;

    let owned_events_overrides = q
        .get_overrides(owned_events.iter().map(|ev| ev.id).collect())
        .await?;

    let shared_events_overrides = q
        .get_overrides(shared_events.iter().map(|ev| ev.id).collect())
        .await?;

    let owned_events = gen_owned_events(owned_events_overrides, owned_events, starts_at, ends_at);
    let shared_events =
        gen_shared_events(shared_events_overrides, shared_events, starts_at, ends_at);

    let all_events = merge_sorted_vectors_ascending(owned_events, shared_events);
    Ok(all_events)
}
fn gen_owned_events(
    overrides: Vec<QOverride>,
    events: Vec<QOwnedEvent>,
    search_starts_at: OffsetDateTime,
    search_ends_at: OffsetDateTime,
) -> Events {
    let mut ovrs = group_overrides(overrides);
    let mut entries: Vec<Entry> = vec![];

    let events: HashMap<Uuid, Event> = events
        .into_iter()
        .map(|event| {
            if let Some(Json(rule)) = event.recurrence_rule {
                let mut entry_ranges = rule
                    .get_event_range(
                        &EventPart {
                            starts_at: search_starts_at,
                            length: Some(RecurrenceEndsAt::Until(search_ends_at)),
                        },
                        &TimeRange::new(event.starts_at, event.ends_at),
                    )
                    .unwrap();
                entry_ranges.reverse();

                gen_entries(event.id, &mut entry_ranges, &mut ovrs, &mut entries);
            }

            (
                event.id,
                Event::new(
                    EventPrivileges::Owned,
                    EventPayload::new(event.name, event.description),
                ),
            )
        })
        .collect();

    Events::new(events, entries)
}

fn gen_shared_events(
    overrides: Vec<QOverride>,
    events: Vec<QSharedEvent>,
    search_starts_at: OffsetDateTime,
    search_ends_at: OffsetDateTime,
) -> Events {
    let mut ovrs = group_overrides(overrides);
    let mut entries: Vec<Entry> = vec![];

    let events: HashMap<Uuid, Event> = events
        .into_iter()
        .map(|event| {
            if let Some(Json(rule)) = event.recurrence_rule {
                let mut entry_ranges = rule
                    .get_event_range(
                        &EventPart {
                            starts_at: search_starts_at,
                            length: Some(RecurrenceEndsAt::Until(search_ends_at)),
                        },
                        &TimeRange::new(event.starts_at, event.ends_at),
                    )
                    .unwrap();
                entry_ranges.reverse();

                gen_entries(event.id, &mut entry_ranges, &mut ovrs, &mut entries);
            }

            (
                event.id,
                Event::new(
                    EventPrivileges::Shared {
                        can_edit: event.can_edit,
                    },
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

fn gen_entries(
    event_id: Uuid,
    entry_ranges: &mut Vec<TimeRange>,
    overrides: &mut HashMap<Uuid, Vec<(TimeRange, Override)>>,
    entries: &mut Vec<Entry>,
) {
    if let Some(range_overrides) = overrides.remove(&event_id) {
        for (ovr_range, ovr_payload) in range_overrides {
            while let Some(entry_range) = entry_ranges.last() {
                if entry_range.is_contained(&ovr_range) {
                    entries.push(Entry {
                        event_id,
                        starts_at: ovr_range.start,
                        ends_at: ovr_range.end,
                        recurrence_override: Some(ovr_payload.clone()),
                    });
                    entry_ranges.pop();
                } else if entry_range.is_before(&ovr_range) {
                    entries.push(Entry {
                        event_id,
                        starts_at: entry_range.start,
                        ends_at: entry_range.end,
                        recurrence_override: None,
                    });
                    entry_ranges.pop();
                } else {
                    break;
                }
            }
        }
    } else {
        entries.extend(entry_ranges.iter().map(|range| Entry {
            event_id,
            starts_at: range.start,
            ends_at: range.end,
            recurrence_override: None,
        }));
    }
}
