use crate::utils::events::count_to_until::count_to_until;
use crate::utils::events::errors::EventError;
use crate::utils::events::models::{EntriesSpan, RecurrenceRule, RecurrenceRuleKind, TimeRange};
use crate::utils::events::until_to_count::until_to_count;
use time::macros::datetime;
use time::OffsetDateTime;

pub fn raw_prev_entry(
    provided_time: OffsetDateTime,
    first_entry: TimeRange,
    rule: &RecurrenceRule,
) -> Result<TimeRange, EventError> {
    let count = until_to_count(
        provided_time,
        first_entry.start,
        rule.interval,
        first_entry.duration(),
        &rule.kind,
    )?;

    let until = count_to_until(
        count,
        rule.interval,
        first_entry.start,
        &first_entry,
        &rule.kind,
    )?;
    Ok(TimeRange::new(until - first_entry.duration(), until))
}

pub fn raw_next_entry(
    provided_time: OffsetDateTime,
    first_entry: TimeRange,
    rule: &RecurrenceRule,
) -> Result<TimeRange, EventError> {
    let count = until_to_count(
        provided_time,
        first_entry.start,
        rule.interval,
        first_entry.duration(),
        &rule.kind,
    )?;

    let until = count_to_until(
        count + 1,
        rule.interval,
        first_entry.start,
        &first_entry,
        &rule.kind,
    )?;
    Ok(TimeRange::new(until - first_entry.duration(), until))
}

pub fn prev_entry(
    provided_time: OffsetDateTime,
    first_entry: TimeRange,
    rule: &RecurrenceRule,
) -> Result<Option<TimeRange>, EventError> {
    if provided_time < first_entry.start {
        return Ok(None);
    };

    let prev_entry = raw_prev_entry(provided_time, first_entry, rule)?;
    let next_entry = raw_next_entry(provided_time, first_entry, rule)?;
    let last_entry = rule
        .span
        .map(|span| TimeRange::new(span.end - first_entry.duration(), span.end));

    if let Some(entry) = last_entry {
        if prev_entry.end > entry.end {
            return Ok(Some(entry));
        }
    }

    if next_entry.start <= provided_time && provided_time < next_entry.end {
        Ok(Some(next_entry))
    } else {
        Ok(Some(prev_entry))
    }
}

pub fn next_entry(
    provided_time: OffsetDateTime,
    first_entry: TimeRange,
    rule: &RecurrenceRule,
) -> Result<Option<TimeRange>, EventError> {
    if provided_time < first_entry.start {
        return Ok(Some(first_entry));
    };

    let prev_entry = raw_prev_entry(provided_time, first_entry, rule)?;
    let next_entry = raw_next_entry(provided_time, first_entry, rule)?;
    let last_entry = rule
        .span
        .map(|span| TimeRange::new(span.end - first_entry.duration(), span.end));

    if let Some(entry) = last_entry {
        if prev_entry.end >= entry.end {
            return Ok(None);
        }
    }

    Ok(Some(next_entry))
}

#[cfg(test)]
mod entry_tests {
    use super::*;

    const TEST_RULE: RecurrenceRule = RecurrenceRule {
        span: Some(EntriesSpan {
            end: datetime!(2023-04-01 13:00:00 +0000),
            repetitions: 5,
        }),
        interval: 1,
        kind: RecurrenceRuleKind::Monthly { is_by_day: true },
    };

    const TEST_FIRST_ENTRY: TimeRange = TimeRange {
        start: datetime!(2022-12-01 12:00:00 +0000),
        end: datetime!(2022-12-01 13:00:00 +0000),
    };

    #[test]
    fn prev_entry_test_time_before_recurrence() {
        let provided_time = datetime!(2022-12-01 11:59:59 +0000);
        assert_eq!(
            prev_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            None
        );
    }

    #[test]
    fn prev_entry_test_time_on_entry() {
        let provided_time = datetime!(2023-02-01 12:00:00 +0000);
        assert_eq!(
            prev_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TimeRange {
                start: datetime!(2023-02-01 12:00:00 +0000),
                end: datetime!(2023-02-01 13:00:00 +0000),
            })
        );
    }

    #[test]
    fn prev_entry_test_time_not_on_entry() {
        let provided_time = datetime!(2023-02-28 12:00:00 +0000);
        assert_eq!(
            prev_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TimeRange {
                start: datetime!(2023-02-01 12:00:00 +0000),
                end: datetime!(2023-02-01 13:00:00 +0000),
            })
        );
    }

    #[test]
    fn prev_entry_test_time_after_recurrence() {
        let provided_time = datetime!(2023-05-01 14:00:00 +0000);
        assert_eq!(
            prev_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TimeRange {
                start: datetime!(2023-04-01 12:00:00 +0000),
                end: datetime!(2023-04-01 13:00:00 +0000),
            })
        );
    }

    #[test]
    fn next_entry_test_time_before_recurrence() {
        let provided_time = datetime!(2022-12-01 11:59:59 +0000);
        assert_eq!(
            next_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TEST_FIRST_ENTRY),
        );
    }

    #[test]
    fn next_entry_test_time_on_entry() {
        let provided_time = datetime!(2023-02-01 12:00:00 +0000);
        assert_eq!(
            next_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TimeRange {
                start: datetime!(2023-02-01 12:00:00 +0000),
                end: datetime!(2023-02-01 13:00:00 +0000),
            })
        );
    }

    #[test]
    fn next_entry_test_time_not_on_entry() {
        let provided_time = datetime!(2023-02-01 13:00:00 +0000);
        assert_eq!(
            next_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            Some(TimeRange {
                start: datetime!(2023-03-01 12:00:00 +0000),
                end: datetime!(2023-03-01 13:00:00 +0000),
            })
        );
    }

    #[test]
    fn next_entry_test_time_after_recurrence() {
        let provided_time = datetime!(2023-05-01 14:00:00 +0000);
        assert_eq!(
            next_entry(provided_time, TEST_FIRST_ENTRY, &TEST_RULE).unwrap(),
            None,
        );
    }
}
