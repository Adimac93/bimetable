use crate::utils::events::count_to_until::{
    count_to_until, daily_c_to_u, monthly_c_to_u_by_day, monthly_c_to_u_by_weekday, weekly_c_to_u,
    yearly_c_to_u_by_day, yearly_c_to_u_by_weekday,
};
use crate::utils::events::errors::EventError;
use crate::utils::events::event_range::{
    get_daily_events, get_monthly_events_by_day, get_weekly_events, get_yearly_events_by_weekday,
};
use crate::utils::events::models::{EntriesSpan, RecurrenceRule, RecurrenceRuleKind, TimeRange};
use crate::utils::events::until_to_count::{
    daily_u_to_c, monthly_u_to_c_by_day, monthly_u_to_c_by_weekday, until_to_count, weekly_u_to_c,
    yearly_u_to_c_by_day, yearly_u_to_c_by_weekday,
};
use crate::validation::ValidateContent;
use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid};
use std::collections::HashMap;
use time::macros::datetime;
use time::serde::iso8601;
use tokio::time::interval;
use tracing::trace;
use utoipa::{IntoParams, ToResponse, ToSchema};
use uuid::uuid;
use validator::{Validate, ValidationError};

// Core data models
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OptionalEventData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "iso8601::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "iso8601::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct EventPayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EventPayload {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OptionalEventPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventData {
    pub payload: EventPayload,
    #[serde(with = "iso8601")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub ends_at: OffsetDateTime,
}

// Queries
#[derive(Debug, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct GetEventsQuery {
    #[serde(with = "iso8601")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub ends_at: OffsetDateTime,
    pub filter: EventFilter,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum EventFilter {
    All,
    Owned,
    Shared,
}

// Send payloads
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEvent {
    pub data: EventData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_rule: Option<RecurrenceRuleSchema>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventResult {
    pub event_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEvent {
    pub data: OptionalEventData,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OverrideEvent {
    pub time_range: TimeRange,
    pub data: OptionalEventData,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteEvent {
    pub event_id: Uuid,
    pub is_permanent: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOverride {
    pub override_id: Uuid,
    pub is_permanent: bool,
}

// Receive payloads
#[derive(Debug, Serialize, ToResponse, ToSchema, PartialEq)]
pub struct Events {
    pub events: HashMap<Uuid, Event>,
    pub entries: Vec<Entry>,
}

impl Events {
    pub fn new(events: HashMap<Uuid, Event>, entries: Vec<Entry>) -> Self {
        Self { events, entries }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.events.extend(other.events);
        self.entries.extend(other.entries);
        self.entries.sort_by_key(|entry| entry.time_range.start);
        self
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RecurrenceRuleSchema {
    pub time_rules: TimeRules,
    pub kind: RecurrenceRuleKind,
}

impl RecurrenceRuleSchema {
    pub fn to_compute(self, event_time_range: &TimeRange) -> RecurrenceRule {
        if let Some(ends_at) = &self.time_rules.ends_at {
            let (count, until) = match ends_at {
                RecurrenceEndsAt::Until(until) => {
                    let count = self
                        .until_to_count(event_time_range.start, *until, event_time_range)
                        .unwrap();
                    (count, *until)
                }
                RecurrenceEndsAt::Count(count) => {
                    let until = self
                        .count_to_until(event_time_range.start, *count, event_time_range)
                        .unwrap();
                    (*count, until)
                }
            };

            return RecurrenceRule {
                span: Some(EntriesSpan {
                    end: until,
                    repetitions: count,
                }),
                interval: self.time_rules.interval,
                kind: self.kind,
            };
        }
        RecurrenceRule {
            span: None,
            interval: self.time_rules.interval,
            kind: self.kind,
        }
    }
    /// Returns the end of the nth occurrence of the event, starting from a specified point in time.
    ///
    /// The first event in the given time bound counts as the 0th event.
    ///
    /// Currently, the point in time the search starts in must be the same as the beggining of any event occurrence.
    ///
    /// ```rust
    /// use bimetable::utils::events::models::RecurrenceRuleKind;
    /// use bimetable::utils::events::models::RecurrenceRule;
    /// use bimetable::utils::events::models::TimeRange;
    /// use time::macros::datetime;
    /// use bimetable::routes::events::models::{RecurrenceEndsAt, RecurrenceRuleSchema, TimeRules};
    ///
    /// let event = TimeRange::new(
    ///     datetime!(2023-02-18 10:00 UTC),
    ///     datetime!(2023-02-18 12:15 UTC),
    /// );
    /// let rec_rules = RecurrenceRuleSchema {
    ///     kind: RecurrenceRuleKind::Daily,
    ///     time_rules: TimeRules {
    ///         ends_at: Some(RecurrenceEndsAt::Count(15)),
    ///         interval: 3,
    ///     },
    /// };
    ///
    /// assert_eq!(
    ///     rec_rules.count_to_until(datetime!(2023-02-21 10:00 UTC), 1, &event).unwrap(),
    ///     datetime!(2023-02-24 12:15 UTC)
    /// )
    /// ```
    pub fn count_to_until(
        &self,
        part_starts_at: OffsetDateTime,
        count: u32,
        event: &TimeRange,
    ) -> Result<OffsetDateTime, EventError> {
        self.time_rules.validate_content()?;
        count_to_until(
            count,
            self.time_rules.interval,
            part_starts_at,
            event,
            &self.kind,
        )
    }

    pub fn until_to_count(
        &self,
        part_starts_at: OffsetDateTime,
        until: OffsetDateTime,
        event: &TimeRange,
    ) -> Result<u32, EventError> {
        self.time_rules.validate_content()?;
        until_to_count(
            until,
            part_starts_at,
            self.time_rules.interval,
            event.duration(),
            &self.kind,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RecurrenceEndsAt {
    #[serde(with = "iso8601")]
    Until(OffsetDateTime),
    Count(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeRules {
    pub ends_at: Option<RecurrenceEndsAt>,
    pub interval: u32,
}

#[derive(Debug, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub payload: EventPayload,
    pub recurrence_rule: Option<RecurrenceRule>,
    #[serde(with = "iso8601")]
    pub entries_start: OffsetDateTime,
    pub is_owned: bool,
    pub can_edit: bool,
}

#[derive(Debug)]
pub enum EventPrivileges {
    Owned,
    Shared { can_edit: bool },
}

impl Event {
    pub fn new(
        privileges: EventPrivileges,
        payload: EventPayload,
        recurrence_rule: Option<RecurrenceRule>,
        entries_start: OffsetDateTime,
    ) -> Self {
        match privileges {
            EventPrivileges::Owned => Self {
                payload,
                recurrence_rule,
                entries_start,
                is_owned: true,
                can_edit: true,
            },
            EventPrivileges::Shared { can_edit } => Self {
                payload,
                recurrence_rule,
                entries_start,
                is_owned: false,
                can_edit,
            },
        }
    }
}

#[derive(Debug, Serialize, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub event_id: Uuid,
    pub time_range: TimeRange,
    #[serde(rename(serialize = "override"))]
    #[schema(rename = "override")]
    pub recurrence_override: Option<Override>,
}

impl Entry {
    pub fn new(
        event_id: Uuid,
        time_range: TimeRange,
        recurrence_override: Option<Override>,
    ) -> Self {
        Self {
            event_id,
            time_range,
            recurrence_override,
        }
    }
}

#[derive(Debug, Serialize, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "iso8601::option", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEditPrivilege {
    pub user_id: Uuid,
    pub can_edit: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventOwner {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewEventOwner {
    pub user_id: Uuid,
}

#[test]
fn merge_events_1() {
    let mut entries = vec![];
    let id = Uuid::new_v4();
    entries.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:00 UTC),
        ),
        None,
    ));
    entries.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-19 10:00 UTC),
            datetime!(2023-02-19 12:00 UTC),
        ),
        None,
    ));
    entries.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-20 10:00 UTC),
            datetime!(2023-02-20 12:00 UTC),
        ),
        None,
    ));
    let events = Events::new(
        HashMap::from([(
            id,
            Event::new(
                EventPrivileges::Owned,
                EventPayload::new(String::from("A"), None),
                None,
                datetime!(2023-02-18 10:00 UTC),
            ),
        )]),
        entries,
    );

    let mut other_entries = vec![];
    let other_id = Uuid::new_v4();
    other_entries.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-17 10:00 UTC),
            datetime!(2023-02-17 12:00 UTC),
        ),
        None,
    ));
    other_entries.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-20 10:00 UTC),
            datetime!(2023-02-20 12:00 UTC),
        ),
        None,
    ));
    other_entries.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-21 10:00 UTC),
            datetime!(2023-02-21 12:00 UTC),
        ),
        None,
    ));

    let other_events = Events::new(
        HashMap::from([(
            id,
            Event::new(
                EventPrivileges::Owned,
                EventPayload::new(String::from("A"), None),
                None,
                datetime!(2023-02-17 10:00 UTC),
            ),
        )]),
        other_entries,
    );

    let merged = events.merge(other_events);
    let mut expected = vec![];

    expected.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-17 10:00 UTC),
            datetime!(2023-02-17 12:00 UTC),
        ),
        None,
    ));
    expected.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:00 UTC),
        ),
        None,
    ));
    expected.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-19 10:00 UTC),
            datetime!(2023-02-19 12:00 UTC),
        ),
        None,
    ));
    expected.push(Entry::new(
        id,
        TimeRange::new(
            datetime!(2023-02-20 10:00 UTC),
            datetime!(2023-02-20 12:00 UTC),
        ),
        None,
    ));

    expected.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-20 10:00 UTC),
            datetime!(2023-02-20 12:00 UTC),
        ),
        None,
    ));
    expected.push(Entry::new(
        other_id,
        TimeRange::new(
            datetime!(2023-02-21 10:00 UTC),
            datetime!(2023-02-21 12:00 UTC),
        ),
        None,
    ));

    println!("{:#?}", merged);
    for (a, b) in merged.entries.iter().zip(expected.iter()) {
        assert_eq!(a.time_range.start, b.time_range.start)
    }
}
