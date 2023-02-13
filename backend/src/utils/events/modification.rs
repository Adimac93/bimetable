use sqlx::types::Json;
use time::{Duration, OffsetDateTime, Weekday, Month, util::{days_in_year_month, is_leap_year}, macros::datetime};
use tracing::event;

use crate::app_errors::DefaultContext;

use super::{models::{EventPart, RecurrenceEndsAt, TimeRules, EventRules}, errors::EventError};

impl EventPart {
    pub fn verify_event_part(&self) -> Result<(), EventError> {
        match self.part_length {
            Some(RecurrenceEndsAt::Count(x)) if x == 0 => Err(EventError::WrongEventBounds),
            Some(RecurrenceEndsAt::Until(time)) if self.part_starts_at > time => Err(EventError::WrongEventBounds),
            _ => Ok(()),
        }
    }

    fn count_to_until(
        &self,
    ) -> Result<Option<OffsetDateTime>, anyhow::Error> {
        let Json(rec_rules) = self.event_data.recurrence_rule.as_ref().ok_or(EventError::NotFound)?;
        let event_starts_at = self.event_data.starts_at.as_ref().ok_or(EventError::NotFound)?;
        let event_ends_at = self.event_data.ends_at.as_ref().ok_or(EventError::NotFound)?;
        match rec_rules {
            EventRules::Yearly { time_rules, is_by_day } => {
                if *is_by_day {
                    year_is_by_day_count_to_until(self.part_starts_at, self.part_length.as_ref(), time_rules.interval, *event_starts_at, *event_ends_at)
                } else {
                    todo!()
                }
            }
            EventRules::Monthly { time_rules, is_by_day } => {
                if *is_by_day {
                    month_is_by_day_count_to_until(self.part_starts_at, self.part_length.as_ref(), time_rules.interval, *event_starts_at, *event_ends_at)
                } else {
                    todo!()
                }
            }
            EventRules::Weekly { time_rules, week_map } => {
                let string_week_map = format!("{:0>7b}", week_map % 128);
                week_count_to_until(self.part_starts_at, self.part_length.as_ref(), time_rules.interval, string_week_map, *event_starts_at, *event_ends_at)
            }
            EventRules::Daily { time_rules } => {
                day_count_to_until(self.part_starts_at, self.part_length.as_ref(), time_rules.interval, *event_starts_at, *event_ends_at)
            }
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
                let event_duration: Duration = event_ends_at - event_starts_at;
                let time_to_next_event: Duration =
                    event_duration.checked_add(Duration::days(1).checked_mul(i32::try_from(interval).dc()?).dc()?).dc()?;
                let rec_ends_at: OffsetDateTime =
                    part_starts_at.checked_add(time_to_next_event.checked_mul(i32::try_from(*n).dc()?).dc()?).dc()?;
                return Ok(Some(rec_ends_at));
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
                let offset_res =
                    get_offset_from_the_map(&week_map, modulo as u8, part_starts_at.weekday().number_days_from_monday());

                // check whether last events carry over to the next week interval
                if part_starts_at.weekday().number_days_from_monday() + offset_res > 6 {
                    weeks_passed += interval - 1
                };

                let rec_ends_at = part_starts_at + Duration::weeks(weeks_passed as i64) + Duration::days(weeks_passed as i64) + (event_ends_at - event_starts_at);

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
    (&week_map[(weekday.number_days_from_monday() as usize)..=6]).chars().map(|x| x as u8 - 48).sum::<u8>()
}

fn get_offset_from_the_map(week_map: &str, event_number: u8, start_at: u8) -> u8 {
    let mut two_week_map = week_map.to_string();
    two_week_map.push_str(week_map);
    let mut event_count: u8 = 0;
    let mut i = start_at;
    while event_count < event_number {
        if i == start_at + 6 { return 6 }
        // i is guaranteed to be between 0 and 13
        if &two_week_map[i as usize..=i as usize] == "1" { event_count += 1 }
        i += 1;
    }
    return i - start_at - 1
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
            RecurrenceEndsAt::Count(mut n) => {
                match part_starts_at.day() {
                    1..=28 => {
                        let base_value = add_months(part_starts_at, n * interval)?;
                        return Ok(Some(base_value + (event_ends_at - event_starts_at)));
                    },
                    29..=31 => {
                        let mut part_ends_until = part_starts_at;
                        while n > 0 {
                            part_ends_until = add_months(part_ends_until, interval)?;
                            if days_in_year_month(part_ends_until.year(), part_ends_until.month()) <= part_starts_at.day() {
                                n -= 1;
                            }
                        }
                        return Ok(Some(part_ends_until + (event_ends_at - event_starts_at)));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    Ok(None) // never
}

fn add_months(val: OffsetDateTime, chg: u32) -> anyhow::Result<OffsetDateTime> {
    let month_res = nth_next_month(val.month(), chg)?;
    let year_number = (val.month() as u8 + chg as u8) / 12;
    Ok(val.replace_year(val.year() + year_number as i32).dc()?.replace_month(month_res).dc()?)
}

fn nth_next_month(val: Month, chg: u32) -> anyhow::Result<Month> {
    Month::try_from((((val as u32).checked_add(chg).dc()? - 1) % 12 + 1) as u8).dc()
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
                        let mut part_ends_until = part_starts_at;
                        while n > 0 {
                            part_ends_until = part_ends_until.replace_year(part_ends_until.year() + i32::try_from(n * interval).dc()?)?;
                            if is_leap_year(part_ends_until.year()) {
                                n -= 1;
                            }
                        }
                        return Ok(Some(part_ends_until + (event_ends_at - event_starts_at)));
                    }
                    _ => {
                        let base_value = part_starts_at.replace_year(part_starts_at.year() + i32::try_from(n * interval).dc()?)?;
                        return Ok(Some(base_value + (event_ends_at - event_starts_at)));
                    },
                }
            }
        }
    }
    Ok(None) // never
}
