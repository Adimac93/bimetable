use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::serde::timestamp;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_rule: Option<Json<EventRules>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EventRules {
    Yearly {
        time_rules: TimeRules,
        is_by_day: bool,
    },
    Monthly {
        time_rules: TimeRules,
        is_by_day: bool,
    },
    Weekly {
        time_rules: TimeRules,
        week_map: u8,
    },
    Daily {
        time_rules: TimeRules,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EndsAt {
    Until(OffsetDateTime),
    Count(usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeRules {
    pub ends_at: Option<EndsAt>,
    pub interval: usize,
}
