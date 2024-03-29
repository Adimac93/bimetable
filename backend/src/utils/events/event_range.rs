use std::cmp::max;

use time::{ext::NumericalDuration, util::weeks_in_year, OffsetDateTime, Weekday};

use crate::app_errors::DefaultContext;

use super::{
    additions::{
        iso_year_start, max_date_time, next_good_month, next_good_month_by_weekday, AddTime,
        CyclicTimeTo, TimeStart, TimeTo,
    },
    errors::EventError,
    models::TimeRange,
};

pub struct EventRangeData {
    pub range: TimeRange,
    pub event_range: TimeRange,
    pub rec_ends_at: Option<OffsetDateTime>,
    pub interval: u32,
}

pub fn get_daily_events(range_data: EventRangeData) -> Result<Vec<TimeRange>, EventError> {
    let day_amount = (range_data.range.start - range_data.event_range.end).whole_days();
    let offset_from_origin_event = max(
        day_amount - day_amount.rem_euclid(range_data.interval as i64),
        0,
    )
    .days();

    let mut daily_event = range_data
        .event_range
        .checked_add(offset_from_origin_event)
        .dc()?;
    let mut res = Vec::new();

    while !daily_event.is_after(&range_data.range)
        && daily_event.start < range_data.rec_ends_at.unwrap_or(max_date_time())
    {
        if daily_event.is_overlapping(&range_data.range) {
            res.push(daily_event);
        }

        daily_event = daily_event
            .checked_add((range_data.interval as i64).days())
            .dc()?;
    }

    Ok(res)
}

pub fn get_weekly_events(
    range_data: EventRangeData,
    week_map: &str,
) -> Result<Vec<TimeRange>, EventError> {
    let week_amount = (range_data.range.start - range_data.event_range.end).whole_weeks();
    let offset_from_origin_event = max(
        week_amount - week_amount.rem_euclid(range_data.interval as i64),
        0,
    )
    .weeks();
    let offset_from_week_start =
        (Weekday::Monday.cyclic_time_to(range_data.event_range.start.weekday()) as i64).days();

    let mut res = Vec::new();

    let weekly_event_start = range_data
        .event_range
        .start
        .checked_add(offset_from_origin_event)
        .dc()?
        .checked_sub(offset_from_week_start)
        .dc()?;
    let mut weekly_event =
        TimeRange::new_relative_checked(weekly_event_start, range_data.event_range.duration())
            .dc()?;

    while !weekly_event.is_after(&range_data.range)
        && weekly_event.start < range_data.rec_ends_at.unwrap_or(max_date_time())
    {
        for (i, elem) in week_map.chars().enumerate() {
            let target_range = weekly_event.checked_add((i as i64).days()).dc()?;
            if elem == '1'
                && target_range.is_overlapping(&range_data.range)
                && target_range.start < range_data.rec_ends_at.unwrap_or(max_date_time())
            {
                res.push(target_range);
            }
        }

        weekly_event = weekly_event
            .checked_add((range_data.interval as i64).weeks())
            .dc()?;
    }

    Ok(res)
}

pub fn get_monthly_events_by_day(
    range_data: EventRangeData,
    is_by_day: bool,
) -> Result<Vec<TimeRange>, EventError> {
    let (event_end_year, event_end_month, _) = range_data.event_range.end.to_calendar_date();
    let (range_start_year, range_start_month, _) = range_data.range.start.to_calendar_date();

    let month_amount =
        (event_end_year, event_end_month).time_to((range_start_year, range_start_month));

    let offset_from_origin_event = max(
        month_amount - month_amount.rem_euclid(range_data.interval as i32),
        0,
    );

    let month_start = range_data
        .event_range
        .start
        .month_start()
        .add_months(offset_from_origin_event as i64)
        .dc()?;
    let mut monthly_step = range_data.event_range.start;

    while monthly_step < month_start {
        if is_by_day {
            monthly_step = next_good_month(monthly_step, range_data.interval as i64)?;
        } else {
            monthly_step = next_good_month_by_weekday(monthly_step, range_data.interval as i64)?;
        }
    }

    let mut res = Vec::new();

    while monthly_step < range_data.range.end
        && monthly_step < range_data.rec_ends_at.unwrap_or(max_date_time())
    {
        let monthly_event =
            TimeRange::new_relative_checked(monthly_step, range_data.event_range.duration())
                .dc()?;
        if monthly_event.is_overlapping(&range_data.range) {
            res.push(monthly_event);
        };

        if is_by_day {
            monthly_step = next_good_month(monthly_step, range_data.interval as i64)?;
        } else {
            monthly_step = next_good_month_by_weekday(monthly_step, range_data.interval as i64)?;
        }
    }

    Ok(res)
}

pub fn get_yearly_events_by_weekday(
    range_data: EventRangeData,
) -> Result<Vec<TimeRange>, EventError> {
    let (range_base_year, ..) = range_data.range.start.to_iso_week_date();
    let (event_base_year, target_week_number, target_weekday) =
        range_data.event_range.start.to_iso_week_date();
    let year_amount = range_base_year - event_base_year;
    let offset_from_origin_event = max(
        year_amount - year_amount.rem_euclid(range_data.interval as i32),
        0,
    );

    let mut yearly_step = event_base_year.checked_add(offset_from_origin_event).dc()?;
    let offset_from_iso_year_start = (target_week_number as i64 - 1).weeks()
        + (Weekday::Monday.cyclic_time_to(target_weekday) as i64).days();
    let mut res = Vec::new();

    while iso_year_start(yearly_step) < range_data.range.end
        && iso_year_start(yearly_step) < range_data.rec_ends_at.unwrap_or(max_date_time())
    {
        if weeks_in_year(yearly_step) >= target_week_number {
            let target_day = iso_year_start(yearly_step)
                .checked_add(offset_from_iso_year_start)
                .dc()?;

            res.push(
                TimeRange::new_relative_checked(
                    target_day.replace_time(range_data.event_range.start.time()),
                    range_data.event_range.duration(),
                )
                .dc()?,
            );
        };

        yearly_step = yearly_step.checked_add(range_data.interval as i32).dc()?;
    }

    Ok(res)
}

// Since entry count is not relevant for calculations, the entry amount is arbitrary.
#[cfg(test)]
mod event_range_tests {
    use time::macros::datetime;

    use crate::utils::events::models::{EntriesSpan, RecurrenceRule, RecurrenceRuleKind};

    use super::*;

    #[test]
    fn daily_range() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Daily,
        };
        let part = TimeRange {
            start: datetime!(2023-02-21 0:00 UTC),
            end: datetime!(2023-02-27 22:45 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-02-21 22:45 UTC),
                    datetime!(2023-02-22 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-02-23 22:45 UTC),
                    datetime!(2023-02-24 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-02-25 22:45 UTC),
                    datetime!(2023-02-26 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn weekly_range_1() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Weekly { week_map: 54 },
        };
        let part = TimeRange {
            start: datetime!(2023-02-21 0:00 UTC),
            end: datetime!(2023-03-15 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-02-28 22:45 UTC),
                    datetime!(2023-03-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-01 22:45 UTC),
                    datetime!(2023-03-02 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-03 22:45 UTC),
                    datetime!(2023-03-04 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-04 22:45 UTC),
                    datetime!(2023-03-05 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-14 22:45 UTC),
                    datetime!(2023-03-15 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn weekly_range_2() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Weekly { week_map: 54 },
        };
        let part = TimeRange {
            start: datetime!(2023-03-01 0:00 UTC),
            end: datetime!(2023-03-22 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-03-01 22:45 UTC),
                    datetime!(2023-03-02 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-03 22:45 UTC),
                    datetime!(2023-03-04 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-04 22:45 UTC),
                    datetime!(2023-03-05 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-14 22:45 UTC),
                    datetime!(2023-03-15 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-15 22:45 UTC),
                    datetime!(2023-03-16 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-17 22:45 UTC),
                    datetime!(2023-03-18 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-18 22:45 UTC),
                    datetime!(2023-03-19 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn weekly_range_3() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2023-03-16 0:00 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Weekly { week_map: 54 },
        };
        let part = TimeRange {
            start: datetime!(2023-02-21 0:00 UTC),
            end: datetime!(2023-03-20 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-02-28 22:45 UTC),
                    datetime!(2023-03-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-01 22:45 UTC),
                    datetime!(2023-03-02 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-03 22:45 UTC),
                    datetime!(2023-03-04 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-04 22:45 UTC),
                    datetime!(2023-03-05 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-14 22:45 UTC),
                    datetime!(2023-03-15 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-15 22:45 UTC),
                    datetime!(2023-03-16 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_1() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };
        let part = TimeRange {
            start: datetime!(2023-03-01 0:00 UTC),
            end: datetime!(2023-08-17 22:45 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-17 22:45 UTC),
                    datetime!(2023-04-18 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-06-17 22:45 UTC),
                    datetime!(2023-06-18 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_2() {
        let event = TimeRange::new(
            datetime!(2023-01-31 22:45 UTC),
            datetime!(2023-02-01 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 1,
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2024-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-31 22:45 UTC),
                    datetime!(2023-02-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-31 22:45 UTC),
                    datetime!(2023-04-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-05-31 22:45 UTC),
                    datetime!(2023-06-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-07-31 22:45 UTC),
                    datetime!(2023-08-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-08-31 22:45 UTC),
                    datetime!(2023-09-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-10-31 22:45 UTC),
                    datetime!(2023-11-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-12-31 22:45 UTC),
                    datetime!(2024-01-01 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_3() {
        let event = TimeRange::new(
            datetime!(2023-01-31 22:45 UTC),
            datetime!(2023-02-01 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2023-06-01 0:00 UTC),
                repetitions: 50,
            }),
            interval: 1,
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2024-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-31 22:45 UTC),
                    datetime!(2023-02-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-03-31 22:45 UTC),
                    datetime!(2023-04-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-05-31 22:45 UTC),
                    datetime!(2023-06-01 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_1() {
        let event = TimeRange::new(
            datetime!(2023-02-17 22:45 UTC),
            datetime!(2023-02-18 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2023-02-28 0:00 UTC),
            end: datetime!(2023-12-19 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-21 22:45 UTC),
                    datetime!(2023-04-22 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-06-16 22:45 UTC),
                    datetime!(2023-06-17 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-08-18 22:45 UTC),
                    datetime!(2023-08-19 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-10-20 22:45 UTC),
                    datetime!(2023-10-21 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-12-15 22:45 UTC),
                    datetime!(2023-12-16 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_2() {
        let event = TimeRange::new(
            datetime!(2023-01-29 22:45 UTC),
            datetime!(2023-01-30 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 1,
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2023-02-01 0:00 UTC),
            end: datetime!(2024-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-30 22:45 UTC),
                    datetime!(2023-05-01 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-07-30 22:45 UTC),
                    datetime!(2023-07-31 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-10-29 22:45 UTC),
                    datetime!(2023-10-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2023-12-31 22:45 UTC),
                    datetime!(2024-01-01 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_day_1() {
        let event = TimeRange::new(
            datetime!(2023-01-29 22:45 UTC),
            datetime!(2023-01-30 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2029-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-29 22:45 UTC),
                    datetime!(2023-01-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2025-01-29 22:45 UTC),
                    datetime!(2025-01-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2027-01-29 22:45 UTC),
                    datetime!(2027-01-30 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_day_2() {
        let event = TimeRange::new(
            datetime!(2023-01-29 22:45 UTC),
            datetime!(2023-01-30 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2025-01-30 0:00 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2029-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-29 22:45 UTC),
                    datetime!(2023-01-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2025-01-29 22:45 UTC),
                    datetime!(2025-01-30 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_1() {
        let event = TimeRange::new(
            datetime!(2023-01-29 22:45 UTC),
            datetime!(2023-01-30 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2029-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-29 22:45 UTC),
                    datetime!(2023-01-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2025-01-26 22:45 UTC),
                    datetime!(2025-01-27 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2027-01-31 22:45 UTC),
                    datetime!(2027-02-01 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_2() {
        let event = TimeRange::new(
            datetime!(2020-12-28 22:45 UTC),
            datetime!(2020-12-29 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 1,
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2029-01-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![TimeRange::new(
                datetime!(2026-12-28 22:45 UTC),
                datetime!(2026-12-29 0:00 UTC)
            ),]
        )
    }

    #[test]
    fn yearly_range_by_weekday_3() {
        let event = TimeRange::new(
            datetime!(2023-01-02 22:45 UTC),
            datetime!(2023-01-03 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 1,
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2023-01-01 0:00 UTC),
            end: datetime!(2027-02-01 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-02 22:45 UTC),
                    datetime!(2023-01-03 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2024-01-01 22:45 UTC),
                    datetime!(2024-01-02 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2024-12-30 22:45 UTC),
                    datetime!(2024-12-31 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2025-12-29 22:45 UTC),
                    datetime!(2025-12-30 0:00 UTC)
                ),
                TimeRange::new(
                    datetime!(2027-01-04 22:45 UTC),
                    datetime!(2027-01-05 0:00 UTC)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_4() {
        let event = TimeRange::new(
            datetime!(2023-01-14 22:45 UTC),
            datetime!(2023-01-15 0:00 UTC),
        );
        let rec_rules = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2100-12-31 23:59:59 UTC),
                repetitions: 50,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        let part = TimeRange {
            start: datetime!(2027-01-16 0:00 UTC),
            end: datetime!(2027-01-17 0:00 UTC),
        };

        assert_eq!(
            rec_rules.get_event_range(part, event).unwrap(),
            vec![TimeRange::new(
                datetime!(2027-01-16 22:45 UTC),
                datetime!(2027-01-17 0:00 UTC)
            ),]
        )
    }

    #[test]
    fn adimac93_test_1() {
        // search starts before the event starts, and event recurrence ends based on count
        let rule = RecurrenceRule {
            span: Some(EntriesSpan {
                end: datetime!(2023-03-05 17:06:43.941 +00:00:00),
                repetitions: 2,
            }),
            interval: 2,
            kind: RecurrenceRuleKind::Daily,
        };

        let ranges = rule.get_event_range(
            TimeRange::new(
                datetime!(2020-03-01 15:00:11.469 +00:00:00),
                datetime!(2024-03-01 15:00:11.469 +00:00:00),
            ),
            TimeRange::new(
                datetime!(2023-03-01 16:06:43.941 +00:00:00),
                datetime!(2023-03-01 17:06:43.941 +00:00:00),
            ),
        );

        assert_eq!(
            ranges.unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-03-01 16:06:43.941 +00:00:00),
                    datetime!(2023-03-01 17:06:43.941 +00:00:00),
                ),
                TimeRange::new(
                    datetime!(2023-03-03 16:06:43.941 +00:00:00),
                    datetime!(2023-03-03 17:06:43.941 +00:00:00),
                ),
                TimeRange::new(
                    datetime!(2023-03-05 16:06:43.941 +00:00:00),
                    datetime!(2023-03-05 17:06:43.941 +00:00:00),
                ),
            ]
        );
    }
}
