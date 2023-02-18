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

pub fn add_months(val: OffsetDateTime, chg: u32) -> anyhow::Result<OffsetDateTime> {
    let month_res = nth_next_month(val.month(), chg)?;
    let year_number = (((val.month() as u32).checked_add(chg)).dc()? - 1) / 12;
    Ok(val
        .replace_year(val.year().checked_add(year_number as i32).dc()?)
        .dc()?
        .replace_month(month_res)
        .dc()?)
}

fn nth_next_month(val: Month, chg: u32) -> anyhow::Result<Month> {
    Month::try_from((((val as u32).checked_add(chg).dc()? - 1) % 12 + 1) as u8).dc()
}

pub fn days_between_two_weekdays(val_a: Weekday, val_b: Weekday) -> u8 {
    ((val_b.number_from_monday() as i8) - (val_a.number_from_monday() as i8)).rem_euclid(7) as u8
}
