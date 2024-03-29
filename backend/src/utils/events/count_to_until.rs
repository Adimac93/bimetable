use time::{Date, Duration, Month, OffsetDateTime};

use crate::app_errors::DefaultContext;
use crate::utils::events::models::{RecurrenceRuleKind, TimeRange};
use crate::validation::{ValidateContent, ValidateContentError};

use super::{
    additions::{
        get_amount_from_week_map, get_offset_from_the_map,
        next_good_month_by_weekday, nth_53_week_year_by_weekday, nth_good_month, AddTime,
        CyclicTimeTo,
    },
    errors::EventError,
};

pub struct CountToUntilData {
    pub part_starts_at: OffsetDateTime,
    pub count: u32,
    pub interval: u32,
    pub event_duration: Duration,
}

impl ValidateContent for CountToUntilData {
    fn validate_content(&self) -> Result<(), ValidateContentError> {
        if self.event_duration < Duration::ZERO {
            return Err(ValidateContentError::Expected(
                "Event starts after it ends".to_string(),
            ));
        }
        if self.interval == 0 {
            return Err(ValidateContentError::Expected(
                "Interval is equal to 0".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn count_to_until(
    count: u32,
    interval: u32,
    start: OffsetDateTime,
    event: &TimeRange,
    kind: &RecurrenceRuleKind,
) -> Result<OffsetDateTime, EventError> {
    let conv_data = CountToUntilData {
        part_starts_at: start,
        count,
        interval,
        event_duration: event.duration(),
    };

    conv_data.validate_content()?;

    match kind {
        RecurrenceRuleKind::Yearly { is_by_day: true } => yearly_c_to_u_by_day(conv_data),
        RecurrenceRuleKind::Yearly { is_by_day: false } => yearly_c_to_u_by_weekday(conv_data),
        RecurrenceRuleKind::Monthly { is_by_day: true } => monthly_c_to_u_by_day(conv_data),
        RecurrenceRuleKind::Monthly { is_by_day: false } => monthly_c_to_u_by_weekday(conv_data),
        RecurrenceRuleKind::Weekly { week_map } => {
            let string_week_map = format!("{:0>7b}", week_map % 128);
            weekly_c_to_u(conv_data, &string_week_map)
        }
        RecurrenceRuleKind::Daily => daily_c_to_u(conv_data),
    }
}

pub fn daily_c_to_u(conv_data: CountToUntilData) -> Result<OffsetDateTime, EventError> {
    Ok(conv_data
        .part_starts_at
        .add_days(conv_data.count.checked_mul(conv_data.interval).dc()? as i64)?
        .checked_add(conv_data.event_duration)
        .dc()?)
}

pub fn weekly_c_to_u(
    conv_data: CountToUntilData,
    week_map: &str,
) -> Result<OffsetDateTime, EventError> {
    // get amount of event recurrences in 1 week
    let week_event_num = get_amount_from_week_map(week_map);

    // calculate the number of full week intervals
    let mut weeks_passed = (conv_data.count / week_event_num as u32)
        .checked_mul(conv_data.interval)
        .dc()?;

    // calculate the amount of days passed in the last interval
    let bonus_days_passed = get_offset_from_the_map(
        week_map,
        (conv_data.count % week_event_num as u32) as u8 + 1,
        conv_data.part_starts_at.weekday().number_days_from_monday(),
    );

    // account for events carrying over to the next week interval
    if conv_data.part_starts_at.weekday().number_days_from_monday() + bonus_days_passed > 6 {
        weeks_passed = weeks_passed.checked_add(conv_data.interval - 1).dc()?;
    };

    Ok(conv_data
        .part_starts_at
        .add_weeks(weeks_passed as i64)?
        .add_days(bonus_days_passed as i64)?
        .checked_add(conv_data.event_duration)
        .dc()?)
}

pub fn monthly_c_to_u_by_day(conv_data: CountToUntilData) -> Result<OffsetDateTime, EventError> {
    let base_date = conv_data.part_starts_at;

    let target_date = if conv_data.part_starts_at.day() <= 28 {
        base_date.add_months(conv_data.count.checked_mul(conv_data.interval).dc()? as i64)?
    } else {
        nth_good_month(base_date, conv_data.count, conv_data.interval as i64)?
    };

    Ok(target_date.checked_add(conv_data.event_duration).dc()?)
}

pub fn monthly_c_to_u_by_weekday(
    conv_data: CountToUntilData,
) -> Result<OffsetDateTime, EventError> {
    if conv_data.part_starts_at.day() <= 28 {
        monthly_c_to_u_for_other_days(conv_data)
    } else {
        monthly_c_to_u_for_last_days(conv_data)
    }
}

fn monthly_c_to_u_for_other_days(
    conv_data: CountToUntilData,
) -> Result<OffsetDateTime, EventError> {
    let week_number = (conv_data.part_starts_at.day() - 1) / 7;

    let first_target_month_day = conv_data
        .part_starts_at
        .add_months(conv_data.count.checked_mul(conv_data.interval).dc()? as i64)?
        .replace_day(1)
        .dc()?;

    let days_passed = first_target_month_day
        .weekday()
        .cyclic_time_to(conv_data.part_starts_at.weekday());

    Ok(first_target_month_day
        .add_weeks(week_number as i64)?
        .add_days(days_passed as i64)?
        .checked_add(conv_data.event_duration)
        .dc()?)
}

fn monthly_c_to_u_for_last_days(
    mut conv_data: CountToUntilData,
) -> Result<OffsetDateTime, EventError> {
    let mut monthly_step = conv_data.part_starts_at;
    while conv_data.count != 0 {
        monthly_step = next_good_month_by_weekday(monthly_step, conv_data.interval as i64)?;
        conv_data.count -= 1;
    }

    Ok(monthly_step.checked_add(conv_data.event_duration).dc()?)
}

pub fn yearly_c_to_u_by_day(conv_data: CountToUntilData) -> Result<OffsetDateTime, EventError> {
    let base_date = conv_data.part_starts_at;

    let target_date = if (
        conv_data.part_starts_at.month(),
        conv_data.part_starts_at.day(),
    ) == (Month::February, 29)
    {
        nth_good_month(base_date, conv_data.count, conv_data.interval as i64 * 12)?
    } else {
        base_date.add_years(conv_data.count.checked_mul(conv_data.interval).dc()? as i64)?
    };

    Ok(target_date.checked_add(conv_data.event_duration).dc()?)
}

pub fn yearly_c_to_u_by_weekday(conv_data: CountToUntilData) -> Result<OffsetDateTime, EventError> {
    let (base_year, target_week, target_weekday) = conv_data.part_starts_at.to_iso_week_date();

    let target_date = if conv_data.part_starts_at.iso_week() == 53 {
        nth_53_week_year_by_weekday(
            conv_data.part_starts_at,
            conv_data.count,
            conv_data.interval,
        )?
    } else {
        conv_data.part_starts_at.replace_date(
            Date::from_iso_week_date(
                base_year
                    .checked_add(
                        i32::try_from(conv_data.count.checked_mul(conv_data.interval).dc()?)
                            .dc()?,
                    )
                    .dc()?,
                target_week,
                target_weekday,
            )
            .dc()?,
        )
    };

    Ok(target_date.checked_add(conv_data.event_duration).dc()?)
}

#[cfg(test)]
mod recurrence_tests {
    use crate::routes::events::models::{RecurrenceEndsAt, RecurrenceRuleSchema, TimeRules};
    use crate::utils::events::models::{RecurrenceRuleKind, TimeRange};
    use time::macros::datetime;

    #[test]
    fn daily_recurrence_test() {
        let event = TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(15)),
                interval: 3,
            },
            kind: RecurrenceRuleKind::Daily,
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-02-21 10:00 UTC), 7, &event)
                .unwrap(),
            datetime!(2023-03-14 12:15 UTC)
        )
    }

    #[test]
    fn weekly_recurrence_test() {
        let event = TimeRange::new(
            datetime!(2023-02-15 10:00 UTC),
            datetime!(2023-02-15 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(30)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 86 },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-02-27 10:00 UTC), 5, &event)
                .unwrap(),
            datetime!(2023-03-15 12:15 UTC)
        )
    }

    #[test]
    fn weekly_recurrence_test_next_week_offset() {
        let event = TimeRange::new(
            datetime!(2023-02-15 10:00 UTC),
            datetime!(2023-02-15 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(30)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 86 },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-03-01 10:00 UTC), 7, &event)
                .unwrap(),
            datetime!(2023-03-27 12:15 UTC)
        )
    }

    #[test]
    fn bonus_weekly_recurrence_test() {
        let event = TimeRange::new(
            datetime!(2023-03-08 10:00 UTC),
            datetime!(2023-03-08 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 24 },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-03-08 10:00 UTC), 10, &event)
                .unwrap(),
            datetime!(2023-04-12 12:15 UTC)
        )
    }

    #[test]
    fn monthly_recurrence_test_by_day() {
        let event = TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-04-18 10:00 UTC), 2, &event)
                .unwrap(),
            datetime!(2023-08-18 12:15 UTC)
        )
    }

    #[test]
    fn monthly_recurrence_test_by_day_month_end() {
        let event = TimeRange::new(
            datetime!(2025-01-29 10:00 UTC),
            datetime!(2025-01-29 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(30)),
                interval: 5,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2025-11-29 10:00 UTC), 15, &event)
                .unwrap(),
            datetime!(2032-07-29 12:15 UTC)
        )
    }

    #[test]
    fn monthly_recurrence_test_by_weekday() {
        let event = TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-04-15 10:00 UTC), 2, &event)
                .unwrap(),
            datetime!(2023-08-19 12:15 UTC)
        )
    }

    #[test]
    fn monthly_recurrence_test_by_weekday_month_end() {
        let event = TimeRange::new(
            datetime!(2023-01-31 10:00 UTC),
            datetime!(2023-01-31 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2023-05-30 10:00 UTC), 2, &event)
                .unwrap(),
            datetime!(2023-10-31 12:15 UTC)
        )
    }

    #[test]
    fn yearly_recurrence_test_by_day() {
        let event = TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2025-02-18 10:00 UTC), 2, &event)
                .unwrap(),
            datetime!(2029-02-18 12:15 UTC)
        )
    }

    #[test]
    fn yearly_recurrence_test_by_day_feb_29() {
        let event = TimeRange::new(
            datetime!(2024-02-29 10:00 UTC),
            datetime!(2024-02-29 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2028-02-29 10:00 UTC), 1, &event)
                .unwrap(),
            datetime!(2032-02-29 12:15 UTC)
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday() {
        let event = TimeRange::new(
            datetime!(2023-02-18 10:00 UTC),
            datetime!(2023-02-18 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2025-02-15 10:00 UTC), 2, &event)
                .unwrap(),
            datetime!(2029-02-17 12:15 UTC)
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_52nd_week() {
        let event = TimeRange::new(
            datetime!(2020-12-26 10:00 UTC),
            datetime!(2020-12-26 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2022-01-01 10:00 UTC), 1, &event)
                .unwrap(),
            datetime!(2022-12-31 12:15 UTC)
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_53rd_week() {
        let event = TimeRange::new(
            datetime!(2020-12-30 10:00 UTC),
            datetime!(2020-12-30 12:15 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(10)),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };

        assert_eq!(
            rec_rules
                .count_to_until(datetime!(2026-12-31 10:00 UTC), 1, &event)
                .unwrap(),
            datetime!(2032-12-30 12:15 UTC)
        )
    }
}
