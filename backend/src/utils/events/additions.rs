use std::ops::{Add, AddAssign, Sub, SubAssign};

use time::{ext::NumericalDuration, Duration, Month, OffsetDateTime, Weekday};

use crate::app_errors::DefaultContext;

use super::models::TimeRange;

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

pub trait AddMonths
where
    Self: Sized,
{
    fn add_months(self, chg: i32) -> anyhow::Result<Self>;
}

impl AddMonths for OffsetDateTime {
    fn add_months(self, chg: i32) -> anyhow::Result<OffsetDateTime> {
        let month_res = nth_next_month(self.month(), chg)?;
        let year_offset = (((self.month() as i32).checked_add(chg)).dc()? - 1).div_euclid(12);
        Ok(self
            .replace_year(self.year().checked_add(year_offset as i32).dc()?)
            .dc()?
            .replace_month(month_res)
            .dc()?)
    }
}

fn nth_next_month(val: Month, chg: i32) -> anyhow::Result<Month> {
    Month::try_from((((val as i32).checked_add(chg).dc()? - 1).rem_euclid(12) + 1) as u8).dc()
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

pub fn next_good_month(time: OffsetDateTime, chg: i32) -> OffsetDateTime {
    let mut first_day = time.replace_day(1).unwrap();
    first_day = first_day.add_months(chg).unwrap();
    while first_day.replace_day(time.day()).is_err() {
        first_day = first_day.add_months(chg).unwrap();
    }
    first_day.replace_day(time.day()).unwrap()
}

pub fn next_good_month_by_weekday(time: OffsetDateTime, chg: i32) -> OffsetDateTime {
    let mut first_day = time.replace_day(1).unwrap();
    first_day = first_day.add_months(chg).unwrap();
    let day_offset = (time.day() - 1) / 7 * 7 + 1;
    while first_day
        .replace_day(day_offset + first_day.weekday().cyclic_time_to(time.weekday()) as u8)
        .is_err()
    {
        first_day = first_day.add_months(chg).unwrap();
    }
    first_day
        .replace_day(day_offset + first_day.weekday().cyclic_time_to(time.weekday()) as u8)
        .unwrap()
}

pub fn iso_year_start(year: i32) -> OffsetDateTime {
    let time = OffsetDateTime::now_local()
        .unwrap()
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
