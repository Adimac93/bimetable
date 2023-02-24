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
pub struct EventData {
    pub name: String,
    pub description: Option<String>,
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
    pub event_id: Uuid,
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
    event_id: Uuid,
    is_permanent: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteOverride {
    override_id: Uuid,
    is_permanent: bool,
}

// Receive payloads
#[derive(Debug, Deserialize, Serialize)]
pub struct Events {
    owned: Vec<Event>,
    shared: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub data: EventData,
    pub recurrences: Vec<RecurrenceEvent>, // Option<Vec<T>> ?
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecurrenceEvent {
    pub starts_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
    pub recurrence_override: Option<Override>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Override {
    pub data: OptionalEventData,
    created_at: OffsetDateTime,
}
