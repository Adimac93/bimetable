use crate::modules::database::{PgPool, PgQuery};
use crate::routes::events::models::{
    CreateEvent, Entry, Event, EventFilter, EventPayload, EventPrivileges, Events,
    OptionalEventData, Override, OverrideEvent, UpdateEvent,
};
use crate::utils::events::models::{RecurrenceRule, TimeRange};
use sqlx::types::{time::OffsetDateTime, Json};
use sqlx::{query, query_as, Acquire};
use std::collections::HashMap;
use tracing::log::trace;
use uuid::Uuid;

use self::errors::EventError;

pub mod additions;
pub mod calculations;
pub mod count_to_until;
pub mod errors;
pub mod event_range;
pub mod exe;
pub mod models;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct QOwnedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<Json<RecurrenceRule>>,
}

#[derive(Debug)]
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

#[derive(Debug)]
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
        let event_id = query!(
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

        trace!("Created event {event_id}");
        Ok(event_id)
    }

    pub async fn get_event(&mut self, event_id: Uuid) -> Result<Option<Event>, EventError> {
        let event = query!(
            r#"
                SELECT id, owner_id, name, description, starts_at, ends_at, deleted_at, recurrence_rule
                FROM events
                WHERE id = $1 AND deleted_at IS NULL
            "#,
            event_id,
        )
            .fetch_optional(&mut *self.conn)
            .await?;

        if let Some(event) = event {
            let payload = EventPayload::new(event.name, event.description);
            let rec_rule = event
                .recurrence_rule
                .and_then(|x| serde_json::from_value(x).ok()?);

            if event.owner_id == self.payload.user_id {
                trace!("Got owned event {}", event.id);

                return Ok(Some(Event::new(EventPrivileges::Owned, payload, rec_rule)));
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
                trace!("Got shared event {}", event.id);
                return Ok(Some(Event::new(
                    EventPrivileges::Shared {
                        can_edit: shared.can_edit,
                    },
                    payload,
                    rec_rule,
                )));
            }
        }
        trace!("There is no event with id {event_id}");
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

        trace!("Got owned event {event_id}");
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
                WHERE owner_id = $1 AND starts_at >= $2 AND ends_at < $3 AND deleted_at IS NULL
                ORDER BY starts_at ASC
            "#,
            self.payload.user_id,
            search_range.start,
            search_range.end
        )
            .fetch_all(&mut *self.conn)
            .await?;

        if !events.is_empty() {
            trace!(
                "Got {} owned events in search range {search_range}",
                events.len()
            );
        } else {
            trace!("No owned events in search range {search_range}");
        }

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
                WHERE user_id = $1 AND starts_at >= $2 AND ends_at < $3 AND deleted_at IS NULL AND owner_id <> $1
                ORDER BY events.starts_at ASC
            "#,
            self.payload.user_id,
            search_range.start,
            search_range.end
        )
            .fetch_all(&mut *self.conn)
            .await?;

        if !shared_events.is_empty() {
            trace!("Got shared events in search range {search_range}");
        } else {
            trace!("No shared events in search range {search_range}");
        }

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

        if !overrides.is_empty() {
            trace!("Got events' overrides for {overrides:#?}");
        }

        Ok(overrides)
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

        trace!("Created event override for event {event_id}");

        Ok(())
    }
    pub async fn update_event(
        &mut self,
        event_id: Uuid,
        event: OptionalEventData,
    ) -> Result<(), EventError> {
        // only empty string will delete description because it is an optional parameter
        query!(
            r#"
                UPDATE events
                SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                starts_at = COALESCE($3, starts_at),
                ends_at = COALESCE($4, ends_at)
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

        trace!("Updated event {event_id}");

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

        trace!("Temporarily deleted event {event_id}");

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

        trace!("Permanently deleted event {event_id}");

        Ok(())
    }

    pub async fn is_owner(&mut self, event_id: Uuid) -> Result<bool, EventError> {
        let query_res = query!(
            r#"
                SELECT owner_id FROM events WHERE id = $1
            "#,
            event_id
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .ok_or(EventError::NotFound)?;

        let res = query_res.owner_id == self.payload.user_id;

        if res {
            trace!("User {} owns the event {event_id}", self.payload.user_id)
        } else {
            trace!(
                "User {} does not own the event {event_id}",
                self.payload.user_id
            )
        }

        Ok(res)
    }

    pub async fn can_edit(&mut self, event_id: Uuid) -> Result<bool, EventError> {
        let res = query!(
            r#"
                SELECT can_edit
                FROM user_events
                WHERE user_id = $1 AND event_id = $2
            "#,
            self.payload.user_id,
            event_id
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .ok_or(EventError::NotFound)?;

        if res.can_edit {
            trace!(
                "User {} can edit the event {event_id}",
                self.payload.user_id
            )
        } else {
            trace!(
                "User {} can not edit the event {event_id}",
                self.payload.user_id
            )
        }

        Ok(res.can_edit)
    }

    pub async fn update_edit_privileges(
        &mut self,
        target_user_id: Uuid,
        event_id: Uuid,
        can_edit: bool,
    ) -> Result<(), EventError> {
        query!(
            r#"
                UPDATE user_events
                SET can_edit = $1
                WHERE user_id = $2
                AND event_id = $3
            "#,
            can_edit,
            target_user_id,
            event_id,
        )
        .execute(&mut *self.conn)
        .await?;

        trace!("Updated editing privileges for user {target_user_id} and event {event_id} to {can_edit}");

        Ok(())
    }

    pub async fn update_event_owner(
        &mut self,
        owner_id: Uuid,
        event_id: Uuid,
    ) -> Result<(), EventError> {
        query!(
            r#"
                UPDATE events
                SET owner_id = $1
                WHERE id = $2
            "#,
            owner_id,
            event_id,
        )
        .execute(&mut *self.conn)
        .await?;

        trace!("Set owner of the event {event_id} to {owner_id}");

        Ok(())
    }

    pub async fn delete_user_event(
        &mut self,
        user_id: Uuid,
        event_id: Uuid,
    ) -> Result<(), EventError> {
        query!(
            r#"
                DELETE FROM user_events
                WHERE user_id = $1
                AND event_id = $2
            "#,
            user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        trace!("Removed user {user_id} from event {event_id}");

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

                let new_entries = get_entries(event.id, &mut entry_ranges, &mut ovrs);
                entries.extend(new_entries);

                return (
                    event.id,
                    Event::new(
                        event.privileges,
                        EventPayload::new(event.name, event.description),
                        Some(rule),
                    ),
                );
            }

            (
                event.id,
                Event::new(
                    event.privileges,
                    EventPayload::new(event.name, event.description),
                    None,
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
    if !ovrs.is_empty() {
        trace!("Grouped overrides {ovrs:#?}");
    }

    ovrs
}

fn get_entries(
    event_id: Uuid,
    entry_ranges: &mut Vec<TimeRange>,
    overrides: &mut HashMap<Uuid, Vec<(TimeRange, Override)>>,
) -> impl IntoIterator<Item = Entry> {
    if let Some(range_overrides) = overrides.remove(&event_id) {
        let event_entries = apply_event_overrides(event_id, entry_ranges, range_overrides);
        trace!(
            "Got {} entries with overrides for event {event_id}",
            event_entries.len()
        );
        return event_entries;
    }

    trace!("Got {} entries for event {event_id}", entry_ranges.len());
    entry_ranges
        .iter_mut()
        .map(|range| Entry::new(event_id, range.start, range.end, None))
        .collect()
}

fn apply_event_overrides(
    event_id: Uuid,
    entry_ranges: &mut Vec<TimeRange>,
    overrides: Vec<(TimeRange, Override)>,
) -> Vec<Entry> {
    let mut entries: Vec<Entry> = vec![];
    for (ovr_range, ovr_payload) in overrides {
        while let Some(entry_range) = entry_ranges.last() {
            if entry_range.is_contained(entry_range) {
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

enum RecEndsAt {
    Until(OffsetDateTime),
    Count(u64),
}

enum RecEndsType {
    Until,
    Count,
}

struct RecEndsAtt {
    kind: RecEndsType,
    count: u64,
    until: OffsetDateTime,
}
