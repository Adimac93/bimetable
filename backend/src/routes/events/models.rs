use crate::utils::events::models::RecurrenceRule;
use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid};
use std::collections::HashMap;
use time::macros::datetime;
use time::serde::iso8601;
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
    pub recurrence_rule: Option<RecurrenceRule>,
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
    #[serde(with = "iso8601")]
    pub override_starts_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub override_ends_at: OffsetDateTime,
    pub data: OptionalEventData,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteEvent {
    pub event_id: Uuid,
    pub is_permanent: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteOverride {
    pub override_id: Uuid,
    pub is_permanent: bool,
}

// Receive payloads
#[derive(Debug, Deserialize, Serialize, ToResponse, ToSchema, PartialEq)]
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
        self.entries.sort_by_key(|entry| entry.starts_at);
        self
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct Event {
    pub payload: EventPayload,
    pub is_owned: bool,
    pub can_edit: bool,
}

#[derive(Debug)]
pub enum EventPrivileges {
    Owned,
    Shared { can_edit: bool },
}

impl Event {
    pub fn new(privileges: EventPrivileges, payload: EventPayload) -> Self {
        match privileges {
            EventPrivileges::Owned => Self {
                payload,
                is_owned: true,
                can_edit: true,
            },
            EventPrivileges::Shared { can_edit } => Self {
                payload,
                is_owned: false,
                can_edit,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema, PartialEq)]
pub struct Entry {
    pub event_id: Uuid,
    pub starts_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
    pub recurrence_override: Option<Override>,
}

impl Entry {
    pub fn new(
        event_id: Uuid,
        starts_at: OffsetDateTime,
        ends_at: OffsetDateTime,
        recurrence_override: Option<Override>,
    ) -> Self {
        Self {
            event_id,
            starts_at,
            ends_at,
            recurrence_override,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema, PartialEq)]
pub struct Override {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "iso8601::option", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[test]
fn merge_events_1() {
    let mut entries = vec![];
    let id = Uuid::new_v4();
    entries.push(Entry::new(
        id,
        datetime!(2023-02-18 10:00 UTC),
        datetime!(2023-02-18 12:00 UTC),
        None,
    ));
    entries.push(Entry::new(
        id,
        datetime!(2023-02-19 10:00 UTC),
        datetime!(2023-02-19 12:00 UTC),
        None,
    ));
    entries.push(Entry::new(
        id,
        datetime!(2023-02-20 10:00 UTC),
        datetime!(2023-02-20 12:00 UTC),
        None,
    ));
    let events = Events::new(
        HashMap::from([(
            id,
            Event::new(
                EventPrivileges::Owned,
                EventPayload::new(String::from("A"), None),
            ),
        )]),
        entries,
    );

    let mut other_entries = vec![];
    let other_id = Uuid::new_v4();
    other_entries.push(Entry::new(
        other_id,
        datetime!(2023-02-17 10:00 UTC),
        datetime!(2023-02-17 12:00 UTC),
        None,
    ));
    other_entries.push(Entry::new(
        other_id,
        datetime!(2023-02-20 10:00 UTC),
        datetime!(2023-02-20 12:00 UTC),
        None,
    ));
    other_entries.push(Entry::new(
        other_id,
        datetime!(2023-02-21 10:00 UTC),
        datetime!(2023-02-21 12:00 UTC),
        None,
    ));

    let other_events = Events::new(
        HashMap::from([(
            id,
            Event::new(
                EventPrivileges::Owned,
                EventPayload::new(String::from("A"), None),
            ),
        )]),
        other_entries,
    );

    let merged = events.merge(other_events);
    let mut expected = vec![];

    expected.push(Entry::new(
        other_id,
        datetime!(2023-02-17 10:00 UTC),
        datetime!(2023-02-17 12:00 UTC),
        None,
    ));
    expected.push(Entry::new(
        id,
        datetime!(2023-02-18 10:00 UTC),
        datetime!(2023-02-18 12:00 UTC),
        None,
    ));
    expected.push(Entry::new(
        id,
        datetime!(2023-02-19 10:00 UTC),
        datetime!(2023-02-19 12:00 UTC),
        None,
    ));
    expected.push(Entry::new(
        id,
        datetime!(2023-02-20 10:00 UTC),
        datetime!(2023-02-20 12:00 UTC),
        None,
    ));

    expected.push(Entry::new(
        other_id,
        datetime!(2023-02-20 10:00 UTC),
        datetime!(2023-02-20 12:00 UTC),
        None,
    ));
    expected.push(Entry::new(
        other_id,
        datetime!(2023-02-21 10:00 UTC),
        datetime!(2023-02-21 12:00 UTC),
        None,
    ));

    println!("{:#?}", merged);
    for (a, b) in merged.entries.iter().zip(expected.iter()) {
        assert_eq!(a.starts_at, b.starts_at)
    }
}
