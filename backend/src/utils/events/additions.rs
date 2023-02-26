use std::ops::{Add, AddAssign, Sub, SubAssign};

use time::{
    ext::NumericalDuration, macros::datetime, util::weeks_in_year, Duration, Month, OffsetDateTime,
    Weekday,
};

use crate::app_errors::DefaultContext;

use super::{errors::EventError, models::TimeRange};

pub fn get_amount_from_week_map(week_map: &str) -> u8 {
    week_map.chars().map(|x| x as u8 - 48).sum::<u8>()
}

pub fn get_offset_from_the_map(week_map: &str, mut event_number: u8, start_at: u8) -> u8 {
    for i in 1..8 {
        if &week_map[(start_at + i) as usize % 7..=(start_at + i) as usize % 7] == "1" {
            event_number -= 1;
        }
        if event_number == 0 {
            return i;
        }
    }
    return 7;
}

pub trait AddTime
where
    Self: Sized,
{
    fn add_days(self, chg: i64) -> anyhow::Result<Self>;
    fn add_weeks(self, chg: i64) -> anyhow::Result<Self>;
    fn add_months(self, chg: i64) -> anyhow::Result<Self>;
    fn add_years(self, chg: i64) -> anyhow::Result<Self>;
}

impl AddTime for OffsetDateTime {
    fn add_days(self, chg: i64) -> anyhow::Result<Self> {
        Ok(self.checked_add((chg as i64).days()).dc()?)
    }

    fn add_weeks(self, chg: i64) -> anyhow::Result<Self> {
        Ok(self.checked_add((chg as i64).weeks()).dc()?)
    }

    fn add_months(self, chg: i64) -> anyhow::Result<OffsetDateTime> {
        let month_res = nth_next_month(self.month(), chg)?;
        let year_offset = (((self.month() as i64).checked_add(chg)).dc()? - 1).div_euclid(12);
        Ok(self
            .replace_year(self.year().checked_add(year_offset as i32).dc()?)
            .dc()?
            .replace_month(month_res)
            .dc()?)
    }

    fn add_years(self, chg: i64) -> anyhow::Result<Self> {
        Ok(self
            .replace_year(self.year().checked_add(chg as i32).dc()?)
            .dc()?)
    }
}

fn nth_next_month(val: Month, chg: i64) -> anyhow::Result<Month> {
    Month::try_from((((val as i64).checked_add(chg).dc()? - 1).rem_euclid(12) + 1) as u8).dc()
}

pub fn yearly_conv_data(
    part_starts_at: OffsetDateTime,
) -> anyhow::Result<(Weekday, u8, OffsetDateTime)> {
    // get the week number and the weekday
    let target_weekday = part_starts_at.weekday();
    let target_week = part_starts_at.iso_week() - 1;
    let mut base_year = part_starts_at
        .replace_day(1)?
        .replace_month(Month::January)?;

    // accounting for edge case in events happening in the start/end of the year
    if target_week >= 51 && part_starts_at.month() == Month::January {
        base_year = base_year.replace_year(base_year.year().checked_sub(1).dc()?)?;
    } else if target_week == 0 && part_starts_at.month() == Month::December {
        base_year = base_year.replace_year(base_year.year().checked_add(1).dc()?)?;
    }

    Ok((target_weekday, target_week, base_year))
}

pub trait TimeTo
where
    Self: Sized,
{
    fn time_to(self, rhs: Self) -> i32;
}

impl TimeTo for Weekday {
    fn time_to(self, rhs: Self) -> i32 {
        (rhs as i32) - (self as i32)
    }
}

impl TimeTo for Month {
    fn time_to(self, rhs: Self) -> i32 {
        (rhs as i32) - (self as i32)
    }
}

impl TimeTo for (i32, Month) {
    fn time_to(self, rhs: Self) -> i32 {
        (rhs.0 - self.0) * 12 + self.1.time_to(rhs.1)
    }
}

pub trait CyclicTimeTo: TimeTo {
    fn cyclic_time_to(self, rhs: Self) -> u32;
}

impl CyclicTimeTo for Weekday {
    fn cyclic_time_to(self, rhs: Self) -> u32 {
        self.time_to(rhs).rem_euclid(7) as u32
    }
}

impl CyclicTimeTo for Month {
    fn cyclic_time_to(self, rhs: Self) -> u32 {
        self.time_to(rhs).rem_euclid(12) as u32
    }
}

impl Add<Duration> for TimeRange {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::new(self.start + rhs, self.end + rhs)
    }
}

impl AddAssign<Duration> for TimeRange {
    fn add_assign(&mut self, rhs: Duration) {
        self.start += rhs;
        self.end += rhs;
    }
}

impl Sub<Duration> for TimeRange {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::new(self.start - rhs, self.end - rhs)
    }
}

impl SubAssign<Duration> for TimeRange {
    fn sub_assign(&mut self, rhs: Duration) {
        self.start -= rhs;
        self.end -= rhs;
    }
}

pub trait TimeStart {
    fn day_start(self) -> Self;
    fn week_start(self) -> Self;
    fn month_start(self) -> Self;
    fn year_start(self) -> Self;
}

impl TimeStart for OffsetDateTime {
    fn day_start(self) -> Self {
        self.replace_time(time::Time::MIDNIGHT)
    }

    fn week_start(self) -> Self {
        self.day_start() - (Weekday::Monday.cyclic_time_to(self.weekday()) as i64).days()
    }

    fn month_start(self) -> Self {
        self.day_start().replace_day(1).unwrap()
    }

    fn year_start(self) -> Self {
        self.month_start().replace_month(Month::January).unwrap()
    }
}

pub fn next_good_month(time: OffsetDateTime, chg: i64) -> Result<OffsetDateTime, EventError> {
    let mut first_day = time.replace_day(1).dc()?;
    first_day = first_day.add_months(chg).dc()?;
    while first_day.replace_day(time.day()).is_err() {
        first_day = first_day.add_months(chg).dc()?;
    }
    Ok(first_day.replace_day(time.day()).dc()?)
}

pub fn nth_good_month(
    mut monthly_step: OffsetDateTime,
    mut count: u32,
    chg: i64,
) -> Result<OffsetDateTime, EventError> {
    while count > 0 {
        monthly_step = next_good_month(monthly_step, chg)?;
        count -= 1;
    }

    Ok(monthly_step)
}

pub fn next_good_month_by_weekday(
    time: OffsetDateTime,
    chg: i64,
) -> Result<OffsetDateTime, EventError> {
    let mut first_day = time.replace_day(1).dc()?;
    first_day = first_day.add_months(chg).dc()?;
    let day_offset = (time.day() - 1) / 7 * 7 + 1;
    while first_day
        .replace_day(day_offset + first_day.weekday().cyclic_time_to(time.weekday()) as u8)
        .is_err()
    {
        first_day = first_day.add_months(chg).dc()?;
    }
    Ok(first_day
        .replace_day(day_offset + first_day.weekday().cyclic_time_to(time.weekday()) as u8)
        .dc()?)
}

pub fn nth_53_week_year_by_weekday(
    mut yearly_step: i32,
    mut count: u32,
    chg: u32,
) -> Result<i32, EventError> {
    while count > 0 {
        yearly_step = yearly_step.checked_add(i32::try_from(chg).dc()?).dc()?;
        if weeks_in_year(yearly_step) == 53 {
            count -= 1;
        }
    }

    Ok(yearly_step)
}

pub fn iso_year_start(year: i32) -> OffsetDateTime {
    let time = OffsetDateTime::UNIX_EPOCH
        .replace_year(year)
        .unwrap()
        .year_start();
    let (_, iso_week, weekday) = time.to_iso_week_date();

    if iso_week == 52 || iso_week == 53 {
        time + (weekday.cyclic_time_to(Weekday::Monday) as i64).days()
    } else {
        time - (Weekday::Monday.time_to(weekday) as i64).days()
    }
}

pub fn max_date_time() -> OffsetDateTime {
    datetime!(9999-12-31 23:59:59.999999999 UTC)
}

#[cfg(test)]
mod test {
    use time::Month;

    use crate::utils::events::additions::{CyclicTimeTo, TimeTo};

    #[test]
    fn time_to_test() {
        let a = Month::April;
        let b = Month::August;
        assert_eq!(a.time_to(a), 0);
        assert_eq!(a.time_to(b), 4);
        assert_eq!(b.time_to(a), -4);
        assert_eq!(a.cyclic_time_to(a), 0);
        assert_eq!(a.cyclic_time_to(b), 4);
        assert_eq!(b.cyclic_time_to(a), 8);

        let a = (2021, Month::April);
        let b = (2023, Month::February);
        assert_eq!(a.time_to(b), 22);
        assert_eq!(b.time_to(a), -22);
    }
}
