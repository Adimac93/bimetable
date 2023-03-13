use http::StatusCode;
use thiserror::Error;
use time::Duration;
use tracing::error;

use crate::{
    app_errors::DefaultContext,
    routes::events::models::{
        CreateEvent, Event, EventData, GetEventsQuery, OptionalEventData, OverrideEvent,
        UpdateEvent,
    },
    utils::events::models::{
        RecurrenceEndsAt, RecurrenceRule, RecurrenceRuleKind, TimeRange, TimeRules,
    },
};

#[derive(Debug, Error)]
pub enum ValidateContentError {
    #[error("Data rejected with validation")]
    Expected(String),
    #[error("Unexpected server error")]
    Unexpected(#[from] anyhow::Error),
}

impl ValidateContentError {
    pub fn new(content: impl ToString) -> Self {
        Self::Expected(content.to_string())
    }
}

impl From<&ValidateContentError> for StatusCode {
    fn from(value: &ValidateContentError) -> Self {
        match value {
            ValidateContentError::Expected(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ValidateContentError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub trait ValidateContent {
    fn validate_content(&self) -> Result<(), ValidateContentError>;
}

impl ValidateContent for TimeRange {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        if self.duration() < Duration::seconds(0) {
            Err(ValidateContentError::new("TimeRange duration is negative"))
        } else {
            Ok(())
        }
    }
}

impl ValidateContent for TimeRules {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        if self.interval == 0 {
            Err(ValidateContentError::new(
                "Time rule interval is equal to 0",
            ))
        } else {
            Ok(())
        }
    }
}

impl ValidateContent for RecurrenceRule {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        if self.time_rules.validate_content().is_err() {
            return Err(ValidateContentError::new("Incorrect time rules"));
        }
        if let RecurrenceRuleKind::Weekly { week_map: 0 } = self.kind {
            return Err(ValidateContentError::new("No events in the week map"));
        };
        Ok(())
    }
}

impl ValidateContent for EventData {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        TimeRange::new(self.starts_at, self.ends_at).validate_content()
    }
}

impl ValidateContent for CreateEvent {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        self.data.validate_content()?;

        let Some(rule) = &self.recurrence_rule else {
            return Ok(());
        };

        rule.validate_content()?;

        let until = match rule.time_rules.ends_at {
            Some(RecurrenceEndsAt::Count(n)) => rule
                .count_to_until(
                    self.data.starts_at,
                    n,
                    &TimeRange::new(self.data.starts_at, self.data.ends_at),
                )
                .dc()?,
            Some(RecurrenceEndsAt::Until(t)) => t,
            None => return Ok(()),
        };

        if until < self.data.ends_at {
            Err(ValidateContentError::new(
                "Recurrence ends sooner than the event ends",
            ))
        } else {
            Ok(())
        }
    }
}

impl ValidateContent for OptionalEventData {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        match (self.starts_at, self.ends_at) {
            (Some(start), Some(end)) if start > end => Err(ValidateContentError::new(
                "Event ends sooner than it starts",
            )),
            _ => Ok(()),
        }
    }
}

impl ValidateContent for GetEventsQuery {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        TimeRange::new(self.starts_at, self.ends_at).validate_content()
    }
}

impl ValidateContent for UpdateEvent {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        self.data.validate_content()
    }
}

impl ValidateContent for OverrideEvent {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        TimeRange::new(self.override_starts_at, self.override_ends_at).validate_content()?;
        self.data.validate_content()
    }
}

impl ValidateContent for Event {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        if self.is_owned && !self.can_edit {
            Err(ValidateContentError::new(
                "The event owner must have editing privileges for it",
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use time::macros::datetime;

    use crate::routes::events::models::EventPayload;

    use super::*;

    #[test]
    fn time_range_validation_ok() {
        let data = TimeRange::new(
            datetime!(2023-03-01 13:00 UTC),
            datetime!(2023-03-01 13:01 UTC),
        );
        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn time_range_validation_err() {
        let data = TimeRange::new(
            datetime!(2023-03-01 13:00 UTC),
            datetime!(2023-03-01 12:59 UTC),
        );
        assert!(data.validate_content().is_err())
    }

    #[test]
    fn time_rules_validation_ok_1() {
        let data = TimeRules {
            ends_at: Some(RecurrenceEndsAt::Count(0)),
            interval: 1,
        };
        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn time_rules_validation_ok_2() {
        let data = TimeRules {
            ends_at: None,
            interval: 1,
        };
        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn time_rules_validation_err() {
        let data = TimeRules {
            ends_at: Some(RecurrenceEndsAt::Count(0)),
            interval: 0,
        };
        assert!(data.validate_content().is_err())
    }

    #[test]
    fn recurrence_rule_validation_ok() {
        let data = RecurrenceRule {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 1 },
        };
        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn recurrence_rule_validation_err_1() {
        let data = RecurrenceRule {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 0,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 1 },
        };
        assert!(data.validate_content().is_err())
    }

    #[test]
    fn recurrence_rule_validation_err_2() {
        let data = RecurrenceRule {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 0 },
        };
        assert!(data.validate_content().is_err())
    }

    #[test]
    fn create_event_validation_ok() {
        let data = CreateEvent {
            data: EventData {
                payload: EventPayload {
                    name: "test_name".to_string(),
                    description: Some("test_desc".to_string()),
                },
                starts_at: datetime!(2023-03-01 12:00 UTC),
                ends_at: datetime!(2023-03-02 12:00 UTC),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                kind: RecurrenceRuleKind::Weekly { week_map: 1 },
            }),
        };

        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn create_event_validation_err_1() {
        let data = CreateEvent {
            data: EventData {
                payload: EventPayload {
                    name: "test_name".to_string(),
                    description: Some("test_desc".to_string()),
                },
                starts_at: datetime!(2023-03-01 12:00 UTC),
                ends_at: datetime!(2023-03-02 12:00 UTC),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 0,
                },
                kind: RecurrenceRuleKind::Weekly { week_map: 1 },
            }),
        };

        assert!(data.validate_content().is_err())
    }

    #[test]
    fn create_event_validation_err_2() {
        let data = CreateEvent {
            data: EventData {
                payload: EventPayload {
                    name: "test_name".to_string(),
                    description: Some("test_desc".to_string()),
                },
                starts_at: datetime!(2023-03-01 12:00 UTC),
                ends_at: datetime!(2023-03-02 12:00 UTC),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                kind: RecurrenceRuleKind::Weekly { week_map: 0 },
            }),
        };

        assert!(data.validate_content().is_err())
    }

    #[test]
    fn create_event_validation_err_3() {
        let data = CreateEvent {
            data: EventData {
                payload: EventPayload {
                    name: "test_name".to_string(),
                    description: Some("test_desc".to_string()),
                },
                starts_at: datetime!(2023-03-01 12:01 UTC),
                ends_at: datetime!(2023-03-01 12:00 UTC),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                kind: RecurrenceRuleKind::Weekly { week_map: 1 },
            }),
        };

        assert!(data.validate_content().is_err())
    }

    #[test]
    fn create_event_validation_err_4() {
        let data = CreateEvent {
            data: EventData {
                payload: EventPayload {
                    name: "test_name".to_string(),
                    description: Some("test_desc".to_string()),
                },
                starts_at: datetime!(2023-03-01 12:00 UTC),
                ends_at: datetime!(2023-03-02 12:00 UTC),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-02 11:59 UTC))),
                    interval: 1,
                },
                kind: RecurrenceRuleKind::Weekly { week_map: 1 },
            }),
        };

        assert!(data.validate_content().is_err())
    }

    #[test]
    fn optional_event_data_validation_ok_1() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: None,
            ends_at: None,
        };

        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn optional_event_data_validation_ok_2() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: Some(datetime!(2023-03-01 12:00 UTC)),
            ends_at: None,
        };
        println!(
            "{:?}",
            Some(datetime!(2023-03-01 12:00 UTC)).partial_cmp(&None)
        );
        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn optional_event_data_validation_ok_3() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: None,
            ends_at: Some(datetime!(2023-03-01 12:00 UTC)),
        };

        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn optional_event_data_validation_ok_4() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: Some(datetime!(2023-03-01 12:00 UTC)),
            ends_at: Some(datetime!(2023-03-02 12:00 UTC)),
        };

        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn optional_event_data_validation_err() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: Some(datetime!(2023-03-01 12:00 UTC)),
            ends_at: Some(datetime!(2023-03-01 11:59 UTC)),
        };

        assert!(data.validate_content().is_err())
    }

    #[test]
    fn event_validation_ok() {
        let data = Event {
            payload: EventPayload {
                name: "test_name".to_string(),
                description: Some("test_desc".to_string()),
            },
            recurrence_rule: Some(RecurrenceRule {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(2)),
                    interval: 2,
                },
                kind: RecurrenceRuleKind::Daily,
            }),
            entries_start: datetime!(2023-03-01 12:00 UTC),
            entries_end: Some(datetime!(2023-03-03 13:00 UTC)),
            is_owned: true,
            can_edit: true,
        };

        assert!(data.validate_content().is_ok())
    }

    #[test]
    fn event_validation_err() {
        let data = Event {
            payload: EventPayload {
                name: "test_name".to_string(),
                description: Some("test_desc".to_string()),
            },
            recurrence_rule: None,
            entries_start: datetime!(2023-03-01 12:00 UTC),
            entries_end: Some(datetime!(2023-03-01 11:59 UTC)),
            is_owned: true,
            can_edit: false,
        };

        assert!(data.validate_content().is_err())
    }
}
