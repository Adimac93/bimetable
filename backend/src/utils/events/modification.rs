use sqlx::types::Json;
use time::{
    macros::datetime,
    util::{days_in_year_month, is_leap_year, weeks_in_year},
    Duration, Month, OffsetDateTime, Weekday,
};

use crate::app_errors::DefaultContext;

use super::{
    errors::EventError,
    models::{EventPart, EventRules, RecurrenceEndsAt, TimeRules},
};

impl EventPart {
    pub fn verify_event_part(&self) -> Result<(), EventError> {
        match self.part_length {
            Some(RecurrenceEndsAt::Count(x)) if x == 0 => Err(EventError::WrongEventBounds),
            Some(RecurrenceEndsAt::Until(time)) if self.part_starts_at > time => {
                Err(EventError::WrongEventBounds)
            }
            _ => Ok(()),
        }
    }

    fn count_to_until(&self) -> Result<Option<OffsetDateTime>, anyhow::Error> {
        let Json(rec_rules) = self
            .event_data
            .recurrence_rule
            .as_ref()
            .ok_or(EventError::NotFound)?;
        let event_starts_at = self
            .event_data
            .starts_at
            .as_ref()
            .ok_or(EventError::NotFound)?;
        let event_ends_at = self
            .event_data
            .ends_at
            .as_ref()
            .ok_or(EventError::NotFound)?;
        match rec_rules {
            EventRules::Yearly {
                time_rules,
                is_by_day,
            } => {
                if *is_by_day {
                    year_is_by_day_count_to_until(
                        self.part_starts_at,
                        self.part_length.as_ref(),
                        time_rules.interval,
                        *event_starts_at,
                        *event_ends_at,
                    )
                } else {
                    year_count_to_until(
                        self.part_starts_at,
                        self.part_length.as_ref(),
                        time_rules.interval,
                        *event_starts_at,
                        *event_ends_at,
                    )
                }
            }
            EventRules::Monthly {
                time_rules,
                is_by_day,
            } => {
                if *is_by_day {
                    month_is_by_day_count_to_until(
                        self.part_starts_at,
                        self.part_length.as_ref(),
                        time_rules.interval,
                        *event_starts_at,
                        *event_ends_at,
                    )
                } else {
                    month_count_to_until(
                        self.part_starts_at,
                        self.part_length.as_ref(),
                        time_rules.interval,
                        *event_starts_at,
                        *event_ends_at,
                    )
                }
            }
            EventRules::Weekly {
                time_rules,
                week_map,
            } => {
                let string_week_map = format!("{:0>7b}", week_map % 128);
                week_count_to_until(
                    self.part_starts_at,
                    self.part_length.as_ref(),
                    time_rules.interval,
                    string_week_map,
                    *event_starts_at,
                    *event_ends_at,
                )
            }
            EventRules::Daily { time_rules } => day_count_to_until(
                self.part_starts_at,
                self.part_length.as_ref(),
                time_rules.interval,
                *event_starts_at,
                *event_ends_at,
            ),
        }
    }
}

fn day_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(n) => {
                // let event_duration: Duration = event_ends_at - event_starts_at;
                // let time_to_next_event: Duration = event_duration
                //     .checked_add(
                //         Duration::days(1)
                //             .checked_mul(i32::try_from(interval).dc()?)
                //             .dc()?,
                //     )
                //     .dc()?;
                // let rec_ends_at: OffsetDateTime = part_starts_at
                //     .checked_add(
                //         time_to_next_event
                //             .checked_mul(i32::try_from(*n).dc()?)
                //             .dc()?,
                //     )
                //     .dc()?;
                return Ok(Some(part_starts_at + Duration::days((*n * interval) as i64) + (event_ends_at - event_starts_at)));
            }
        }
    }
    Ok(None) // never
}

fn week_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    week_map: String,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(mut n) => {
                // get an amount of events in 1 week
                let week_event_num = get_amount_from_week_map(&week_map);
                // calculate the number of week intervals passed with integer division
                let mut weeks_passed = (n / week_event_num as u32) * interval;
                // - this is the amount of weeks passed, from the first Monday after the part starts
                // calculate the modulo between these numbers and seek the nth occurence of the event in one week, where n is the modulo + 1
                // n has already been verified to be greater than 0, so no overflow (underflow) happens
                let modulo = (n - 1) % week_event_num as u32 + 1;
                let offset_res = get_offset_from_the_map(
                    &week_map,
                    modulo as u8,
                    part_starts_at.weekday().number_days_from_monday() + 1,
                );

                // check whether last events carry over to the next week interval
                if part_starts_at.weekday().number_days_from_monday() + offset_res > 6 {
                    weeks_passed += interval - 1
                };

                println!("dbg: week_event_num = {week_event_num}, weeks_passed = {weeks_passed}, modulo = {modulo}, offset_res = {offset_res}");

                let rec_ends_at = part_starts_at
                    + Duration::weeks(weeks_passed as i64)
                    + Duration::days(offset_res as i64)
                    + (event_ends_at - event_starts_at);

                return Ok(Some(rec_ends_at));
            }
        }
    }
    Ok(None) // never
}

fn get_amount_from_week_map(week_map: &str) -> u8 {
    week_map.chars().map(|x| x as u8 - 48).sum::<u8>()
}

fn get_amount_from_week_map_from(week_map: &str, weekday: Weekday) -> u8 {
    (&week_map[(weekday.number_days_from_monday() as usize)..=6])
        .chars()
        .map(|x| x as u8 - 48)
        .sum::<u8>()
}

fn get_offset_from_the_map(week_map: &str, event_number: u8, start_at: u8) -> u8 {
    let mut two_week_map = week_map.to_string();
    two_week_map.push_str(week_map);
    let mut event_count: u8 = 0;
    let mut i = start_at;
    while event_count < event_number {
        if i == start_at + 6 {
            return 6;
        }
        // i is guaranteed to be between 0 and 13
        if &two_week_map[i as usize..=i as usize] == "1" {
            event_count += 1
        }
        i += 1;
    }
    return i - start_at;
}

fn month_is_by_day_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(mut n) => match part_starts_at.day() {
                1..=28 => {
                    let base_value = add_months(part_starts_at, n * interval)?;
                    return Ok(Some(base_value + (event_ends_at - event_starts_at)));
                }
                29..=31 => {
                    let mut part_ends_until = part_starts_at.replace_day(1)?;
                    while n > 0 {
                        part_ends_until = add_months(part_ends_until, interval)?;
                        if days_in_year_month(part_ends_until.year(), part_ends_until.month())
                            >= part_starts_at.day()
                        {
                            n -= 1;
                        }
                    }
                    return Ok(Some(part_ends_until.replace_day(part_starts_at.day())? + (event_ends_at - event_starts_at)));
                }
                _ => unreachable!(),
            },
        }
    }
    Ok(None) // never
}

fn month_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(mut n) => match part_starts_at.day() {
                1..=28 => {
                    let week_number = (part_starts_at.day() - 1) / 7;
                    let target_weekday = part_starts_at.weekday();

                    let target_month = add_months(part_starts_at, n * interval)?.replace_day(1)?;
                    let first_day_weekday = target_month.weekday();

                    let offset = Duration::days(days_between_two_weekdays(first_day_weekday, target_weekday) as i64);

                    return Ok(Some(target_month + Duration::days((week_number as i64) * 7) + offset + (event_ends_at - event_starts_at)));
                }
                29..=31 => {
                    let mut part_ends_until = part_starts_at;
                    let target_weekday = part_starts_at.weekday();
                    let mut target_day = 0;
                    while n > 0 {
                        part_ends_until = add_months(part_ends_until, interval)?;
                        let first_day_weekday = part_ends_until.replace_day(1)?.weekday();
                        target_day = 29 + days_between_two_weekdays(first_day_weekday, target_weekday);
                        if days_in_year_month(part_ends_until.year(), part_ends_until.month())
                            >= target_day
                        {
                            n -= 1;
                        }
                    }
                    part_ends_until = part_ends_until.replace_day(target_day).dc()?;
                    return Ok(Some(part_ends_until + (event_ends_at - event_starts_at)));
                }
                _ => unreachable!(),
            },
        }
    }
    Ok(None) // never
}

fn add_months(val: OffsetDateTime, chg: u32) -> anyhow::Result<OffsetDateTime> {
    let month_res = nth_next_month(val.month(), chg)?;
    let year_number = ((val.month() as u8 + chg as u8) - 1) / 12;
    Ok(val
        .replace_year(val.year() + year_number as i32)
        .dc()?
        .replace_month(month_res)
        .dc()?)
}

fn nth_next_month(val: Month, chg: u32) -> anyhow::Result<Month> {
    Month::try_from((((val as u32).checked_add(chg).dc()? - 1) % 12 + 1) as u8).dc()
}

fn days_between_two_weekdays(val_a: Weekday, val_b: Weekday) -> u8 {
    (((val_b.number_from_monday() as i8) - (val_a.number_from_monday() as i8)).rem_euclid(7)) as u8
}

fn year_is_by_day_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(mut n) => {
                match (part_starts_at.month(), part_starts_at.day()) {
                    (Month::February, 29) => {
                        let mut part_ends_until = part_starts_at.replace_day(1)?;
                        while n > 0 {
                            part_ends_until = part_ends_until.replace_year(
                                part_ends_until.year() + i32::try_from(n * interval).dc()?,
                            )?;
                            if is_leap_year(part_ends_until.year()) {
                                n -= 1;
                            }
                        }
                        return Ok(Some(part_ends_until.replace_day(part_starts_at.day())? + (event_ends_at - event_starts_at)));
                    }
                    _ => {
                        let base_value = part_starts_at.replace_year(
                            part_starts_at.year() + i32::try_from(n * interval).dc()?,
                        )?;
                        return Ok(Some(base_value + (event_ends_at - event_starts_at)));
                    }
                }
            }
        }
    }
    Ok(None) // never
}

fn year_count_to_until(
    part_starts_at: OffsetDateTime,
    part_ends_at: Option<&RecurrenceEndsAt>,
    interval: u32,
    event_starts_at: OffsetDateTime,
    event_ends_at: OffsetDateTime,
) -> anyhow::Result<Option<OffsetDateTime>> {
    if let Some(rec_ends_at) = part_ends_at {
        match rec_ends_at {
            RecurrenceEndsAt::Until(t) => return Ok(Some(*t)),
            RecurrenceEndsAt::Count(mut n) => {
                // get the week number and the weekday
                let target_weekday = part_starts_at.weekday();
                let target_week = part_starts_at.iso_week() - 1;
                let mut base_year = part_starts_at.replace_day(1)?.replace_month(Month::January)?;
                // edge case happens when the event is at the beginning of the year, but is still in the last week of the previous year
                if target_week >= 51 && part_starts_at.month() == Month::January {
                    base_year = base_year.replace_year(base_year.year() - 1)?;
                }

                match target_week {
                    52 => {
                        let mut part_ends_until = base_year;
                        while n > 0 {
                            part_ends_until = part_ends_until.replace_year(
                                part_ends_until.year() + i32::try_from(interval).dc()?,
                            )?;
                            if weeks_in_year(part_ends_until.year()) == 53 {
                                n -= 1;
                            }
                        }
                        let first_monday = part_ends_until + Duration::days(days_between_two_weekdays(part_ends_until.weekday(), Weekday::Monday) as i64);
                        return Ok(Some(
                            first_monday
                            + Duration::weeks((target_week - first_monday.iso_week() + 1) as i64)
                            + Duration::days(days_between_two_weekdays(Weekday::Monday, target_weekday) as i64)
                            + (event_ends_at - event_starts_at)
                        ));
                    }
                    _ => {
                        base_year = base_year.replace_year(base_year.year() + i32::try_from(n * interval).dc()?)?;
                        let first_monday = base_year + Duration::days(days_between_two_weekdays(base_year.weekday(), Weekday::Monday) as i64);
                        return Ok(Some(
                            first_monday
                            + Duration::weeks((target_week - first_monday.iso_week() + 1) as i64)
                            + Duration::days(days_between_two_weekdays(Weekday::Monday, target_weekday) as i64)
                            + (event_ends_at - event_starts_at)
                        ));
                    }
                }
            }
        }
    }
    Ok(None) // never
}

mod recurrence_tests {
    use uuid::Uuid;

    use crate::utils::events::models::Event;

    use super::*;

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
            EventRules::Daily { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(15)), interval: 3 } },
            datetime!(2023-02-21 10:00 +1),
            RecurrenceEndsAt::Count(7)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-03-14 12:15 +1)))
    }

    #[test]
    fn weekly_recurrence_test() {
        let data = create_test_event_part(
            datetime!(2023-02-15 10:00 +1),
            datetime!(2023-02-15 12:15 +1),
            EventRules::Weekly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(30)), interval: 2 }, week_map: 86 },
            datetime!(2023-02-27 10:00 +1),
            RecurrenceEndsAt::Count(5)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-03-15 12:15 +1)))
    }

    #[test]
    fn weekly_recurrence_test_next_week_offset() {
        let data = create_test_event_part(
            datetime!(2023-02-15 10:00 +1),
            datetime!(2023-02-15 12:15 +1),
            EventRules::Weekly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(30)), interval: 2 }, week_map: 86 },
            datetime!(2023-03-01 10:00 +1),
            RecurrenceEndsAt::Count(7)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-03-27 12:15 +1)))
    }

    #[test]
    fn monthly_recurrence_test_by_day() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Monthly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 2 }, is_by_day: true },
            datetime!(2023-04-18 10:00 +1),
            RecurrenceEndsAt::Count(2)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-08-18 12:15 +1)))
    }

    #[test]
    fn monthly_recurrence_test_by_day_month_end() {
        let data = create_test_event_part(
            datetime!(2025-01-29 10:00 +1),
            datetime!(2025-01-29 12:15 +1),
            EventRules::Monthly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(30)), interval: 5 }, is_by_day: true },
            datetime!(2025-11-29 10:00 +1),
            RecurrenceEndsAt::Count(15),
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2032-07-29 12:15 +1)))
    }

    #[test]
    fn monthly_recurrence_test_by_weekday() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Monthly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 2 }, is_by_day: false },
            datetime!(2023-04-15 10:00 +1),
            RecurrenceEndsAt::Count(2)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-08-19 12:15 +1)))
    }

    #[test]
    fn monthly_recurrence_test_by_weekday_month_end() {
        let data = create_test_event_part(
            datetime!(2023-01-31 10:00 +1),
            datetime!(2023-01-31 12:15 +1),
            EventRules::Monthly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 1 }, is_by_day: false },
            datetime!(2023-05-30 10:00 +1),
            RecurrenceEndsAt::Count(2),
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2023-10-31 12:15 +1)))
    }

    #[test]
    fn yearly_recurrence_test_by_day() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Yearly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 2 }, is_by_day: true },
            datetime!(2025-02-18 10:00 +1),
            RecurrenceEndsAt::Count(2)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2029-02-18 12:15 +1)))
    }

    #[test]
    fn yearly_recurrence_test_by_day_feb_29() {
        let data = create_test_event_part(
            datetime!(2024-02-29 10:00 +1),
            datetime!(2024-02-29 12:15 +1),
            EventRules::Yearly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 1 }, is_by_day: true },
            datetime!(2028-02-29 10:00 +1),
            RecurrenceEndsAt::Count(1)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2032-02-29 12:15 +1)))
    }

    #[test]
    fn yearly_recurrence_test_by_weekday() {
        let data = create_test_event_part(
            datetime!(2023-02-18 10:00 +1),
            datetime!(2023-02-18 12:15 +1),
            EventRules::Yearly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 2 }, is_by_day: false },
            datetime!(2025-02-15 10:00 +1),
            RecurrenceEndsAt::Count(2)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2029-02-17 12:15 +1)))
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_52nd_week() {
        let data = create_test_event_part(
            datetime!(2020-12-26 10:00 +1),
            datetime!(2020-12-26 12:15 +1),
            EventRules::Yearly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 1 }, is_by_day: false },
            datetime!(2022-01-01 10:00 +1),
            RecurrenceEndsAt::Count(1)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2022-12-31 12:15 +1)))
    }

    #[test]
    fn yearly_recurrence_test_by_weekday_53rd_week() {
        let data = create_test_event_part(
            datetime!(2020-12-30 10:00 +1),
            datetime!(2020-12-30 12:15 +1),
            EventRules::Yearly { time_rules: TimeRules { ends_at: Some(RecurrenceEndsAt::Count(10)), interval: 1 }, is_by_day: false },
            datetime!(2026-12-31 10:00 +1),
            RecurrenceEndsAt::Count(1)
        );

        assert_eq!(data.count_to_until().unwrap(), Some(datetime!(2032-12-30 12:15 +1)))
    }
}
