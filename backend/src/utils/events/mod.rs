use crate::modules::database::{PgPool, PgQuery};
use crate::routes::events::models::{
    CreateEvent, Entry, Event, EventFilter, EventPayload, EventPrivileges, Events,
    OptionalEventData, Override, OverrideEvent,
};
use crate::utils::events::models::{EventRules, TimeRange};
use sqlx::types::{time::OffsetDateTime, Json};
use sqlx::{query, query_as};
use std::collections::HashMap;
use uuid::Uuid;

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
    recurrence_rule: Option<Json<EventRules>>,
}

pub struct QSharedEvent {
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
        let event = query!(
            r#"
                SELECT id, owner_id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>" 
                FROM events
                WHERE id = $1 AND deleted_at = null
            "#,
            event_id,
        )
            .fetch_optional(&mut *self.conn)
            .await?;

        if let Some(event) = event {
            let payload = EventPayload::new(event.name, event.description);
            if event.owner_id == user_id {
                return Ok(Some(Event::new(EventPrivileges::Owned, payload)));
            }

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
        }

        Ok(None)
    }

    pub async fn get_owned_event(
        &mut self,
        user_id: Uuid,
        event_id: Uuid,
    ) -> sqlx::Result<QOwnedEvent> {
        let event = query_as!(
            QOwnedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>" 
                FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            user_id,
            event_id
        )
            .fetch_one(&mut *self.conn)
            .await?;

        Ok(event)
    }

    pub async fn get_owned_events(
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

    pub async fn is_owned_event(&mut self, user_id: Uuid, event_id: Uuid) -> sqlx::Result<bool> {
        let res = query!(
            r#"
                SELECT * FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            user_id,
            event_id
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .is_some();

        Ok(res)
    }
    pub async fn create_override(
        &mut self,
        user_id: Uuid,
        event_id: Uuid,
        ovr: OverrideEvent,
    ) -> sqlx::Result<()> {
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
        user_id: Uuid,
        event_id: Uuid,
        event: OptionalEventData,
    ) -> sqlx::Result<()> {
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
            user_id,
            event_id,
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

async fn get_owned(
    user_id: Uuid,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    query: &mut PgQuery<'_, EventQuery>,
) -> sqlx::Result<Events> {
    let owned_events = query.get_owned_events(user_id, starts_at, ends_at).await?;
    let owned_events_overrides = query
        .get_overrides(owned_events.iter().map(|ev| ev.id).collect())
        .await?;

    Ok(gen_owned_events(
        owned_events_overrides,
        owned_events,
        starts_at,
        ends_at,
    ))
}

async fn get_shared(
    user_id: Uuid,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    query: &mut PgQuery<'_, EventQuery>,
) -> sqlx::Result<Events> {
    let shared_events = query.get_shared_events(user_id, starts_at, ends_at).await?;
    let shared_events_overrides = query
        .get_overrides(shared_events.iter().map(|ev| ev.id).collect())
        .await?;

    Ok(gen_shared_events(
        shared_events_overrides,
        shared_events,
        starts_at,
        ends_at,
    ))
}

pub async fn get_many_events(
    user_id: Uuid,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    filter: EventFilter,
    pool: PgPool,
) -> sqlx::Result<Events> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    return match filter {
        EventFilter::All => {
            let owned_events = get_owned(user_id, starts_at, ends_at, &mut q).await?;
            let shared_events = get_shared(user_id, starts_at, ends_at, &mut q).await?;

            Ok(owned_events.merge(shared_events))
        }
        EventFilter::Owned => Ok(get_owned(user_id, starts_at, ends_at, &mut q).await?),
        EventFilter::Shared => Ok(get_shared(user_id, starts_at, ends_at, &mut q).await?),
    };
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
                        TimeRange::new(search_starts_at, search_ends_at),
                        TimeRange::new(event.starts_at, event.ends_at),
                    )
                    .unwrap();
                entry_ranges.reverse();

                add_entries(event.id, &mut entry_ranges, &mut ovrs, &mut entries);
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
                        TimeRange::new(search_starts_at, search_ends_at),
                        TimeRange::new(event.starts_at, event.ends_at),
                    )
                    .unwrap();
                entry_ranges.reverse();

                add_entries(event.id, &mut entry_ranges, &mut ovrs, &mut entries);
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
        entries.extend(entry_ranges.iter().map(|range| Entry {
            event_id,
            starts_at: range.start,
            ends_at: range.end,
            recurrence_override: None,
        }));
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
    entries
}

// fn gen_entries_for_many_events()
