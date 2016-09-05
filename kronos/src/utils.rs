use chrono::naive::date::NaiveDate as Date;
use chrono::Datelike;
use std::cmp;

// TODO: could be intelligent about the loop
pub fn startof_next_month(d: Date) -> Date {
    let m = d.month();
    let mut next_month = d.clone();
    while m == next_month.month() {
        next_month = next_month.succ();
    }
    next_month
}

// TODO: could be intelligent about the loop
pub fn startof_next_week(d: Date) -> Date {
    let week = d.isoweekdate().1;
    let mut next_week = d.clone();
    while week == next_week.isoweekdate().1 {
        next_week = next_week.succ();
    }
    next_week
}

// TODO: could be intelligent about the loop
pub fn startof_next_year(d: Date) -> Date {
    let y = d.year();
    let mut next_year = d.clone();
    while y == next_year.year() {
        next_year = startof_next_month(next_year);
    }
    next_year
}

pub fn days_in_month(m: u32, y: i32) -> u32 {
    static DIM: [u32;12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    assert!(m > 0 && m <= 12);
    // check when february has 29 days
    if m == 2 && y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { return 29; }
    DIM[(m-1) as usize]
}

pub fn date_add(dt: Date, y: i32, mut m: u32, mut d: u32) -> Date {
    let mut day = dt.day();
    let mut month = dt.month();
    let mut year = dt.year();
    while d > 0 { // shift days
        let diff = cmp::min(days_in_month(month, year)-day, d);
        day += diff;
        d -= diff;
        if d > 0 {
            day = 0;
            month += 1;
            if month > 12 {
                year += 1;
                month = 1;
            }
        }
    }
    while m > 0 {
        let diff = cmp::min(12 - month, m);
        month += diff;
        m -= diff;
        if m > 0 {
            month = 0;
            year += 1;
        }
    }
    year += y;
    day = cmp::min(day, days_in_month(month, year));
    Date::from_ymd(year, month, day)
}

#[cfg(test)]
mod tests {
    use chrono::naive::date::NaiveDate as Date;
    use super::{date_add};
    #[test]
    fn test_dateadd() {
        let dt = Date::from_ymd(2016, 9, 5);
        assert_eq!(date_add(dt, 0, 0, 30), Date::from_ymd(2016, 10, 5));
        assert_eq!(date_add(dt, 0, 0, 1234), Date::from_ymd(2020, 1, 22));
        assert_eq!(date_add(dt, 0, 0, 365), Date::from_ymd(2017, 9, 5));
        assert_eq!(date_add(dt, 0, 0, 2541), Date::from_ymd(2023, 8, 21));
        assert_eq!(date_add(dt, 0, 1, 0), Date::from_ymd(2016, 10, 5));
        let dt = Date::from_ymd(2016, 1, 30);
        assert_eq!(date_add(dt, 0, 1, 0), Date::from_ymd(2016, 2, 29));
        assert_eq!(date_add(dt, 0, 2, 0), Date::from_ymd(2016, 3, 30));
        assert_eq!(date_add(dt, 0, 12, 0), Date::from_ymd(2017, 1, 30));
    }
}
