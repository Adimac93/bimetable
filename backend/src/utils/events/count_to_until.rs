use time::{
    util::{days_in_year_month, is_leap_year, weeks_in_year},
    Duration, Month, OffsetDateTime, Weekday,
};

use crate::app_errors::DefaultContext;

use super::{
    additions::{
        add_months, get_amount_from_week_map, get_offset_from_the_map, yearly_conv_data,
        CyclicTimeTo,
    },
    calculations::CountToUntilData,
    errors::EventError,
};

pub fn day_count_to_until(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    Ok(Some(
        conv_data
            .part_starts_at
            .checked_add(Duration::days(
                (conv_data.count as i64)
                    .checked_mul(conv_data.interval as i64)
                    .dc()?,
            ))
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

pub fn week_count_to_until(
    conv_data: CountToUntilData,
    week_map: &str,
) -> Result<Option<OffsetDateTime>, EventError> {
    // get amount of event recurrences in 1 week
    let week_event_num = get_amount_from_week_map(week_map);

    // calculate the number of full week intervals
    let mut weeks_passed = (conv_data.count / week_event_num as u32)
        .checked_mul(conv_data.interval)
        .dc()?;

    // calculate the amount of days passed in the last interval
    let bonus_days_passed = get_offset_from_the_map(
        week_map,
        ((conv_data.count - 1) % week_event_num as u32) as u8 + 1,
        conv_data.part_starts_at.weekday().number_days_from_monday(),
    );

    // account for events carrying over to the next week interval
    if conv_data.part_starts_at.weekday().number_days_from_monday() + bonus_days_passed > 6 {
        weeks_passed = weeks_passed.checked_add(conv_data.interval - 1).dc()?;
    };

    Ok(Some(
        conv_data
            .part_starts_at
            .checked_add(Duration::weeks(weeks_passed as i64))
            .dc()?
            .checked_add(Duration::days(bonus_days_passed as i64))
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

pub fn month_is_by_day_count_to_until(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    if conv_data.part_starts_at.day() <= 28 {
        month_is_by_day_count_to_until_easy_days(conv_data)
    } else {
        month_is_by_day_count_to_until_hard_days(conv_data)
    }
}

fn month_is_by_day_count_to_until_easy_days(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let base_date = add_months(
        conv_data.part_starts_at,
        conv_data.count.checked_mul(conv_data.interval).dc()?,
    )?;
    Ok(Some(base_date + (conv_data.event_duration)))
}

fn month_is_by_day_count_to_until_hard_days(
    mut conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let mut monthly_step = conv_data.part_starts_at.replace_day(1).dc()?;
    while conv_data.count > 0 {
        monthly_step = add_months(monthly_step, conv_data.interval)?;
        if days_in_year_month(monthly_step.year(), monthly_step.month())
            >= conv_data.part_starts_at.day()
        {
            conv_data.count -= 1;
        }
    }
    Ok(Some(
        monthly_step
            .replace_day(conv_data.part_starts_at.day())
            .dc()?
            + (conv_data.event_duration),
    ))
}

pub fn month_count_to_until(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    if conv_data.part_starts_at.day() <= 28 {
        month_count_to_until_easy_days(conv_data)
    } else {
        month_count_to_until_hard_days(conv_data)
    }
}

fn month_count_to_until_easy_days(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let week_number = (conv_data.part_starts_at.day() - 1) / 7;

    let first_target_month_day = add_months(
        conv_data.part_starts_at,
        conv_data.count.checked_mul(conv_data.interval).dc()?,
    )?
    .replace_day(1)
    .dc()?;

    let days_passed = first_target_month_day
        .weekday()
        .cyclic_time_to(conv_data.part_starts_at.weekday());

    Ok(Some(
        first_target_month_day
            .checked_add(Duration::weeks(week_number as i64))
            .dc()?
            .checked_add(Duration::days(days_passed as i64))
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

fn month_count_to_until_hard_days(
    mut conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let mut monthly_step = conv_data.part_starts_at.replace_day(1).dc()?;
    loop {
        monthly_step = add_months(monthly_step, conv_data.interval)?;
        let target_day = monthly_step
            .weekday()
            .cyclic_time_to(conv_data.part_starts_at.weekday()) as u8
            + 29;

        if days_in_year_month(monthly_step.year(), monthly_step.month()) >= target_day {
            conv_data.count -= 1;
        }

        if conv_data.count == 0 {
            monthly_step = monthly_step.replace_day(target_day).dc()?;
            return Ok(Some(
                monthly_step.checked_add(conv_data.event_duration).dc()?,
            ));
        }
    }
}

pub fn year_is_by_day_count_to_until(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    if (
        conv_data.part_starts_at.month(),
        conv_data.part_starts_at.day(),
    ) == (Month::February, 29)
    {
        year_is_by_day_count_to_until_feb_29(conv_data)
    } else {
        year_is_by_day_count_to_until_other_days(conv_data)
    }
}

fn year_is_by_day_count_to_until_feb_29(
    mut conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let mut yearly_step = conv_data.part_starts_at.replace_day(1).dc()?;

    while conv_data.count > 0 {
        yearly_step = yearly_step
            .replace_year(
                yearly_step
                    .year()
                    .checked_add(
                        i32::try_from(conv_data.count.checked_mul(conv_data.interval).dc()?)
                            .dc()?,
                    )
                    .dc()?,
            )
            .dc()?;
        if is_leap_year(yearly_step.year()) {
            conv_data.count -= 1;
        }
    }

    Ok(Some(
        yearly_step
            .replace_day(conv_data.part_starts_at.day())
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

fn year_is_by_day_count_to_until_other_days(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let target_date = conv_data
        .part_starts_at
        .replace_year(
            conv_data
                .part_starts_at
                .year()
                .checked_add(
                    i32::try_from(conv_data.count.checked_mul(conv_data.interval).dc()?).dc()?,
                )
                .dc()?,
        )
        .dc()?;
    return Ok(Some(
        target_date.checked_add(conv_data.event_duration).dc()?,
    ));
}

pub fn year_count_to_until(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    if conv_data.part_starts_at.iso_week() == 53 {
        year_count_to_until_hard_days(conv_data)
    } else {
        year_count_to_until_easy_days(conv_data)
    }
}

fn year_count_to_until_easy_days(
    conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let (target_weekday, target_week, base_year) = yearly_conv_data(conv_data.part_starts_at)?;

    let target_year = base_year
        .replace_year(
            base_year
                .year()
                .checked_add(
                    i32::try_from(conv_data.count.checked_mul(conv_data.interval).dc()?).dc()?,
                )
                .dc()?,
        )
        .dc()?;

    let first_monday =
        target_year + Duration::days(target_year.weekday().cyclic_time_to(Weekday::Monday) as i64);

    Ok(Some(
        first_monday
            .checked_add(Duration::weeks(
                (target_week - first_monday.iso_week() + 1) as i64,
            ))
            .dc()?
            .checked_add(Duration::days(
                Weekday::Monday.cyclic_time_to(target_weekday) as i64,
            ))
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

fn year_count_to_until_hard_days(
    mut conv_data: CountToUntilData,
) -> Result<Option<OffsetDateTime>, EventError> {
    let (target_weekday, target_week, base_year) = yearly_conv_data(conv_data.part_starts_at)?;
    let mut yearly_step = base_year;

    while conv_data.count > 0 {
        yearly_step = yearly_step
            .replace_year(
                yearly_step
                    .year()
                    .checked_add(i32::try_from(conv_data.interval).dc()?)
                    .dc()?,
            )
            .dc()?;
        if weeks_in_year(yearly_step.year()) == 53 {
            conv_data.count -= 1;
        }
    }

    let first_target_year_monday =
        yearly_step + Duration::days(yearly_step.weekday().cyclic_time_to(Weekday::Monday) as i64);

    Ok(Some(
        first_target_year_monday
            .checked_add(Duration::weeks(
                ((target_week as i8) - (first_target_year_monday.iso_week() as i8) + 1) as i64,
            ))
            .dc()?
            .checked_add(Duration::days(
                Weekday::Monday.cyclic_time_to(target_weekday) as i64,
            ))
            .dc()?
            .checked_add(conv_data.event_duration)
            .dc()?,
    ))
}

#[cfg(test)]
mod recurrence_tests {
    use super::*;
    use crate::utils::events::models::{Event, EventPart, EventRules, RecurrenceEndsAt, TimeRules};
    use sqlx::types::Json;
    use time::macros::datetime;
    use uuid::Uuid;

    fn create_test_event_part(
        event_starts_at: OffsetDateTime,
        event_ends_at: OffsetDateTime,
        recurrence_rule: EventRules,
        part_starts_at: OffsetDateTime,
        part_length: RecurrenceEndsAt,
    ) -> EventPart {
        EventPart {
            event_data: Event {
                id: Uuid::new_v4(),
                owner_id: Uuid::new_v4(),
                name: "Test event".into(),
                starts_at: Some(event_starts_at),
                ends_at: Some(event_ends_at),
                recurrence_rule: Some(Json(recurrence_rule)),
                description: "Test description".into(),
            },
            part_starts_at,
            part_length: Some(part_length),
        }
    }

    #[test]
    fn daily_recurrence_test() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Daily {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(15)),
                    interval: 3,
                },
            },
            datetime!(2023-02-21 10:00 +1),
            RecurrenceEndsAt::Count(7),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-03-14 12:15 +1))
        )
    }

    #[test]
    fn weekly_recurrence_test() {
        let data = create_test_event_part(
            datetime!(2023-02-15 10:00 +1),
            datetime!(2023-02-15 12:15 +1),
            EventRules::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(30)),
                    interval: 2,
                },
                week_map: 86,
            },
            datetime!(2023-02-27 10:00 +1),
            RecurrenceEndsAt::Count(5),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-03-15 12:15 +1))
        )
    }

    #[test]
    fn weekly_recurrence_test_next_week_offset() {
        let data = create_test_event_part(
            datetime!(2023-02-15 10:00 +1),
            datetime!(2023-02-15 12:15 +1),
            EventRules::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(30)),
                    interval: 2,
                },
                week_map: 86,
            },
            datetime!(2023-03-01 10:00 +1),
            RecurrenceEndsAt::Count(7),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-03-27 12:15 +1))
        )
    }

    #[test]
    fn monthly_recurrence_test_by_day() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 2,
                },
                is_by_day: true,
            },
            datetime!(2023-04-18 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-08-18 12:15 +1))
        )
    }

    #[test]
    fn monthly_recurrence_test_by_day_month_end() {
        let data = create_test_event_part(
            datetime!(2025-01-29 10:00 +1),
            datetime!(2025-01-29 12:15 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(30)),
                    interval: 5,
                },
                is_by_day: true,
            },
            datetime!(2025-11-29 10:00 +1),
            RecurrenceEndsAt::Count(15),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2032-07-29 12:15 +1))
        )
    }

    #[test]
    fn monthly_recurrence_test_by_weekday() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 2,
                },
                is_by_day: false,
            },
            datetime!(2023-04-15 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-08-19 12:15 +1))
        )
    }

    #[test]
    fn monthly_recurrence_test_by_weekday_month_end() {
        let data = create_test_event_part(
            datetime!(2023-01-31 10:00 +1),
            datetime!(2023-01-31 12:15 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2023-05-30 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2023-10-31 12:15 +1))
        )
    }

    #[test]
    fn yearly_recurrence_test_by_day() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 2,
                },
                is_by_day: true,
            },
            datetime!(2025-02-18 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2029-02-18 12:15 +1))
        )
    }

    #[test]
    fn yearly_recurrence_test_by_day_feb_29() {
        let data = create_test_event_part(
            datetime!(2024-02-29 10:00 +1),
            datetime!(2024-02-29 12:15 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 1,
                },
                is_by_day: true,
            },
            datetime!(2028-02-29 10:00 +1),
            RecurrenceEndsAt::Count(1),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2032-02-29 12:15 +1))
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 2,
                },
                is_by_day: false,
            },
            datetime!(2025-02-15 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2029-02-17 12:15 +1))
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_52nd_week() {
        let data = create_test_event_part(
            datetime!(2020-12-26 10:00 +1),
            datetime!(2020-12-26 12:15 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2022-01-01 10:00 +1),
            RecurrenceEndsAt::Count(1),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2022-12-31 12:15 +1))
        )
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_53rd_week() {
        let data = create_test_event_part(
            datetime!(2020-12-30 10:00 +1),
            datetime!(2020-12-30 12:15 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(10)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2026-12-31 10:00 +1),
            RecurrenceEndsAt::Count(1),
        );

        assert_eq!(
            data.count_to_until().unwrap(),
            Some(datetime!(2032-12-30 12:15 +1))
        )
    }
}
