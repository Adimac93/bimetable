use std::{cmp::Ordering, fmt::Display};

use thiserror::Error;
use time::Duration;

use crate::{
    routes::events::models::{
        CreateEvent, Event, EventData, GetEventsQuery, OptionalEventData, OverrideEvent,
        UpdateEvent,
    },
    utils::events::models::{RecurrenceEndsAt, RecurrenceRule, TimeRange, TimeRules},
};

#[derive(Debug, Error)]
pub struct BimetableValidationError {
    pub content: String,
}

impl Display for BimetableValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl BimetableValidationError {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

pub trait BimetableValidate {
    fn b_validate(&self) -> Result<(), BimetableValidationError>;
}

impl BimetableValidate for TimeRange {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.duration() > Duration::seconds(0) {
            return Err(BimetableValidationError::new(
                "TimeRange duration is negative",
            ));
        } else {
            Ok(())
        }
    }
}

impl BimetableValidate for RecurrenceRule {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.time_rules().b_validate().is_err() {
            return Err(BimetableValidationError::new("Incorrect time rules"));
        }
        if let RecurrenceRule::Weekly {
            time_rules: _,
            week_map,
        } = self
        {
            if *week_map == 0 {
                return Err(BimetableValidationError::new("No events in the week map"));
            }
        };
        Ok(())
    }
}

impl BimetableValidate for TimeRules {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.interval == 0 {
            Err(BimetableValidationError::new(
                "Time rule interval is equal to 0",
            ))
        } else {
            Ok(())
        }
    }
}

impl BimetableValidate for EventData {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        TimeRange::new(self.starts_at, self.ends_at).b_validate()
    }
}

impl BimetableValidate for CreateEvent {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        self.data.b_validate()?;

        let Some(rule) = &self.recurrence_rule else {
            return Ok(());
        };

        rule.b_validate()?;

        let until = match rule.time_rules().ends_at {
            Some(RecurrenceEndsAt::Count(n)) => rule
                .count_to_until(
                    self.data.starts_at,
                    n,
                    &TimeRange::new(self.data.starts_at, self.data.ends_at),
                )
                .map_err(|_| {
                    BimetableValidationError::new(
                        "Failed to convert event count to its recurrence end time",
                    )
                })?,
            Some(RecurrenceEndsAt::Until(t)) => t,
            None => return Ok(()),
        };

        if until < self.data.ends_at {
            Err(BimetableValidationError::new(
                "Recurrence ends sooner than the event ends",
            ))
        } else {
            Ok(())
        }
    }
}

impl BimetableValidate for OptionalEventData {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.starts_at.partial_cmp(&self.ends_at) == Some(Ordering::Greater) {
            Err(BimetableValidationError::new(
                "Event ends sooner than it starts",
            ))
        } else {
            Ok(())
        }
    }
}

impl BimetableValidate for GetEventsQuery {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        TimeRange::new(self.starts_at, self.ends_at).b_validate()
    }
}

impl BimetableValidate for UpdateEvent {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        self.data.b_validate()
    }
}

impl BimetableValidate for OverrideEvent {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        TimeRange::new(self.override_starts_at, self.override_ends_at).b_validate()?;
        self.data.b_validate()
    }
}

impl BimetableValidate for Event {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.is_owned && !self.can_edit {
            Err(BimetableValidationError::new(
                "The event owner must have editing privileges for it",
            ))
        } else {
            Ok(())
        }
    }
}
