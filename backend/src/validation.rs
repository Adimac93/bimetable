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
        if self.duration() < Duration::seconds(0) {
            return Err(BimetableValidationError::new(
                "TimeRange duration is negative",
            ));
        } else {
            Ok(())
        }
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

impl BimetableValidate for RecurrenceRule {
    fn b_validate(&self) -> Result<(), BimetableValidationError> {
        if self.time_rules().b_validate().is_err() {
            return Err(BimetableValidationError::new("Incorrect time rules"));
        }
        if let RecurrenceRule::Weekly {
            time_rules: _,
            week_map: 0,
        } = self
        {
            return Err(BimetableValidationError::new("No events in the week map"));
        };
        Ok(())
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
        match (self.starts_at, self.ends_at) {
            (Some(start), Some(end)) if start > end => Err(BimetableValidationError::new(
                "Event ends sooner than it starts",
            )),
            _ => Ok(()),
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
        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn time_range_validation_err() {
        let data = TimeRange::new(
            datetime!(2023-03-01 13:00 UTC),
            datetime!(2023-03-01 12:59 UTC),
        );
        assert!(data.b_validate().is_err())
    }

    #[test]
    fn time_rules_validation_ok_1() {
        let data = TimeRules {
            ends_at: Some(RecurrenceEndsAt::Count(0)),
            interval: 1,
        };
        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn time_rules_validation_ok_2() {
        let data = TimeRules {
            ends_at: None,
            interval: 1,
        };
        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn time_rules_validation_err() {
        let data = TimeRules {
            ends_at: Some(RecurrenceEndsAt::Count(0)),
            interval: 0,
        };
        assert!(data.b_validate().is_err())
    }

    #[test]
    fn recurrence_rule_validation_ok() {
        let data = RecurrenceRule::Weekly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 1,
            },
            week_map: 1,
        };
        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn recurrence_rule_validation_err_1() {
        let data = RecurrenceRule::Weekly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 0,
            },
            week_map: 1,
        };
        assert!(data.b_validate().is_err())
    }

    #[test]
    fn recurrence_rule_validation_err_2() {
        let data = RecurrenceRule::Weekly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-05 19:00 UTC))),
                interval: 1,
            },
            week_map: 0,
        };
        assert!(data.b_validate().is_err())
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
            recurrence_rule: Some(RecurrenceRule::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                week_map: 1,
            }),
        };

        assert!(data.b_validate().is_ok())
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
            recurrence_rule: Some(RecurrenceRule::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 0,
                },
                week_map: 1,
            }),
        };

        assert!(data.b_validate().is_err())
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
            recurrence_rule: Some(RecurrenceRule::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                week_map: 0,
            }),
        };

        assert!(data.b_validate().is_err())
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
            recurrence_rule: Some(RecurrenceRule::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-03 12:00 UTC))),
                    interval: 1,
                },
                week_map: 1,
            }),
        };

        assert!(data.b_validate().is_err())
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
            recurrence_rule: Some(RecurrenceRule::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-02 11:59 UTC))),
                    interval: 1,
                },
                week_map: 1,
            }),
        };

        assert!(data.b_validate().is_err())
    }

    #[test]
    fn optional_event_data_validation_ok_1() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: None,
            ends_at: None,
        };

        assert!(data.b_validate().is_ok())
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
        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn optional_event_data_validation_ok_3() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: None,
            ends_at: Some(datetime!(2023-03-01 12:00 UTC)),
        };

        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn optional_event_data_validation_ok_4() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: Some(datetime!(2023-03-01 12:00 UTC)),
            ends_at: Some(datetime!(2023-03-02 12:00 UTC)),
        };

        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn optional_event_data_validation_err() {
        let data = OptionalEventData {
            name: None,
            description: None,
            starts_at: Some(datetime!(2023-03-01 12:00 UTC)),
            ends_at: Some(datetime!(2023-03-01 11:59 UTC)),
        };

        assert!(data.b_validate().is_err())
    }

    #[test]
    fn event_validation_ok() {
        let data = Event {
            payload: EventPayload {
                name: "test_name".to_string(),
                description: Some("test_desc".to_string()),
            },
            is_owned: true,
            can_edit: true,
        };

        assert!(data.b_validate().is_ok())
    }

    #[test]
    fn event_validation_err() {
        let data = Event {
            payload: EventPayload {
                name: "test_name".to_string(),
                description: Some("test_desc".to_string()),
            },
            is_owned: true,
            can_edit: false,
        };

        assert!(data.b_validate().is_err())
    }
}
