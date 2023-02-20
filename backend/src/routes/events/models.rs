use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::serde::timestamp;

use crate::utils::events::models::EventRules;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsQuery {
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    pub name: String,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_rule: Option<EventRules>,
}
