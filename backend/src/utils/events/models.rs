use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::{serde::timestamp, Duration};

use crate::app_errors::DefaultContext;

use super::{errors::EventError, calculations::{CountToUntilData, EventRangeData}, count_to_until::{year_is_by_day_count_to_until, year_count_to_until, month_is_by_day_count_to_until, month_count_to_until, week_count_to_until, day_count_to_until}, event_range::{get_monthly_events_by_day, get_yearly_events_by_weekday, get_weekly_events, get_daily_events}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: String,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_rule: Option<Json<EventRules>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventPart {
    pub starts_at: OffsetDateTime,
    pub length: Option<RecurrenceEndsAt>,
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
    pub fn count_to_until(&self, part: &EventPart, event: &TimeRange) -> Result<Option<OffsetDateTime>, EventError> {
        let Some(part_ends_at) = part.length.as_ref() else {
            return Ok(None)
        };

        let count = match part_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(n) => *n,
        };

        let mut conv_data = CountToUntilData::new(
            part.starts_at,
            count,
            0,
            event.duration(),
        );
        match self {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => {
                conv_data.interval = time_rules.interval;
                if *is_by_day {
                    year_is_by_day_count_to_until(conv_data)
                } else {
                    year_count_to_until(conv_data)
                }
            }
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => {
                conv_data.interval = time_rules.interval;
                if *is_by_day {
                    month_is_by_day_count_to_until(conv_data)
                } else {
                    month_count_to_until(conv_data)
                }
            }
            EventRules::Weekly {
                time_rules,
                week_map,
            } => {
                conv_data.interval = time_rules.interval;
                let string_week_map = format!("{:0>7b}", week_map % 128);
                if week_map % 128 == 0 {
                    return Err(EventError::InvalidEventFormat);
                }
                week_count_to_until(conv_data, &string_week_map)
            }
            EventRules::Daily { time_rules } => {
                conv_data.interval = time_rules.interval;
                day_count_to_until(conv_data)
            }
        }
    }

    pub fn get_event_range(&self, part: &EventPart, event: &TimeRange) -> Result<Vec<TimeRange>, EventError> {
        let part_ends_at = part.length.as_ref().ok_or(EventError::NotFound)?;

        let part_ends_at = match part_ends_at {
            RecurrenceEndsAt::Until(t) => *t,
            RecurrenceEndsAt::Count(_n) => self.count_to_until(part, event)?.dc()?,
        };

        let mut range_data = EventRangeData::new(
            part.starts_at,
            part_ends_at,
            0,
            event.start,
            event.end,
        );

        match self {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                if *is_by_day {
                    // year and 12 months are the same
                    range_data.interval *= 12;
                    Ok(get_monthly_events_by_day(range_data, *is_by_day))
                } else {
                    get_yearly_events_by_weekday(range_data)
                }
            }
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                Ok(get_monthly_events_by_day(range_data, *is_by_day))
            }
            EventRules::Weekly {
                time_rules,
                week_map,
            } => {
                range_data.interval = time_rules.interval;
                let string_week_map = format!("{:0>7b}", week_map % 128);
                if week_map % 128 == 0 {
                    return Err(EventError::InvalidEventFormat);
                }
                Ok(get_weekly_events(range_data, &string_week_map))
            }
            EventRules::Daily { time_rules } => {
                range_data.interval = time_rules.interval;
                Ok(get_daily_events(range_data))
            }
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeRange {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl TimeRange {
    pub fn new(start: OffsetDateTime, end: OffsetDateTime) -> Self {
        Self { start, end }
    }

    pub fn new_relative(start: OffsetDateTime, length: Duration) -> Self {
        Self::new(start, start + length)
    }

    pub fn checked_add(self, rhs: Duration) -> Option<Self> {
        Some(Self::new(
            self.start.checked_add(rhs)?,
            self.end.checked_add(rhs)?,
        ))
    }

    pub fn is_before(&self, other: &Self) -> bool {
        self.end <= other.start
    }

    pub fn is_overlapping(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }

    pub fn is_after(&self, other: &Self) -> bool {
        self.start >= other.end
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}
