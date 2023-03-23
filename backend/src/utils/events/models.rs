use crate::utils::events::event_range::EventRangeData;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Json;
use std::fmt::{Display, Formatter};
use time::macros::{datetime, format_description};
use time::{serde::iso8601, Duration};
use tracing::trace;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::validation::ValidateContent;

use super::{
    errors::EventError,
    event_range::{
        get_daily_events, get_monthly_events_by_day, get_weekly_events,
        get_yearly_events_by_weekday,
    },
};

pub struct EventPart {
    pub starts_at: OffsetDateTime,
    pub length: Option<EntriesSpan>,
}

/// Computational struct.
///
/// Used for generating event entries and to be stored in the db.
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RecurrenceRule {
    pub span: Option<EntriesSpan>,
    pub interval: u32,
    pub kind: RecurrenceRuleKind,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone, Copy)]
pub struct EntriesSpan {
    pub end: OffsetDateTime,
    pub repetitions: u32,
}

impl RecurrenceRule {
    pub fn from_db_data(
        kind: Option<Json<RecurrenceRuleKind>>,
        until: Option<OffsetDateTime>,
        count: Option<i32>,
        interval: Option<i32>,
    ) -> Option<Self> {
        kind.and_then(|Json(rec_kind)| {
            Some(Self {
                span: if let (Some(u), Some(c)) = (until, count) {
                    Some(EntriesSpan {
                        end: u,
                        repetitions: c as u32,
                    })
                } else {
                    None
                },
                interval: interval? as u32,
                kind: rec_kind,
            })
        })
    }

    /// Returns all event occurences in a given range.
    ///
    /// For an event occurrence to be included in the result, it must overlap with the given range,
    /// which means that the occurrence must end strictly after the range, and vice versa.
    ///
    /// ```rust
    /// use bimetable::utils::events::models::{EntriesSpan, RecurrenceRuleKind};
    /// use bimetable::utils::events::models::RecurrenceRule;
    /// use bimetable::utils::events::models::TimeRange;
    /// use time::macros::datetime;
    /// use bimetable::routes::events::models::{RecurrenceEndsAt, RecurrenceRuleSchema, TimeRules};
    ///
    /// let event = TimeRange::new(
    ///     datetime!(2023-02-17 22:45 UTC),
    ///     datetime!(2023-02-18 0:00 UTC),
    /// );
    /// let rec_rules = RecurrenceRule {
    ///     span: Some(EntriesSpan {
    ///         end: datetime!(2100-12-31 23:59 UTC),
    ///         repetitions: 50
    ///     }),
    ///     interval: 2,
    ///     kind: RecurrenceRuleKind::Daily,
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
        // self.validate_content()?;

        let mut range_data = EventRangeData {
            range: part,
            event_range: event,
            rec_ends_at: self.span.map(|x| x.end),
            interval: self.interval,
        };

        let res = match self.kind {
            RecurrenceRuleKind::Yearly { is_by_day: true } => {
                // year and 12 months are the same
                range_data.interval *= 12;
                get_monthly_events_by_day(range_data, true)
            }
            RecurrenceRuleKind::Yearly { is_by_day: false } => {
                get_yearly_events_by_weekday(range_data)
            }
            RecurrenceRuleKind::Monthly { is_by_day } => {
                get_monthly_events_by_day(range_data, is_by_day)
            }
            RecurrenceRuleKind::Weekly { week_map } => {
                let string_week_map = format!("{:0>7b}", week_map % 128);
                get_weekly_events(range_data, &string_week_map)
            }
            RecurrenceRuleKind::Daily => get_daily_events(range_data),
        }?;

        trace!("Got {} event entries using a time range search", res.len());

        Ok(res)
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RecurrenceRuleKind {
    #[serde(rename_all = "camelCase")]
    Yearly { is_by_day: bool },
    #[serde(rename_all = "camelCase")]
    Monthly { is_by_day: bool },
    #[serde(rename_all = "camelCase")]
    Weekly { week_map: u8 },
    #[serde(rename_all = "camelCase")]
    Daily,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize, ToSchema)]
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

impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format = format_description!(
            "[year]-[month]-[day] [hour]:[minute] [offset_hour sign:mandatory]:[offset_minute]"
        );
        let start = self.start.format(&format).unwrap();
        let end = self.end.format(&format).unwrap();
        write!(f, "{start} - {end}")

        // non panic alternative?
        // write!(
        //     f,
        //     "{} {}:{} {} - {} {}:{} {}",
        //     self.start.date(),
        //     self.start.hour(),
        //     self.start.minute(),
        //     self.start.offset(),
        //     self.end.date(),
        //     self.end.hour(),
        //     self.end.minute(),
        //     self.end.offset(),
        // )
    }
}

pub struct UserEvent {
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub can_edit: bool,
}

impl UserEvent {
    pub fn new(user_id: Uuid, event_id: Uuid, can_edit: bool) -> Self {
        Self {
            user_id,
            event_id,
            can_edit,
        }
    }
}
