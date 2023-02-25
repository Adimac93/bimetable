use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::{serde::timestamp, Duration};

use super::{
    calculations::{CountToUntilData, EventRangeData},
    count_to_until::{
        daily_conv, monthly_conv_by_day, monthly_conv_by_weekday, weekly_conv, yearly_conv_by_day,
        yearly_conv_by_weekday,
    },
    errors::EventError,
    event_range::{
        get_daily_events, get_monthly_events_by_day, get_weekly_events,
        get_yearly_events_by_weekday,
    },
};

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
    pub fn count_to_until(
        &self,
        part_starts_at: OffsetDateTime,
        count: u32,
        event: &TimeRange,
    ) -> Result<OffsetDateTime, EventError> {
        let mut conv_data = CountToUntilData {
            part_starts_at,
            count,
            interval: 0,
            event_duration: event.duration(),
        };
        match self {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => {
                conv_data.interval = time_rules.interval;
                if *is_by_day {
                    yearly_conv_by_day(conv_data)
                } else {
                    yearly_conv_by_weekday(conv_data)
                }
            }
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => {
                conv_data.interval = time_rules.interval;
                if *is_by_day {
                    monthly_conv_by_day(conv_data)
                } else {
                    monthly_conv_by_weekday(conv_data)
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
                weekly_conv(conv_data, &string_week_map)
            }
            EventRules::Daily { time_rules } => {
                conv_data.interval = time_rules.interval;
                daily_conv(conv_data)
            }
        }
    }

    pub fn get_event_range(
        &self,
        part: TimeRange,
        event: TimeRange,
    ) -> Result<Vec<TimeRange>, EventError> {
        let t_rules = self.time_rules();

        let mut range_data = EventRangeData {
            range: part,
            event_range: event,
            rec_ends_at: None,
            interval: t_rules.interval,
        };

        range_data.rec_ends_at = t_rules.ends_at.as_ref().and_then(|x| match x {
            RecurrenceEndsAt::Count(n) => Some(self.count_to_until(part.start, *n, &event).ok()?),
            RecurrenceEndsAt::Until(t) => Some(*t),
        });

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

    fn time_rules(&self) -> TimeRules {
        let res = match self {
            EventRules::Yearly { time_rules, .. } => time_rules,
            EventRules::Monthly { time_rules, .. } => time_rules,
            EventRules::Weekly { time_rules, .. } => time_rules,
            EventRules::Daily { time_rules } => time_rules,
        };
        res.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RecurrenceEndsAt {
    Until(OffsetDateTime),
    Count(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn is_contained(&self, other: &Self) -> bool {
        other.start <= self.start && other.end >= self.end
    }

    pub fn is_after(&self, other: &Self) -> bool {
        self.start >= other.end
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}
