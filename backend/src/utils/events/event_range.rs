use std::cmp::max;

use time::{ext::NumericalDuration, util::weeks_in_year, Month, Weekday};

use super::{
    additions::{
        iso_year_start, next_good_month, next_good_month_by_weekday, AddMonths, CyclicTimeTo,
        TimeStart, TimeTo,
    },
    calculations::EventRangeData,
    errors::EventError,
    models::TimeRange,
};

pub fn get_daily_events(range_data: EventRangeData) -> Vec<TimeRange> {
    let day_amount = (range_data.range.start - range_data.event_range.end).whole_days();
    let offset_from_origin_event = (day_amount - day_amount % range_data.interval as i64).days();

    let mut daily_event = range_data.event_range + offset_from_origin_event;
    let mut res = Vec::new();

    while !daily_event.is_after(&range_data.range) {
        if daily_event.is_overlapping(&range_data.range) {
            res.push(daily_event);
        }

        daily_event += (range_data.interval as i64).days();
    }

    res
}

pub fn get_weekly_events(range_data: EventRangeData, week_map: &str) -> Vec<TimeRange> {
    let week_amount = (range_data.range.start - range_data.event_range.end).whole_weeks();
    let offset_from_origin_event = (week_amount - week_amount % range_data.interval as i64).weeks();
    let offset_from_week_start =
        (time::Weekday::Monday.cyclic_time_to(range_data.event_range.start.weekday()) as i64)
            .days();

    let mut res = Vec::new();

    let weekly_event_start =
        range_data.event_range.start + offset_from_origin_event - offset_from_week_start;
    let mut weekly_event =
        TimeRange::new_relative(weekly_event_start, range_data.event_range.duration());

    while !weekly_event.is_after(&range_data.range) {
        week_map.chars().enumerate().for_each(|(i, elem)| {
            if elem == '1' && (weekly_event + (i as i64).days()).is_overlapping(&range_data.range) {
                res.push(weekly_event + (i as i64).days());
            }
        });

        weekly_event += (range_data.interval as i64).weeks();
    }

    res
}

pub fn get_monthly_events_by_day(range_data: EventRangeData, is_by_day: bool) -> Vec<TimeRange> {
    let (event_end_year, event_end_month, _) = range_data.event_range.end.to_calendar_date();
    let (range_start_year, range_start_month, _) = range_data.range.start.to_calendar_date();

    let month_amount =
        (event_end_year, event_end_month).time_to((range_start_year, range_start_month));

    let offset_from_origin_event = max(
        month_amount - month_amount.rem_euclid(range_data.interval as i32),
        0,
    );

    let month_start = range_data
        .event_range
        .start
        .month_start()
        .add_months(offset_from_origin_event)
        .unwrap();
    let mut monthly_step = range_data.event_range.start;

    while monthly_step < month_start {
        if is_by_day {
            monthly_step = next_good_month(monthly_step, range_data.interval as i32);
        } else {
            monthly_step = next_good_month_by_weekday(monthly_step, range_data.interval as i32);
        }
    }

    let mut res = Vec::new();

    while monthly_step < range_data.range.end {
        let monthly_event =
            TimeRange::new_relative(monthly_step, range_data.event_range.duration());
        if monthly_event.is_overlapping(&range_data.range) {
            res.push(monthly_event);
        };

        if is_by_day {
            monthly_step = next_good_month(monthly_step, range_data.interval as i32);
        } else {
            monthly_step = next_good_month_by_weekday(monthly_step, range_data.interval as i32);
        }
    }

    res
}

pub fn get_yearly_events_by_weekday(
    range_data: EventRangeData,
) -> Result<Vec<TimeRange>, EventError> {
    let (range_base_year, ..) = range_data.range.start.to_iso_week_date();
    let (event_base_year, target_week_number, target_weekday) =
        range_data.event_range.start.to_iso_week_date();
    let year_amount = range_base_year - event_base_year;
    let offset_from_origin_event = max(
        year_amount - year_amount.rem_euclid(range_data.interval as i32),
        0,
    );

    let mut yearly_step = event_base_year + offset_from_origin_event;
    let offset_from_iso_year_start = (target_week_number as i64 - 1).weeks()
        + (Weekday::Monday.cyclic_time_to(target_weekday) as i64).days();
    let mut res = Vec::new();

    while iso_year_start(yearly_step) < range_data.range.end {
        if weeks_in_year(yearly_step) >= target_week_number {
            let target_day = iso_year_start(yearly_step) + offset_from_iso_year_start;

            res.push(TimeRange::new_relative(
                target_day.replace_time(range_data.event_range.start.time()),
                range_data.event_range.duration(),
            ));
        };

        yearly_step += range_data.interval as i32;
    }

    Ok(res)
}

#[cfg(test)]
mod event_range_tests {
    use sqlx::types::Json;
    use time::{macros::datetime, OffsetDateTime};
    use uuid::Uuid;

    use crate::utils::events::models::{Event, EventPart, EventRules, RecurrenceEndsAt, TimeRules};

    use super::*;

    #[test]
    fn daily_range() {
        let event = TimeRange::new(datetime!(2023-02-17 22:45 +1), datetime!(2023-02-18 0:00 +1));
        let rec_rules = EventRules::Daily {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
        };
        let part = EventPart {
            starts_at: datetime!(2023-02-21 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2023-02-27 22:45 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-02-21 22:45 +1),
                    datetime!(2023-02-22 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-02-23 22:45 +1),
                    datetime!(2023-02-24 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-02-25 22:45 +1),
                    datetime!(2023-02-26 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn weekly_range_1() {
        let event = TimeRange::new(datetime!(2023-02-17 22:45 +1), datetime!(2023-02-18 0:00 +1));
        let rec_rules = EventRules::Weekly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            week_map: 54,
        };
        let part = EventPart {
            starts_at: datetime!(2023-02-21 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2023-03-15 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-02-28 22:45 +1),
                    datetime!(2023-03-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-01 22:45 +1),
                    datetime!(2023-03-02 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-03 22:45 +1),
                    datetime!(2023-03-04 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-04 22:45 +1),
                    datetime!(2023-03-05 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-14 22:45 +1),
                    datetime!(2023-03-15 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn weekly_range_2() {
        let event = TimeRange::new(datetime!(2023-02-17 22:45 +1), datetime!(2023-02-18 0:00 +1));
        let rec_rules = EventRules::Weekly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            week_map: 54,
        };
        let part = EventPart {
            starts_at: datetime!(2023-03-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2023-03-22 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-03-01 22:45 +1),
                    datetime!(2023-03-02 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-03 22:45 +1),
                    datetime!(2023-03-04 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-04 22:45 +1),
                    datetime!(2023-03-05 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-14 22:45 +1),
                    datetime!(2023-03-15 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-15 22:45 +1),
                    datetime!(2023-03-16 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-17 22:45 +1),
                    datetime!(2023-03-18 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-18 22:45 +1),
                    datetime!(2023-03-19 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_1() {
        let event = TimeRange::new(datetime!(2023-02-17 22:45 +1), datetime!(2023-02-18 0:00 +1));
        let rec_rules = EventRules::Monthly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            is_by_day: true,
        };
        let part = EventPart {
            starts_at: datetime!(2023-03-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2023-08-17 22:45 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-17 22:45 +1),
                    datetime!(2023-04-18 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-06-17 22:45 +1),
                    datetime!(2023-06-18 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_day_2() {
        let event = TimeRange::new(datetime!(2023-01-31 22:45 +1), datetime!(2023-02-01 0:00 +1));
        let rec_rules = EventRules::Monthly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 1,
            },
            is_by_day: true,
        };
        let part = EventPart {
            starts_at: datetime!(2023-01-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2024-01-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-31 22:45 +1),
                    datetime!(2023-02-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-03-31 22:45 +1),
                    datetime!(2023-04-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-05-31 22:45 +1),
                    datetime!(2023-06-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-07-31 22:45 +1),
                    datetime!(2023-08-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-08-31 22:45 +1),
                    datetime!(2023-09-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-10-31 22:45 +1),
                    datetime!(2023-11-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-12-31 22:45 +1),
                    datetime!(2024-01-01 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_1() {
        let event = TimeRange::new(datetime!(2023-02-17 22:45 +1), datetime!(2023-02-18 0:00 +1));
        let rec_rules = EventRules::Monthly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2023-02-28 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2023-12-19 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-21 22:45 +1),
                    datetime!(2023-04-22 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-06-16 22:45 +1),
                    datetime!(2023-06-17 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-08-18 22:45 +1),
                    datetime!(2023-08-19 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-10-20 22:45 +1),
                    datetime!(2023-10-21 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-12-15 22:45 +1),
                    datetime!(2023-12-16 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn monthly_range_by_weekday_2() {
        let event = TimeRange::new(datetime!(2023-01-29 22:45 +1), datetime!(2023-01-30 0:00 +1));
        let rec_rules = EventRules::Monthly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 1,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2023-02-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2024-01-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-04-30 22:45 +1),
                    datetime!(2023-05-01 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-07-30 22:45 +1),
                    datetime!(2023-07-31 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-10-29 22:45 +1),
                    datetime!(2023-10-30 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2023-12-31 22:45 +1),
                    datetime!(2024-01-01 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_day() {
        let event = TimeRange::new(datetime!(2023-01-29 22:45 +1), datetime!(2023-01-30 0:00 +1));
        let rec_rules = EventRules::Yearly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            is_by_day: true,
        };
        let part = EventPart {
            starts_at: datetime!(2023-01-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-29 22:45 +1),
                    datetime!(2023-01-30 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2025-01-29 22:45 +1),
                    datetime!(2025-01-30 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2027-01-29 22:45 +1),
                    datetime!(2027-01-30 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_1() {
        let event = TimeRange::new(datetime!(2023-01-29 22:45 +1), datetime!(2023-01-30 0:00 +1));
        let rec_rules = EventRules::Yearly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2023-01-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-29 22:45 +1),
                    datetime!(2023-01-30 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2025-01-26 22:45 +1),
                    datetime!(2025-01-27 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2027-01-31 22:45 +1),
                    datetime!(2027-02-01 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_2() {
        let event = TimeRange::new(datetime!(2020-12-28 22:45 +1), datetime!(2020-12-29 0:00 +1));
        let rec_rules = EventRules::Yearly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 1,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2023-01-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2029-01-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![TimeRange::new(
                datetime!(2026-12-28 22:45 +1),
                datetime!(2026-12-29 0:00 +1)
            ),]
        )
    }

    #[test]
    fn yearly_range_by_weekday_3() {
        let event = TimeRange::new(datetime!(2023-01-02 22:45 +1), datetime!(2023-01-03 0:00 +1));
        let rec_rules = EventRules::Yearly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 1,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2023-01-01 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2027-02-01 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![
                TimeRange::new(
                    datetime!(2023-01-02 22:45 +1),
                    datetime!(2023-01-03 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2024-01-01 22:45 +1),
                    datetime!(2024-01-02 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2024-12-30 22:45 +1),
                    datetime!(2024-12-31 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2025-12-29 22:45 +1),
                    datetime!(2025-12-30 0:00 +1)
                ),
                TimeRange::new(
                    datetime!(2027-01-04 22:45 +1),
                    datetime!(2027-01-05 0:00 +1)
                ),
            ]
        )
    }

    #[test]
    fn yearly_range_by_weekday_4() {
        let event = TimeRange::new(datetime!(2023-01-14 22:45 +1), datetime!(2023-01-15 0:00 +1));
        let rec_rules = EventRules::Yearly {
            time_rules: TimeRules {
                ends_at: Some(RecurrenceEndsAt::Count(50)),
                interval: 2,
            },
            is_by_day: false,
        };
        let part = EventPart {
            starts_at: datetime!(2027-01-16 0:00 +1),
            length: Some(RecurrenceEndsAt::Until(datetime!(2027-01-17 0:00 +1))),
        };

        assert_eq!(
            rec_rules.get_event_range(&part, &event).unwrap(),
            vec![TimeRange::new(
                datetime!(2027-01-16 22:45 +1),
                datetime!(2027-01-17 0:00 +1)
            ),]
        )
    }
}
