use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::{serde::timestamp, Duration};

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
#[serde(rename_all = "snake_case")]
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

impl EventRules {
    pub fn rule(&self, starts_at: OffsetDateTime, ends_at: OffsetDateTime) {
        match self {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => todo!(),
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => todo!(),
            EventRules::Weekly {
                time_rules,
                week_map,
            } => {
                format!("{:0>7b}", week_map % 128);
                todo!();
            }
            EventRules::Daily { time_rules } => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RecurrenceEndsAt {
    Until(OffsetDateTime),
    Count(u32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeRules {
    pub ends_at: Option<RecurrenceEndsAt>,
    pub interval: u32,
}

impl TimeRules {
    fn last_event_recurrence(
        &self,
        event_starts_at: OffsetDateTime,
        event_ends_at: OffsetDateTime,
        mult: Duration,
    ) -> Result<Option<OffsetDateTime>, anyhow::Error> {
        if let Some(rec_ends_at) = &self.ends_at {
            match rec_ends_at {
                RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
                RecurrenceEndsAt::Count(n) => {
                    let event_duration: Duration = event_ends_at - event_starts_at; // overflow?
                    let time_to_next_event: Duration = event_duration
                        .checked_add(
                            mult.checked_mul(
                                i32::try_from(self.interval).context("Deep overflow!")?,
                            )
                            .context("Not too deep overflow!")?,
                        )
                        .context("Surface overflow!")?;
                    let rec_ends_at: OffsetDateTime = event_starts_at
                        .checked_add(
                            time_to_next_event
                                .checked_mul(i32::try_from(*n).context("Deep overflow!")?)
                                .context("Not too deep overflow!")?,
                        )
                        .context("Surface overflow!")?;
                    return Ok(Some(rec_ends_at));
                }
            }
        }
        Ok(None) // never
    }
}
