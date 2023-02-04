use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid};
use time::{serde::timestamp, macros::datetime};

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Event {
//     pub id: Uuid,
//     #[serde(with = "timestamp")]
//     pub starts_at: OffsetDateTime,
//     #[serde(with = "timestamp")]
//     pub ends_at: OffsetDateTime,
//     pub name: String,
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsQuery {
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
    pub name: String,
}

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
enum EventRules {
    Yearly { time_rules: TimeRules, is_by_day: bool },
    Monthly { time_rules: TimeRules, is_by_day: bool },
    Weekly { time_rules: TimeRules, week_map: u8 },
    Daily { time_rules: TimeRules },
}

#[derive(Serialize, Deserialize, Debug)]
enum EndsAt {
    Until(OffsetDateTime),
    Count(usize),
}

#[derive(Serialize, Deserialize, Debug)]
struct TimeRules {
    ends_at: Option<EndsAt>,
    interval: usize,
}
