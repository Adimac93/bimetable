use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, uuid::Uuid, Json};
use time::{serde::iso8601, Duration};
use utoipa::ToSchema;

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
pub struct EventPart {
    pub starts_at: OffsetDateTime,
    pub length: Option<RecurrenceEndsAt>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum RecurrenceRule {
    #[serde(rename_all = "camelCase")]
    Yearly {
        time_rules: TimeRules,
        is_by_day: bool,
    },
    #[serde(rename_all = "camelCase")]
    Monthly {
        time_rules: TimeRules,
        is_by_day: bool,
    },
    #[serde(rename_all = "camelCase")]
    Weekly { time_rules: TimeRules, week_map: u8 },
    #[serde(rename_all = "camelCase")]
    Daily { time_rules: TimeRules },
}

impl RecurrenceRule {
    /// Returns the end of the nth occurrence of the event, starting from a specified point in time.
    ///
    /// The first event in the given time bound counts as the 0th event.
    ///
    /// Currently, the point in time the search starts in must be the same as the beggining of any event occurrence.
    ///
    /// ```rust
    /// use bimetable::utils::events::models::TimeRules;
    /// use bimetable::utils::events::models::RecurrenceRule;
    /// use bimetable::utils::events::models::TimeRange;
    /// use bimetable::utils::events::models::RecurrenceEndsAt;
    /// use time::macros::datetime;
    ///
    /// let event = TimeRange::new(
    ///     datetime!(2023-02-18 10:00 UTC),
    ///     datetime!(2023-02-18 12:15 UTC),
    /// );
    /// let rec_rules = RecurrenceRule::Daily {
    ///     time_rules: TimeRules {
    ///         ends_at: Some(RecurrenceEndsAt::Count(15)),
    ///         interval: 3,
    ///     },
    /// };
    ///
    /// assert_eq!(
    ///     rec_rules.count_to_until(datetime!(2023-02-21 10:00 UTC), 1, &event).unwrap(),
    ///     datetime!(2023-02-24 12:15 UTC)
    /// )
    /// ```
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
            RecurrenceRule::Yearly {
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
            RecurrenceRule::Monthly {
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
            RecurrenceRule::Weekly {
                time_rules,
                week_map,
            } => {
                conv_data.interval = time_rules.interval;
                let string_week_map = format!("{:0>7b}", week_map % 128);
                weekly_conv(conv_data, &string_week_map)
            }
            RecurrenceRule::Daily { time_rules } => {
                conv_data.interval = time_rules.interval;
                daily_conv(conv_data)
            }
        }
    }

    /// Returns all event occurences in a given range.
    ///
    /// For an event occurrence to be included in the result, it must overlap with the given range,
    /// which means that the occurrence must end strictly after the range, and vice versa.
    ///
    /// ```rust
    /// use bimetable::utils::events::models::RecurrenceRule;
    /// use bimetable::utils::events::models::TimeRules;
    /// use bimetable::utils::events::models::RecurrenceEndsAt;
    /// use bimetable::utils::events::models::TimeRange;
    /// use time::macros::datetime;
    ///
    /// let event = TimeRange::new(
    ///     datetime!(2023-02-17 22:45 UTC),
    ///     datetime!(2023-02-18 0:00 UTC),
    /// );
    /// let rec_rules = RecurrenceRule::Daily {
    ///     time_rules: TimeRules {
    ///         ends_at: Some(RecurrenceEndsAt::Count(50)),
    ///         interval: 2,
    ///     },
    /// };
    /// let part = TimeRange {
    ///     start: datetime!(2023-02-21 0:00 UTC),
    ///     end: datetime!(2023-02-27 22:45 UTC),
    /// };
    ///
    /// assert_eq!(
    ///     rec_rules.get_event_range(part, event).unwrap(),
    ///     vec![
    ///         TimeRange::new(
    ///             datetime!(2023-02-21 22:45 UTC),
    ///             datetime!(2023-02-22 0:00 UTC)
    ///         ),
    ///         TimeRange::new(
    ///             datetime!(2023-02-23 22:45 UTC),
    ///             datetime!(2023-02-24 0:00 UTC)
    ///         ),
    ///         TimeRange::new(
    ///             datetime!(2023-02-25 22:45 UTC),
    ///             datetime!(2023-02-26 0:00 UTC)
    ///         ),
    ///     ]
    /// )
    /// ```
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
            RecurrenceRule::Yearly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                if *is_by_day {
                    // year and 12 months are the same
                    range_data.interval *= 12;
                    get_monthly_events_by_day(range_data, *is_by_day)
                } else {
                    get_yearly_events_by_weekday(range_data)
                }
            }
            RecurrenceRule::Monthly {
                time_rules,
                is_by_day,
            } => {
                range_data.interval = time_rules.interval;
                get_monthly_events_by_day(range_data, *is_by_day)
            }
            RecurrenceRule::Weekly {
                time_rules,
                week_map,
            } => {
                range_data.interval = time_rules.interval;
                let string_week_map = format!("{:0>7b}", week_map % 128);
                get_weekly_events(range_data, &string_week_map)
            }
            RecurrenceRule::Daily { time_rules } => {
                range_data.interval = time_rules.interval;
                get_daily_events(range_data)
            }
        }
    }

    pub fn time_rules(&self) -> TimeRules {
        let res = match self {
            RecurrenceRule::Yearly { time_rules, .. } => time_rules,
            RecurrenceRule::Monthly { time_rules, .. } => time_rules,
            RecurrenceRule::Weekly { time_rules, .. } => time_rules,
            RecurrenceRule::Daily { time_rules } => time_rules,
        };
        res.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum RecurrenceEndsAt {
    #[serde(with = "iso8601")]
    Until(OffsetDateTime),
    Count(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
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

    pub fn new_relative_checked(start: OffsetDateTime, length: Duration) -> Option<Self> {
        Some(Self::new(start, start.checked_add(length)?))
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
