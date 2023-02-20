use time::ext::NumericalDuration;

use super::{calculations::EventRangeData, models::TimeRange, additions::days_between_two_weekdays};

pub fn get_daily_events(range_data: EventRangeData) -> Vec<TimeRange> {
    let day_amount = (range_data.part_starts_at - range_data.event_ends_at).whole_days();
    let mut offset_from_origin_event =
        (day_amount - day_amount % range_data.interval as i64).days();

    let mut res = Vec::new();

    while range_data.event_starts_at + offset_from_origin_event < range_data.part_ends_at {
        res.push(TimeRange {
            starts_at: range_data.event_starts_at + offset_from_origin_event,
            ends_at: range_data.event_ends_at + offset_from_origin_event,
        });

        offset_from_origin_event += (range_data.interval as i64).days();
    }

    res
}

pub fn get_weekly_events(range_data: EventRangeData, week_map: &str) -> Vec<TimeRange> {
    let week_amount = (range_data.part_starts_at - range_data.event_ends_at).whole_weeks();
    let mut offset_from_origin_event = (week_amount - week_amount % range_data.interval as i64).weeks();

    let mut res = Vec::new();

    while range_data.event_starts_at + offset_from_origin_event < range_data.part_ends_at {
        let weekly_start_step = range_data.event_starts_at + offset_from_origin_event;
        let weekly_end_step = range_data.event_ends_at + offset_from_origin_event;
        let week_start = weekly_end_step - (days_between_two_weekdays(time::Weekday::Monday, weekly_end_step.weekday()) as i64).days();
        for i in 0..7 {
            if &week_map[i..=i] == "1" && week_start + (i as i64).days() > range_data.part_starts_at {
                res.push(TimeRange {
                    starts_at: weekly_start_step + (i as i64).days(),
                    ends_at: weekly_end_step + (i as i64).days(),
                });
            };
        }

        offset_from_origin_event += (range_data.interval as i64).weeks();
    }

    res
}
