use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid};
use time::serde::timestamp;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Uuid,
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsQuery {
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
}
