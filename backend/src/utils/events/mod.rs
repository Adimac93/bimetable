use anyhow::anyhow;
use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use sqlx::types::{time::OffsetDateTime, Json};
use sqlx::{query, query_as, Acquire};
use time::Duration;
use tracing::debug;
use tracing::log::trace;
use uuid::Uuid;

use crate::modules::database::{PgPool, PgQuery};
use crate::routes::events::models::{
    CreateEvent, Entry, Event, EventFilter, EventPayload, EventPrivileges, Events,
    OptionalEventData, Override, OverrideEvent, UpdateEvent,
};
use crate::utils::events::models::{EntriesSpan, RecurrenceRule, RecurrenceRuleKind, TimeRange};
use crate::utils::events::near_entriies::{next_entry, prev_entry};

use self::errors::EventError;
use self::models::UserEvent;

pub mod additions;
pub mod count_to_until;
pub mod errors;
pub mod event_range;
pub mod exe;
pub mod models;
pub mod near_entriies;
pub mod until_to_count;

#[derive(Debug)]
pub struct QOverride {
    event_id: Uuid,
    override_starts_at: OffsetDateTime,
    override_ends_at: OffsetDateTime,
    created_at: OffsetDateTime,
    name: Option<String>,
    description: Option<String>,
    starts_at: Option<Duration>,
    ends_at: Option<Duration>,
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
    recurrence_rule: Option<RecurrenceRule>,
}

#[derive(Debug)]
pub struct QSharedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<RecurrenceRule>,
    can_edit: bool,
}

#[derive(Debug)]
pub struct QEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    time_range: TimeRange,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<RecurrenceRule>,
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
        let rule = if let Some(rule) = event.recurrence_rule {
            let rule = rule.to_compute(&TimeRange::new(event.data.starts_at, event.data.ends_at));
            Some(rule)
        } else {
            None
        };

        let event_id = query!(
            r#"
                INSERT INTO events (owner_id, name, description, starts_at, ends_at)
                VALUES
                ($1, $2, $3, $4, $5)
                RETURNING id
            "#,
            self.payload.user_id,
            event.data.payload.name,
            event.data.payload.description,
            event.data.starts_at,
            event.data.ends_at,
        )
        .fetch_one(&mut *self.conn)
        .await?
        .id;

        if let Some(recurrence) = rule {
            let (until, count) = (
                recurrence.span.map(|x| x.end),
                recurrence.span.map(|x| x.repetitions as i32),
            );
            let interval = recurrence.interval as i32;
            query!(
                r#"
                INSERT INTO recurrence_rules (event_id, recurrence, until, count, interval)
                VALUES
                ($1, $2, $3, $4, $5)
            "#,
                event_id,
                sqlx::types::Json(recurrence) as _,
                until,
                count,
                interval,
            )
            .execute(&mut *self.conn)
            .await?;
        }

        trace!("Created event {event_id}");
        Ok(event_id)
    }

    pub async fn create_user_event(&mut self, user_event: UserEvent) -> Result<(), EventError> {
        query!(
            r#"
                INSERT INTO user_events (user_id, event_id, can_edit)
                VALUES
                ($1, $2, $3)
            "#,
            self.payload.user_id,
            user_event.event_id,
            user_event.can_edit,
        )
        .execute(&mut *self.conn)
        .await?;

        trace!(
            "Created user event with user_id {} and event_id {}",
            self.payload.user_id,
            user_event.can_edit
        );
        Ok(())
    }

    pub async fn get_event(&mut self, event_id: Uuid) -> Result<Option<Event>, EventError> {
        let event = query!(
            r#"
                SELECT id, owner_id, name, description, starts_at, COALESCE(until, ends_at) AS entries_end, deleted_at, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", until, count, interval AS "interval: Option<i32>"
                FROM events
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE id = $1 AND deleted_at IS NULL
            "#,
            event_id,
        )
        .fetch_optional(&mut *self.conn)
        .await?;

        if let Some(event) = event {
            let payload = EventPayload::new(event.name, event.description);

            let rec_rule = RecurrenceRule::from_db_data(
                event.recurrence,
                event.until,
                event.count,
                event.interval,
            );

            if event.owner_id == self.payload.user_id {
                trace!("Got owned event {}", event.id);

                return Ok(Some(Event::new(
                    EventPrivileges::Owned,
                    payload,
                    rec_rule,
                    event.starts_at,
                    event.entries_end,
                )));
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
                    event.starts_at,
                    event.entries_end,
                )));
            }
        }
        trace!("There is no event with id {event_id}");
        Ok(None)
    }

    // FIXME
    pub async fn get_owned_event(&mut self, event_id: Uuid) -> Result<QOwnedEvent, EventError> {
        let event = query!(
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", until, count, interval AS "interval: Option<i32>"
                FROM events
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE owner_id = $1 AND id = $2
            "#,
            self.payload.user_id,
            event_id
        )
            .fetch_one(&mut *self.conn)
            .await?;

        trace!("Got owned event {event_id}");

        let res = QOwnedEvent {
            id: event.id,
            name: event.name,
            description: event.description,
            starts_at: event.starts_at,
            ends_at: event.ends_at,
            deleted_at: event.deleted_at,
            recurrence_rule: RecurrenceRule::from_db_data(
                event.recurrence,
                event.until,
                event.count,
                event.interval,
            ),
        };
        Ok(res)
    }

    // FIXME
    pub async fn get_owned_events(
        &mut self,
        search_range: TimeRange,
    ) -> Result<Vec<QEvent>, EventError> {
        let events = query!(
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", until, count, interval as "interval: Option<i32>"
                FROM events
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE owner_id = $1 AND starts_at < $2 AND (until >= $3 OR (recurrence IS NULL AND until IS NULL AND ends_at >= $3) OR (recurrence IS NOT NULL AND until IS NULL)) AND deleted_at IS NULL
                ORDER BY starts_at ASC
            "#,
            self.payload.user_id,
            search_range.end,
            search_range.start,
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
                recurrence_rule: RecurrenceRule::from_db_data(
                    event.recurrence,
                    event.until,
                    event.count,
                    event.interval,
                ),
                privileges: EventPrivileges::Owned,
            })
            .collect();

        Ok(events)
    }

    // FIXME
    pub async fn get_shared_events(
        &mut self,
        search_range: TimeRange,
    ) -> Result<Vec<QEvent>, EventError> {
        let shared_events = query!(
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", until, count, interval as "interval: Option<i32>", can_edit
                FROM user_events
                JOIN events ON user_events.event_id = events.id
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE user_id = $1 AND starts_at < $2 AND (until >= $3 OR (recurrence IS NULL AND until IS NULL AND ends_at >= $3) OR (recurrence IS NOT NULL AND until IS NULL)) AND deleted_at IS NULL AND owner_id <> $1
                ORDER BY events.starts_at ASC
            "#,
            self.payload.user_id,
            search_range.end,
            search_range.start,
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
                recurrence_rule: RecurrenceRule::from_db_data(
                    event.recurrence,
                    event.until,
                    event.count,
                    event.interval,
                ),
                privileges: EventPrivileges::Shared {
                    can_edit: event.can_edit,
                },
            })
            .collect();

        Ok(shared_events)
    }

    pub async fn get_overrides(
        &mut self,
        event_ids: Vec<Uuid>,
    ) -> Result<Vec<QOverride>, EventError> {
        let overrides = query!(
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

        let mut res = Vec::new();
        for ovr in overrides.into_iter() {
            let starts_at = match ovr.starts_at {
                Some(entry_offset) => Some(to_time_duration(entry_offset)?),
                None => None,
            };
            let ends_at = match ovr.ends_at {
                Some(entry_offset) => Some(to_time_duration(entry_offset)?),
                None => None,
            };

            res.push(QOverride {
                event_id: ovr.event_id,
                override_starts_at: ovr.override_starts_at,
                override_ends_at: ovr.override_ends_at,
                created_at: ovr.created_at,
                name: ovr.name,
                description: ovr.description,
                starts_at,
                ends_at,
                deleted_at: None,
            });
        }

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
            ovr.data.starts_at as _,
            ovr.data.ends_at as _,
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
    )?)
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
    )?)
}

pub fn map_events(
    overrides: Vec<QOverride>,
    events: Vec<QEvent>,
    search_range: TimeRange,
) -> Result<Events, EventError> {
    let ovrs = group_overrides(overrides);
    let mut entries: Vec<Entry> = vec![];

    let events: HashMap<Uuid, Event> = events
        .into_iter()
        .map(|event| {
            let entries_end = if let Some(rule) = &event.recurrence_rule {
                let entry_ranges = rule.get_event_range(search_range, event.time_range)?;

                let mut new_entries: VecDeque<Entry> = get_entries(event.id, entry_ranges, &ovrs);

                if let Some(entry_range) = prev_entry(
                    search_range.start - Duration::nanoseconds(1),
                    event.time_range,
                    rule,
                )? {
                    if let Some(entry) = check_edge_entry(
                        event.id,
                        entry_range,
                        search_range,
                        ovrs.get(&event.id).unwrap_or(&vec![]),
                    ) {
                        new_entries.push_front(entry);
                    }
                };

                if let Some(entry_range) = next_entry(search_range.end, event.time_range, rule)? {
                    if let Some(entry) = check_edge_entry(
                        event.id,
                        entry_range,
                        search_range,
                        ovrs.get(&event.id).unwrap_or(&vec![]),
                    ) {
                        new_entries.push_back(entry);
                    }
                };

                entries.extend(new_entries);
                rule.span.map(|sp| sp.end)
            } else {
                Some(event.time_range.end)
            };

            return Ok((
                event.id,
                Event::new(
                    event.privileges,
                    EventPayload::new(event.name, event.description),
                    event.recurrence_rule,
                    event.time_range.start,
                    entries_end,
                ),
            ));
        })
        .collect::<Result<HashMap<Uuid, Event>, EventError>>()?;

    Ok(Events::new(events, entries))
}

fn group_overrides(overrides: Vec<QOverride>) -> HashMap<Uuid, Vec<(TimeRange, Override)>> {
    let mut ovrs: HashMap<Uuid, Vec<(TimeRange, Override)>> = HashMap::new();
    overrides.into_iter().for_each(|ovr| {
        let range = TimeRange::new(ovr.override_starts_at, ovr.override_ends_at);
        let entry_override = Override {
            name: ovr.name,
            description: ovr.description,
            starts_at: ovr.starts_at,
            ends_at: ovr.ends_at,
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

fn get_one_entry(
    event_id: Uuid,
    entry_range: TimeRange,
    overrides: &Vec<(TimeRange, Override)>,
) -> Entry {
    Entry {
        event_id,
        time_range: entry_range,
        recurrence_override: overrides
            .iter()
            .filter(|ovr| entry_range.is_contained(&ovr.0))
            .max_by_key(|ovr| ovr.1.created_at)
            .cloned()
            .map(|ovr| ovr.1),
    }
}

fn get_entries(
    event_id: Uuid,
    entry_ranges: Vec<TimeRange>,
    overrides: &HashMap<Uuid, Vec<(TimeRange, Override)>>,
) -> VecDeque<Entry> {
    if let Some(range_overrides) = overrides.get(&event_id) {
        let event_entries = apply_event_overrides(event_id, entry_ranges, range_overrides);
        trace!(
            "Got {} entries with overrides for event {event_id}",
            event_entries.len()
        );
        return event_entries.into();
    }

    trace!("Got {} entries for event {event_id}", entry_ranges.len());
    entry_ranges
        .into_iter()
        .map(|entry| Entry::new(event_id, TimeRange::new(entry.start, entry.end), None))
        .collect::<VecDeque<Entry>>()
}

fn apply_event_overrides(
    event_id: Uuid,
    entry_ranges: Vec<TimeRange>,
    overrides: &Vec<(TimeRange, Override)>,
) -> Vec<Entry> {
    let mut entries: Vec<Entry> = entry_ranges
        .into_iter()
        .map(|entry| Entry::new(event_id, TimeRange::new(entry.start, entry.end), None))
        .collect();
    for (ovr_range, ovr_payload) in overrides {
        let entry_start = entries.partition_point(|x| x.time_range.start < ovr_range.start);
        let entry_end = entries.partition_point(|x| x.time_range.end <= ovr_range.end);
        for i in entry_start..entry_end {
            entries[i].recurrence_override = Some(ovr_payload.clone());
        }
    }
    entries
}

fn to_time_duration(val: PgInterval) -> Result<Duration, EventError> {
    if val.days != 0 || val.months != 0 {
        Err(EventError::Unexpected(anyhow!(
            "Invalid interval data format in database type"
        )))
    } else {
        Ok(Duration::microseconds(val.microseconds))
    }
}

fn check_edge_entry(
    event_id: Uuid,
    entry_range: TimeRange,
    search_range: TimeRange,
    ovrs: &Vec<(TimeRange, Override)>,
) -> Option<Entry> {
    let entry = get_one_entry(event_id, entry_range, ovrs);
    entry.range_with_time_override().and_then(|modified_range| {
        if !entry_range.is_overlapping(&search_range)
            && modified_range.is_overlapping(&search_range)
        {
            Some(entry)
        } else {
            None
        }
    })
}
