use sqlx::types::Json;
use time::{Duration, OffsetDateTime};

use crate::app_errors::DefaultContext;

use super::{
    count_to_until::{
        day_count_to_until, month_count_to_until, month_is_by_day_count_to_until,
        week_count_to_until, year_count_to_until, year_is_by_day_count_to_until,
    },
    errors::EventError,
    event_range::{
        get_daily_events, get_monthly_events_by_day, get_monthly_events_by_weekday,
        get_weekly_events, get_yearly_events_by_day, get_yearly_events_by_weekday,
    },
    models::{EventPart, EventRules, RecurrenceEndsAt, TimeRange},
};

pub struct CountToUntilData {
    pub part_starts_at: OffsetDateTime,
    pub count: u32,
    pub interval: u32,
    pub event_duration: Duration,
}

impl CountToUntilData {
    fn new(
        part_starts_at: OffsetDateTime,
        count: u32,
        interval: u32,
        event_duration: Duration,
    ) -> Self {
        Self {
            part_starts_at,
            count,
            interval,
            event_duration,
        }
    }
}

pub struct EventRangeData {
    pub range: TimeRange,
    pub interval: u32,
    pub event_range: TimeRange,
}

impl EventRangeData {
    fn new(
        part_starts_at: OffsetDateTime,
        part_ends_at: OffsetDateTime,
        interval: u32,
        event_starts_at: OffsetDateTime,
        event_ends_at: OffsetDateTime,
    ) -> Self {
        Self {
            range: TimeRange::new(part_starts_at, part_ends_at),
            interval,
            event_range: TimeRange::new(event_starts_at, event_ends_at),
        }
    }
}

impl EventPart {
    pub fn verify_event_part(&self) -> Result<(), EventError> {
        match self.part_length {
            Some(RecurrenceEndsAt::Count(x)) if x == 0 => Err(EventError::WrongEventBounds),
            Some(RecurrenceEndsAt::Until(time)) if self.part_starts_at > time => {
                Err(EventError::WrongEventBounds)
            }
            _ => Ok(()),
        }
    }

    pub fn count_to_until(&self) -> Result<Option<OffsetDateTime>, EventError> {
        let (rec_rules, event_starts_at, event_ends_at) = self.get_recurrence_data()?;

        let Some(part_ends_at) = self.part_length.as_ref() else {
            return Ok(None)
        };

        let count = match part_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(n) => *n,
        };

        let mut conv_data = CountToUntilData::new(
            self.part_starts_at,
            count,
            0,
            event_ends_at - event_starts_at,
        );
        match rec_rules {
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

    pub fn get_event_range(&self) -> Result<Vec<TimeRange>, EventError> {
        let (rec_rules, event_starts_at, event_ends_at) = self.get_recurrence_data()?;

        let part_ends_at = self.part_length.as_ref().ok_or(EventError::NotFound)?;

        let part_ends_at = match part_ends_at {
            RecurrenceEndsAt::Until(t) => *t,
            RecurrenceEndsAt::Count(_n) => self.count_to_until()?.dc()?,
        };

        let mut range_data = EventRangeData::new(
            self.part_starts_at,
            part_ends_at,
            0,
            event_starts_at,
            event_ends_at,
        );

        match rec_rules {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                if *is_by_day {
                    Ok(get_yearly_events_by_day(range_data))
                } else {
                    get_yearly_events_by_weekday(range_data)
                }
            }
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                if *is_by_day {
                    Ok(get_monthly_events_by_day(range_data))
                } else {
                    Ok(get_monthly_events_by_weekday(range_data))
                }
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

    fn get_recurrence_data(&self) -> anyhow::Result<(&EventRules, OffsetDateTime, OffsetDateTime)> {
        let Json(rec_rules) = self
            .event_data
            .recurrence_rule
            .as_ref()
            .ok_or(EventError::NotFound)?;

        let event_starts_at = self.event_data.starts_at.ok_or(EventError::NotFound)?;

        let event_ends_at = self.event_data.ends_at.ok_or(EventError::NotFound)?;

        Ok((rec_rules, event_starts_at, event_ends_at))
    }
}
