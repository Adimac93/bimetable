use crate::utils::events::count_to_until::count_to_until;
use crate::utils::events::errors::EventError;
use crate::utils::events::models::{RecurrenceRule, RecurrenceRuleKind, TimeRange};
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
