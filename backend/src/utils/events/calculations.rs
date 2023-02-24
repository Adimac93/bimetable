use time::{Duration, OffsetDateTime};

use super::models::TimeRange;

pub struct CountToUntilData {
    pub part_starts_at: OffsetDateTime,
    pub count: u32,
    pub interval: u32,
    pub event_duration: Duration,
}

impl CountToUntilData {
    pub fn new(
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
    pub fn new(
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
