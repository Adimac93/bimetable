use crate::app_errors::DefaultContext;
use crate::utils::events::additions::{
    day_from_week_and_weekday, get_amount_from_week_map, get_char, next_good_month,
    next_good_month_by_weekday, nth_53_week_year_by_weekday, TimeStart, TimeTo,
};
use crate::utils::events::errors::EventError;
use crate::utils::events::models::{RecurrenceRuleKind, TimeRange};
use crate::validation::{ValidateContent, ValidateContentError};
use time::{Date, Duration, Month, OffsetDateTime};

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

pub fn until_to_count(
    until: OffsetDateTime,
    start: OffsetDateTime,
    interval: u32,
    event_duration: Duration,
    kind: &RecurrenceRuleKind,
) -> Result<u32, EventError> {
    let conv_data = UntilToCountData {
        part_starts_at: start,
        until: until - event_duration,
        interval,
    };

    conv_data.validate_content()?;

    match kind {
        RecurrenceRuleKind::Yearly { is_by_day: true } => yearly_u_to_c_by_day(conv_data),
        RecurrenceRuleKind::Yearly { is_by_day: false } => yearly_u_to_c_by_weekday(conv_data),
        RecurrenceRuleKind::Monthly { is_by_day: true } => monthly_u_to_c_by_day(conv_data),
        RecurrenceRuleKind::Monthly { is_by_day: false } => monthly_u_to_c_by_weekday(conv_data),
        RecurrenceRuleKind::Weekly { week_map } => {
            let string_week_map = format!("{:0>7b}", week_map % 128);
            weekly_u_to_c(conv_data, &string_week_map)
        }
        RecurrenceRuleKind::Daily => daily_u_to_c(conv_data),
    }
}

pub fn daily_u_to_c(data: UntilToCountData) -> Result<u32, EventError> {
    Ok(((data.until - data.part_starts_at) / data.interval).whole_days() as u32)
}

pub fn weekly_u_to_c(data: UntilToCountData, week_map: &str) -> Result<u32, EventError> {
    let events_per_week = get_amount_from_week_map(week_map);
    let week_distance = (data.until.week_start() - data.part_starts_at.week_start()).whole_weeks();

    let starting_week_amount = get_amount_from_week_map(
        &week_map[data.part_starts_at.weekday().number_days_from_monday() as usize..],
    ) as u32;

    let ending_week_completion = get_amount_from_week_map(
        &week_map[data.until.weekday().number_days_from_monday() as usize..],
    ) as u32;

    let base_res =
        week_distance as u32 / data.interval * events_per_week as u32 + starting_week_amount - 1;

    if week_distance % data.interval as i64 != 0 {
        return Ok(base_res);
    };

    if get_char(
        week_map,
        data.until.weekday().number_days_from_monday() as usize,
    ) == '1'
        && data.part_starts_at.time() <= data.until.time()
    {
        Ok(base_res + 1 - ending_week_completion)
    } else {
        Ok(base_res - ending_week_completion)
    }
}

pub fn monthly_u_to_c_by_day(data: UntilToCountData) -> Result<u32, EventError> {
    if data.part_starts_at.day() <= 28 {
        let month_distance = (data.part_starts_at.year(), data.part_starts_at.month())
            .time_to((data.until.year(), data.until.month())) as u32;

        if data.part_starts_at.replace_date(
            Date::from_calendar_date(
                data.until.year(),
                data.until.month(),
                data.part_starts_at.day(),
            )
            .dc()?,
        ) > data.until
            && month_distance % data.interval == 0
        {
            Ok(month_distance / data.interval - 1)
        } else {
            Ok(month_distance / data.interval)
        }
    } else {
        let mut monthly_step = data.part_starts_at;
        let mut res = 0;
        while monthly_step <= data.until {
            monthly_step = next_good_month(monthly_step, data.interval as i64)?;
            res += 1;
        }
        Ok(res - 1)
    }
}

pub fn monthly_u_to_c_by_weekday(data: UntilToCountData) -> Result<u32, EventError> {
    if data.part_starts_at.day() <= 28 {
        let month_distance = (data.part_starts_at.year(), data.part_starts_at.month())
            .time_to((data.until.year(), data.until.month())) as u32;
        let target_day = day_from_week_and_weekday(
            data.until,
            (data.part_starts_at.day() - 1) / 7,
            data.part_starts_at.weekday(),
        );

        if data.part_starts_at.replace_date(
            Date::from_calendar_date(data.until.year(), data.until.month(), target_day).dc()?,
        ) > data.until
            && month_distance % data.interval == 0
        {
            Ok(month_distance / data.interval - 1)
        } else {
            Ok(month_distance / data.interval)
        }
    } else {
        let mut monthly_step = data.part_starts_at;
        let mut res = 0;
        while monthly_step <= data.until {
            monthly_step = next_good_month_by_weekday(monthly_step, data.interval as i64)?;
            res += 1;
        }
        Ok(res - 1)
    }
}

pub fn yearly_u_to_c_by_day(data: UntilToCountData) -> Result<u32, EventError> {
    if let (Month::February, 29) = (data.part_starts_at.month(), data.part_starts_at.day()) {
        let mut yearly_step = data.part_starts_at;
        let mut res = 0;
        while yearly_step <= data.until {
            yearly_step = next_good_month(yearly_step, data.interval as i64 * 12)?;
            res += 1;
        }
        Ok(res - 1)
    } else {
        let year_distance = data.until.year() as u32 - data.part_starts_at.year() as u32;

        if data.part_starts_at.replace_year(data.until.year()).unwrap() > data.until
            && year_distance % data.interval == 0
        {
            Ok(year_distance / data.interval - 1)
        } else {
            Ok(year_distance / data.interval)
        }
    }
}

pub fn yearly_u_to_c_by_weekday(data: UntilToCountData) -> Result<u32, EventError> {
    let (start_year, start_week, start_weekday) = data.part_starts_at.to_iso_week_date();
    let (end_year, _end_week, _end_weekday) = data.until.to_iso_week_date();
    if start_week == 53 {
        let mut yearly_step = data.part_starts_at;
        let mut res = 0;
        while yearly_step <= data.until {
            yearly_step = nth_53_week_year_by_weekday(yearly_step, 1, data.interval)?;
            res += 1;
        }
        Ok(res - 1)
    } else {
        let year_distance = end_year as u32 - start_year as u32;

        if data
            .part_starts_at
            .replace_date(Date::from_iso_week_date(end_year, start_week, start_weekday).unwrap())
            > data.until
            && year_distance % data.interval == 0
        {
            Ok(year_distance / data.interval - 1)
        } else {
            Ok(year_distance / data.interval)
        }
    }
}

#[cfg(test)]
mod until_to_count_tests {
    use crate::routes::events::models::{RecurrenceEndsAt, RecurrenceRuleSchema, TimeRules};
    use crate::utils::events::models::{RecurrenceRuleKind, TimeRange};
    use time::macros::datetime;

    #[test]
    fn daily_until_to_count_test_1() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-30 20:00 UTC))),
                interval: 3,
            },
            kind: RecurrenceRuleKind::Daily,
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2023-03-30 20:00 UTC),
                    &event
                )
                .unwrap(),
            3
        )
    }

    #[test]
    fn daily_until_to_count_test_2() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-30 19:59 UTC))),
                interval: 3,
            },
            kind: RecurrenceRuleKind::Daily,
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2023-03-30 19:59 UTC),
                    &event
                )
                .unwrap(),
            2
        )
    }

    #[test]
    fn weekly_until_to_count_test_1() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-04-15 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 103 },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2023-04-15 20:00 UTC),
                    &event
                )
                .unwrap(),
            17
        )
    }

    #[test]
    fn weekly_until_to_count_test_2() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-04-15 20:00 UTC))),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 103 },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2023-04-15 20:00 UTC),
                    &event
                )
                .unwrap(),
            8
        )
    }

    #[test]
    fn weekly_until_to_count_test_3() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-03-24 19:59 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Weekly { week_map: 103 },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2023-03-24 19:59 UTC),
                    &event
                )
                .unwrap(),
            0
        )
    }

    #[test]
    fn monthly_until_to_count_test_by_day_1() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2024-06-21 20:00 UTC))),
                interval: 3,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2024-06-21 20:00 UTC),
                    &event
                )
                .unwrap(),
            5
        )
    }

    #[test]
    fn monthly_until_to_count_test_by_day_2() {
        let event = TimeRange::new(
            datetime!(2023-03-31 18:30 UTC),
            datetime!(2023-03-31 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-12-31 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: true },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-31 18:30 UTC),
                    datetime!(2023-12-31 20:00 UTC),
                    &event
                )
                .unwrap(),
            5
        )
    }

    #[test]
    fn monthly_until_to_count_test_by_weekday_1() {
        let event = TimeRange::new(
            datetime!(2023-03-19 18:30 UTC),
            datetime!(2023-03-19 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-06-17 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-19 18:30 UTC),
                    datetime!(2023-06-17 20:00 UTC),
                    &event
                )
                .unwrap(),
            2
        )
    }

    #[test]
    fn monthly_until_to_count_test_by_weekday_2() {
        let event = TimeRange::new(
            datetime!(2023-03-31 18:30 UTC),
            datetime!(2023-03-31 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2023-12-29 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Monthly { is_by_day: false },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-31 18:30 UTC),
                    datetime!(2023-12-29 20:00 UTC),
                    &event
                )
                .unwrap(),
            3
        )
    }

    #[test]
    fn yearly_until_to_count_test_by_day_1() {
        let event = TimeRange::new(
            datetime!(2023-03-21 18:30 UTC),
            datetime!(2023-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2026-03-21 20:00 UTC))),
                interval: 2,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2023-03-21 18:30 UTC),
                    datetime!(2026-03-21 20:00 UTC),
                    &event
                )
                .unwrap(),
            1
        )
    }

    #[test]
    fn yearly_until_to_count_test_by_day_2() {
        let event = TimeRange::new(
            datetime!(2020-02-29 18:30 UTC),
            datetime!(2020-02-29 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2028-02-29 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: true },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2020-02-29 18:30 UTC),
                    datetime!(2028-02-29 20:00 UTC),
                    &event
                )
                .unwrap(),
            2
        )
    }

    #[test]
    fn yearly_until_to_count_test_by_weekday_1() {
        let event = TimeRange::new(
            datetime!(2021-03-21 18:30 UTC),
            datetime!(2021-03-21 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2026-03-21 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2021-03-21 18:30 UTC),
                    datetime!(2026-03-21 20:00 UTC),
                    &event
                )
                .unwrap(),
            5
        )
    }

    #[test]
    fn yearly_until_to_count_test_by_weekday_2() {
        let event = TimeRange::new(
            datetime!(2020-12-31 18:30 UTC),
            datetime!(2020-12-31 20:00 UTC),
        );
        let rec_rules = RecurrenceRuleSchema {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Until(datetime!(2027-12-31 20:00 UTC))),
                interval: 1,
            },
            kind: RecurrenceRuleKind::Yearly { is_by_day: false },
        };
        assert_eq!(
            rec_rules
                .until_to_count(
                    datetime!(2020-12-31 18:30 UTC),
                    datetime!(2027-12-31 20:00 UTC),
                    &event
                )
                .unwrap(),
            1
        )
    }
}
