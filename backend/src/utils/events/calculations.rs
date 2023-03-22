use crate::validation::{ValidateContent, ValidateContentError};
use time::{Duration, OffsetDateTime};

use super::models::TimeRange;

pub struct CountToUntilData {
    pub part_starts_at: OffsetDateTime,
    pub count: u32,
    pub interval: u32,
    pub event_duration: Duration,
}

pub struct UntilToCountData {
    pub part_starts_at: OffsetDateTime,
    pub until: OffsetDateTime,
    pub interval: u32,
}

impl ValidateContent for UntilToCountData {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        TimeRange::new(self.part_starts_at, self.until).validate_content()?;
        if self.interval == 0 {
            return Err(ValidateContentError::Expected(
                "Interval is equal to 0".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct EventRangeData {
    pub range: TimeRange,
    pub event_range: TimeRange,
    pub rec_ends_at: Option<OffsetDateTime>,
    pub interval: u32,
}
