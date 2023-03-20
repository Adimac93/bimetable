use crate::utils::events::additions::{
    get_amount_from_week_map, next_good_month, next_good_month_by_weekday,
    nth_53_week_year_by_weekday, TimeStart, TimeTo,
};
use crate::utils::events::calculations::UntilToCountData;
use crate::utils::events::errors::EventError;
use time::Month;

pub fn daily_u_to_c(data: UntilToCountData) -> Result<u32, EventError> {
    Ok(((data.until - data.part_starts_at) / data.interval).whole_days() as u32 + 1)
}

pub fn weekly_u_to_c(data: UntilToCountData, week_map: &str) -> Result<u32, EventError> {
    let events_per_week = get_amount_from_week_map(week_map);
    let week_distance = (data.until.week_start() - data.part_starts_at.week_start()).whole_weeks();

    if week_distance == 0 {
        let base_event_amount = get_amount_from_week_map(
            &week_map[data.part_starts_at.weekday().number_days_from_monday() as usize
                ..=data.until.weekday().number_days_from_monday() as usize],
        ) as u32;

        if data.part_starts_at.time() > data.until.time() {
            return Ok(base_event_amount - 1);
        } else {
            return Ok(base_event_amount);
        }
    }

    let starting_week_amount = get_amount_from_week_map(
        &week_map[data.part_starts_at.weekday().number_days_from_monday() as usize..],
    ) as u32;

    let ending_week_amount = get_amount_from_week_map(
        &week_map[..=data.until.weekday().number_days_from_monday() as usize],
    ) as u32;

    Ok(
        (week_distance as u32 - 1) / data.interval * events_per_week as u32
            + starting_week_amount
            + ending_week_amount,
    )
}

pub fn monthly_u_to_c_by_day(data: UntilToCountData) -> Result<u32, EventError> {
    if data.part_starts_at.day() <= 28 {
        let base_res = (data.part_starts_at.year(), data.part_starts_at.month())
            .time_to((data.until.year(), data.until.month())) as u32;
        if data
            .part_starts_at
            .replace_year(data.until.year())
            .unwrap()
            .replace_month(data.until.month())
            .unwrap()
            > data.until
        {
            Ok(base_res + 1)
        } else {
            Ok(base_res)
        }
    } else {
        let mut monthly_step = data.part_starts_at;
        let mut res = 0;
        while monthly_step <= data.until {
            monthly_step = next_good_month(monthly_step, data.interval as i64)?;
            res += 1;
        }
        Ok(res)
    }
}

pub fn monthly_u_to_c_by_weekday(data: UntilToCountData) -> Result<u32, EventError> {
    if data.part_starts_at.day() <= 28 {
        let week_number = data.part_starts_at.day() - 1 / 7;
        let day_number = (data.part_starts_at.day() - 1) % 7 + 1;
        let target_day = week_number * 7 + day_number;
        let base_res = (data.part_starts_at.year(), data.part_starts_at.month())
            .time_to((data.until.year(), data.until.month())) as u32;
        if data
            .part_starts_at
            .replace_year(data.until.year())
            .unwrap()
            .replace_month(data.until.month())
            .unwrap()
            .replace_day(target_day)
            .unwrap()
            > data.until
        {
            Ok(base_res + 1)
        } else {
            Ok(base_res)
        }
    } else {
        let mut monthly_step = data.part_starts_at;
        let mut res = 0;
        while monthly_step <= data.until {
            monthly_step = next_good_month_by_weekday(monthly_step, data.interval as i64)?;
            res += 1;
        }
        Ok(res)
    }
}

pub fn yearly_u_to_c_by_day(data: UntilToCountData) -> Result<u32, EventError> {
    if let (Month::February, 29) = (data.part_starts_at.month(), data.part_starts_at.day()) {
        let mut monthly_step = data.part_starts_at;
        let mut res = 0;
        while monthly_step <= data.until {
            monthly_step = next_good_month(monthly_step, data.interval as i64 * 12)?;
            res += 1;
        }
        Ok(res)
    } else {
        let base_res = (data.part_starts_at.year() - data.until.year()) as u32 / data.interval;
        if data.part_starts_at.replace_year(data.until.year()).unwrap() > data.until {
            Ok(base_res + 1)
        } else {
            Ok(base_res)
        }
    }
}

pub fn yearly_u_to_c_by_weekday(data: UntilToCountData) -> Result<u32, EventError> {
    let (start_year, start_week, start_weekday) = data.part_starts_at.to_iso_week_date();
    let (end_year, end_week, end_weekday) = data.until.to_iso_week_date();
    if start_week == 52 {
        let mut yearly_step = data.part_starts_at;
        let mut res = 0;
        while yearly_step <= data.until {
            yearly_step = yearly_step
                .replace_year(nth_53_week_year_by_weekday(
                    yearly_step.year(),
                    1,
                    data.interval,
                )?)
                .unwrap();
            res += 1;
        }
        Ok(res)
    } else {
        let base_res = (end_year as u32 - start_year as u32) / data.interval;
        if start_week > end_week {
            return Ok(base_res);
        } else if start_week < end_week {
            return Ok(base_res + 1);
        }

        if start_weekday.time_to(end_weekday) < 0 {
            return Ok(base_res);
        } else if start_weekday.time_to(end_weekday) > 0 {
            return Ok(base_res + 1);
        }

        if data.part_starts_at.time() > data.until.time() {
            Ok(base_res)
        } else {
            Ok(base_res + 1)
        }
    }
}
