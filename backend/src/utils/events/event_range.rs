use time::{
    ext::NumericalDuration,
    util::{days_in_year_month, weeks_in_year},
    Month, Weekday,
};

use super::{
    additions::{add_months, yearly_conv_data, CyclicTimeTo, TimeTo},
    calculations::EventRangeData,
    errors::EventError,
    models::TimeRange,
};

pub fn get_daily_events(range_data: EventRangeData) -> Vec<TimeRange> {
    let day_amount = (range_data.range.start - range_data.event_range.end).whole_days();
    let mut offset_from_origin_event =
        (day_amount - day_amount % range_data.interval as i64).days();

    let mut res = Vec::new();

    while range_data.event_range.start + offset_from_origin_event < range_data.range.end {
        res.push(TimeRange::new(
            range_data.event_range.start + offset_from_origin_event,
            range_data.event_range.end + offset_from_origin_event,
        ));

        offset_from_origin_event += (range_data.interval as i64).days();
    }

    res
}

pub fn get_weekly_events(range_data: EventRangeData, week_map: &str) -> Vec<TimeRange> {
    let week_amount = (range_data.range.start - range_data.event_range.end).whole_weeks();
    let mut offset_from_origin_event =
        (week_amount - week_amount % range_data.interval as i64).weeks();

    let mut res = Vec::new();

    while range_data.event_range.start + offset_from_origin_event < range_data.range.end {
        let weekly_start_step = range_data.event_range.start + offset_from_origin_event;
        let weekly_end_step = range_data.event_range.end + offset_from_origin_event;
        let week_start = weekly_end_step
            - (time::Weekday::Monday.cyclic_time_to(weekly_end_step.weekday()) as i64).days();
        for i in 0..7 {
            if &week_map[i..=i] == "1" && week_start + (i as i64).days() > range_data.range.start {
                res.push(TimeRange::new(
                    weekly_start_step + (i as i64).days(),
                    weekly_end_step + (i as i64).days(),
                ));
            };
        }

        offset_from_origin_event += (range_data.interval as i64).weeks();
    }

    res
}

pub fn get_monthly_events_by_day(range_data: EventRangeData) -> Vec<TimeRange> {
    let month_amount = (
        range_data.range.start.year(),
        range_data.range.start.month(),
    )
        .time_to((
            range_data.event_range.end.year(),
            range_data.event_range.end.month(),
        )) as u32;
    let offset_from_origin_event = month_amount - month_amount % range_data.interval;
    let first_month_day = range_data.event_range.start.replace_day(1).unwrap();
    let mut monthly_step = add_months(first_month_day, offset_from_origin_event).unwrap();

    let mut res = Vec::new();

    while monthly_step < range_data.range.end {
        if days_in_year_month(monthly_step.year(), monthly_step.month())
            >= range_data.event_range.start.day()
        {
            res.push(TimeRange::new(
                monthly_step,
                monthly_step + (range_data.event_range.end - range_data.event_range.start),
            ));
        };

        monthly_step = add_months(monthly_step, range_data.interval).unwrap();
    }

    res
}

pub fn get_monthly_events_by_weekday(range_data: EventRangeData) -> Vec<TimeRange> {
    let month_amount = (
        range_data.range.start.year(),
        range_data.range.start.month(),
    )
        .time_to((
            range_data.event_range.end.year(),
            range_data.event_range.end.month(),
        )) as u32;
    let offset_from_origin_event = month_amount - month_amount % range_data.interval;
    let first_month_day = range_data.event_range.start.replace_day(1).unwrap();
    let mut monthly_step = add_months(first_month_day, offset_from_origin_event).unwrap();

    let target_weekday = range_data.event_range.start.weekday();
    let target_week_number = (range_data.event_range.start.day() - 1) % 7;

    let mut res = Vec::new();

    while monthly_step < range_data.range.end {
        let target_day = monthly_step.weekday().cyclic_time_to(target_weekday) as u8
            + 7 * target_week_number
            + 1;
        if days_in_year_month(monthly_step.year(), monthly_step.month()) >= target_day {
            res.push(TimeRange::new(
                monthly_step.replace_day(target_day).unwrap(),
                monthly_step.replace_day(target_day).unwrap()
                    + (range_data.event_range.end - range_data.event_range.start),
            ));
        };

        monthly_step = add_months(monthly_step, range_data.interval).unwrap();
    }

    res
}

pub fn get_yearly_events_by_day(range_data: EventRangeData) -> Vec<TimeRange> {
    let year_amount = range_data.event_range.end.year() - range_data.range.start.year();
    let offset_from_origin_event = year_amount - year_amount % range_data.interval as i32;
    let mut yearly_step = range_data
        .event_range
        .start
        .replace_day(1)
        .unwrap()
        .replace_year(range_data.event_range.start.year() + offset_from_origin_event)
        .unwrap();

    let mut res = Vec::new();

    while yearly_step < range_data.range.end {
        if days_in_year_month(yearly_step.year(), yearly_step.month())
            >= range_data.event_range.start.day()
        {
            res.push(TimeRange::new(
                yearly_step,
                yearly_step + (range_data.event_range.end - range_data.event_range.start),
            ));
        };

        yearly_step = yearly_step
            .replace_year(yearly_step.year() + range_data.interval as i32)
            .unwrap();
    }

    res
}

pub fn get_yearly_events_by_weekday(
    range_data: EventRangeData,
) -> Result<Vec<TimeRange>, EventError> {
    let (target_weekday, target_week_number, range_base_year) =
        yearly_conv_data(range_data.range.start)?;
    let (.., event_base_year) = yearly_conv_data(range_data.event_range.start)?;
    let year_amount = range_base_year.year() - event_base_year.year();
    let offset_from_origin_event = year_amount - year_amount % range_data.interval as i32;
    let mut yearly_step = range_data
        .event_range
        .start
        .replace_day(1)
        .unwrap()
        .replace_month(Month::January)
        .unwrap()
        .replace_year(range_data.event_range.start.year() + offset_from_origin_event)
        .unwrap();

    let mut res = Vec::new();

    while yearly_step < range_data.range.end {
        let first_monday =
            yearly_step + ((yearly_step.weekday().cyclic_time_to(Weekday::Monday)) as i64).days();

        if weeks_in_year(yearly_step.year()) >= target_week_number {
            let target_time = yearly_step
                + (target_week_number as i64 - first_monday.iso_week() as i64).weeks()
                + (Weekday::Monday.cyclic_time_to(target_weekday) as i64).days();
            res.push(TimeRange::new(
                target_time,
                target_time + (range_data.event_range.end - range_data.event_range.start),
            ));
        };

        yearly_step = yearly_step
            .replace_year(yearly_step.year() + range_data.interval as i32)
            .unwrap();
    }

    Ok(res)
}
