use time::{Duration, OffsetDateTime};

use super::models::TimeRange;

pub struct CountToUntilData {
    pub part_starts_at: OffsetDateTime,
    pub count: u32,
    pub interval: u32,
    pub event_duration: Duration,
}

pub struct EventRangeData {
    pub range: TimeRange,
    pub event_range: TimeRange,
    pub rec_ends_at: Option<OffsetDateTime>,
    pub interval: u32,
}
