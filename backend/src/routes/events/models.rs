use std::collections::HashMap;

use crate::utils::events::models::EventRules;
use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::serde::timestamp;

// Core data models
#[derive(Debug, Deserialize, Serialize)]
pub struct OptionalEventData {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventPayload {
    pub name: String,
    pub description: Option<String>,
}

impl EventPayload {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OptionalEventPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventData {
    pub payload: EventPayload,
    pub starts_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
}

// Queries
#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsQuery {
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
    pub include_shared: bool,
}

// Send payloads
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    pub data: EventData,
    pub recurrence_rule: Option<Json<EventRules>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateEvent {
    pub id: Uuid,
    pub data: OptionalEventData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OverrideEvent {
    pub event_id: Uuid,
    pub override_starts_at: OffsetDateTime,
    pub override_ends_at: OffsetDateTime,
    pub data: OptionalEventData,
}

#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Events {
    pub events: HashMap<Uuid, Event>,
    pub entries: Vec<Entry>,
}

impl Events {
    pub fn new(events: HashMap<Uuid, Event>, entries: Vec<Entry>) -> Self {
        Self { events, entries }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub payload: EventPayload,
    pub is_owned: bool,
    pub can_edit: bool,
}

pub enum EventPrivileges {
    Owned,
    Shared { can_edit: bool },
}
impl Event {
    pub fn new(mode: EventPrivileges, payload: EventPayload) -> Self {
        match mode {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entry {
    pub event_id: Uuid,
    pub starts_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
    pub recurrence_override: Option<Override>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Override {
    pub name: Option<String>,
    pub description: Option<String>,
    pub deleted_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}
