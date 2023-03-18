use crate::utils::events::RecurrenceJSON;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::time::OffsetDateTime;
use std::fmt::{Display, Formatter};
use time::macros::{datetime, format_description};
use time::{serde::iso8601, Duration};
use tracing::trace;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::validation::ValidateContent;

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

pub struct EventPart {
    pub starts_at: OffsetDateTime,
    pub length: Option<EntriesSpan>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RecurrenceRule {
    pub span: Option<EntriesSpan>,
    pub interval: u32,
    pub kind: RecurrenceRuleKind,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct EntriesSpan {
    pub end: OffsetDateTime,
    pub repetitions: u32,
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
