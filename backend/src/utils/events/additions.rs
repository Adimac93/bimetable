use time::{Month, OffsetDateTime, Weekday};

use crate::app_errors::DefaultContext;

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

pub fn add_months(val: OffsetDateTime, chg: i32) -> anyhow::Result<OffsetDateTime> {
    let month_res = nth_next_month(val.month(), chg)?;
    let year_offset = (((val.month() as i32).checked_add(chg)).dc()? - 1).div_euclid(12);
    Ok(val
        .replace_year(val.year().checked_add(year_offset as i32).dc()?)
        .dc()?
        .replace_month(month_res)
        .dc()?)
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
