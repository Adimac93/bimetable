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
        (day_amount - day_amount % range_data.interval as i64).days()
        + (range_data.interval as i64).days();

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
        let week_start = dbg!(weekly_end_step
            - (time::Weekday::Monday.cyclic_time_to(weekly_end_step.weekday()) as i64).days());
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

#[cfg(test)]
mod test_tests {
    use sqlx::types::Json;
    use time::{OffsetDateTime, macros::datetime};
    use uuid::Uuid;

    use crate::utils::events::models::{RecurrenceEndsAt, EventRules, EventPart, Event, TimeRules};

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
    fn daily_range() {
        let data = create_test_event_part(
            datetime!(2023-02-17 22:45 +1),
            datetime!(2023-02-18 0:00 +1),
            EventRules::Daily {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
            },
            datetime!(2023-02-21 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2023-02-27 22:45 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-02-21 22:45 +1), datetime!(2023-02-22 0:00 +1)),
                TimeRange::new(datetime!(2023-02-23 22:45 +1), datetime!(2023-02-24 0:00 +1)),
                TimeRange::new(datetime!(2023-02-25 22:45 +1), datetime!(2023-02-26 0:00 +1)),
            ]
        )
    }

    #[test]
    fn weekly_range_1() {
        let data = create_test_event_part(
            datetime!(2023-02-17 22:45 +1),
            datetime!(2023-02-18 0:00 +1),
            EventRules::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                week_map: 54,
            },
            datetime!(2023-02-21 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2023-03-15 0:00 +1)),
        );

        /*
        -------
        ----xx-
        -|-----
        -x
          x-xx-
        -------
        -x|-xx-
        */

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-02-28 22:45 +1), datetime!(2023-03-01 0:00 +1)),
                TimeRange::new(datetime!(2023-03-01 22:45 +1), datetime!(2023-03-02 0:00 +1)),
                TimeRange::new(datetime!(2023-03-03 22:45 +1), datetime!(2023-03-04 0:00 +1)),
                TimeRange::new(datetime!(2023-03-04 22:45 +1), datetime!(2023-03-05 0:00 +1)),
                TimeRange::new(datetime!(2023-03-14 22:45 +1), datetime!(2023-03-15 0:00 +1)),
            ]
        )
    }

    #[test]
    fn weekly_range_2() {
        let data = create_test_event_part(
            datetime!(2023-02-17 22:45 +1),
            datetime!(2023-02-18 0:00 +1),
            EventRules::Weekly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                week_map: 54,
            },
            datetime!(2023-03-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2023-03-22 0:00 +1)),
        );

        /*
        -------
        ----xx-
        -|-----
        -x
          x-xx-
        -------
        -xx-xx-
        --|----
        */

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-03-01 22:45 +1), datetime!(2023-03-02 0:00 +1)),
                TimeRange::new(datetime!(2023-03-03 22:45 +1), datetime!(2023-03-04 0:00 +1)),
                TimeRange::new(datetime!(2023-03-04 22:45 +1), datetime!(2023-03-05 0:00 +1)),
                TimeRange::new(datetime!(2023-03-14 22:45 +1), datetime!(2023-03-15 0:00 +1)),
                TimeRange::new(datetime!(2023-03-15 22:45 +1), datetime!(2023-03-16 0:00 +1)),
                TimeRange::new(datetime!(2023-03-17 22:45 +1), datetime!(2023-03-18 0:00 +1)),
                TimeRange::new(datetime!(2023-03-18 22:45 +1), datetime!(2023-03-19 0:00 +1)),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_1() {
        let data = create_test_event_part(
            datetime!(2023-02-17 22:45 +1),
            datetime!(2023-02-18 0:00 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                is_by_day: true,
            },
            datetime!(2023-03-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2023-08-17 22:45 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-04-17 22:45 +1), datetime!(2023-04-18 0:00 +1)),
                TimeRange::new(datetime!(2023-06-17 22:45 +1), datetime!(2023-06-18 0:00 +1)),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_2() {
        let data = create_test_event_part(
            datetime!(2023-01-31 22:45 +1),
            datetime!(2023-02-01 0:00 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 1,
                },
                is_by_day: true,
            },
            datetime!(2023-01-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2024-01-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-01-31 22:45 +1), datetime!(2023-02-01 0:00 +1)),
                TimeRange::new(datetime!(2023-03-31 22:45 +1), datetime!(2023-04-01 0:00 +1)),
                TimeRange::new(datetime!(2023-05-31 22:45 +1), datetime!(2023-06-01 0:00 +1)),
                TimeRange::new(datetime!(2023-07-31 22:45 +1), datetime!(2023-08-01 0:00 +1)),
                TimeRange::new(datetime!(2023-08-31 22:45 +1), datetime!(2023-09-01 0:00 +1)),
                TimeRange::new(datetime!(2023-10-31 22:45 +1), datetime!(2023-11-01 0:00 +1)),
                TimeRange::new(datetime!(2023-12-31 22:45 +1), datetime!(2024-01-01 0:00 +1)),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_1() {
        let data = create_test_event_part(
            datetime!(2023-02-17 22:45 +1),
            datetime!(2023-02-18 0:00 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                is_by_day: false,
            },
            datetime!(2023-02-28 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2023-12-19 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-04-21 22:45 +1), datetime!(2023-04-22 0:00 +1)),
                TimeRange::new(datetime!(2023-06-16 22:45 +1), datetime!(2023-06-17 0:00 +1)),
                TimeRange::new(datetime!(2023-08-18 22:45 +1), datetime!(2023-08-18 0:00 +1)),
                TimeRange::new(datetime!(2023-10-20 22:45 +1), datetime!(2023-10-21 0:00 +1)),
                TimeRange::new(datetime!(2023-12-15 22:45 +1), datetime!(2023-12-16 0:00 +1)),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_2() {
        let data = create_test_event_part(
            datetime!(2023-01-29 22:45 +1),
            datetime!(2023-01-29 0:00 +1),
            EventRules::Monthly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2023-02-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2024-01-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-04-30 22:45 +1), datetime!(2023-05-01 0:00 +1)),
                TimeRange::new(datetime!(2023-07-30 22:45 +1), datetime!(2023-07-31 0:00 +1)),
                TimeRange::new(datetime!(2023-10-29 22:45 +1), datetime!(2023-10-30 0:00 +1)),
                TimeRange::new(datetime!(2023-12-31 22:45 +1), datetime!(2024-01-01 0:00 +1)),
            ]
        )
    }

    #[test]
    fn yearly_range_by_day() {
        let data = create_test_event_part(
            datetime!(2023-01-29 22:45 +1),
            datetime!(2023-01-30 0:00 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                is_by_day: true,
            },
            datetime!(2023-01-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-01-29 22:45 +1), datetime!(2023-01-30 0:00 +1)),
                TimeRange::new(datetime!(2025-01-29 22:45 +1), datetime!(2025-01-30 0:00 +1)),
                TimeRange::new(datetime!(2027-01-29 22:45 +1), datetime!(2027-01-30 0:00 +1)),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_1() {
        let data = create_test_event_part(
            datetime!(2023-01-29 22:45 +1),
            datetime!(2023-01-30 0:00 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                is_by_day: false,
            },
            datetime!(2023-01-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-01-29 22:45 +1), datetime!(2023-01-30 0:00 +1)),
                TimeRange::new(datetime!(2025-01-26 22:45 +1), datetime!(2025-01-27 0:00 +1)),
                TimeRange::new(datetime!(2027-01-31 22:45 +1), datetime!(2027-02-01 0:00 +1)),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_2() {
        let data = create_test_event_part(
            datetime!(2020-12-28 22:45 +1),
            datetime!(2020-12-29 0:00 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2023-01-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2026-12-28 22:45 +1), datetime!(2026-12-29 0:00 +1)),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_3() {
        let data = create_test_event_part(
            datetime!(2023-01-02 22:45 +1),
            datetime!(2023-01-03 0:00 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 1,
                },
                is_by_day: false,
            },
            datetime!(2023-01-01 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2027-02-01 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2023-01-02 22:45 +1), datetime!(2023-01-03 0:00 +1)),
                TimeRange::new(datetime!(2024-01-01 22:45 +1), datetime!(2024-01-02 0:00 +1)),
                TimeRange::new(datetime!(2024-12-30 22:45 +1), datetime!(2024-12-31 0:00 +1)),
                TimeRange::new(datetime!(2025-12-29 22:45 +1), datetime!(2025-12-30 0:00 +1)),
                TimeRange::new(datetime!(2027-01-04 22:45 +1), datetime!(2027-01-05 0:00 +1)),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_4() {
        let data = create_test_event_part(
            datetime!(2023-01-14 22:45 +1),
            datetime!(2023-01-15 0:00 +1),
            EventRules::Yearly {
                time_rules: TimeRules {
                    ends_at: Some(RecurrenceEndsAt::Count(50)),
                    interval: 2,
                },
                is_by_day: false,
            },
            datetime!(2027-01-16 0:00 +1),
            RecurrenceEndsAt::Until(datetime!(2027-01-17 0:00 +1)),
        );

        assert_eq!(
            data.get_event_range().unwrap(),
            vec![
                TimeRange::new(datetime!(2027-01-16 22:45 +1), datetime!(2027-01-17 0:00 +1)),
            ]
        )
    }
}
