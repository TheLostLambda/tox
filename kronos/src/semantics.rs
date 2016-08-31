use chrono::{Duration, Datelike};
use chrono::naive::datetime::NaiveDateTime as DateTime;
use chrono::naive::date::NaiveDate as Date;

use utils;

use std::rc::Rc;

// shortcircuit bad sequences
const SEQFUSE: usize = 10000;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Granularity {
    //Second,
    //Minute,
    //Hour,
    //TimeOfDay, // ??
    Day,
    Month,
    //Season,
    //Quarter,
    //Weekend,
    //Week,
    Year,
    //Decade,
    //Century,
    //TempD, // constante dependent duration
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Granularity,
}

// A generator of Ranges
pub type Seq = Rc<Fn(DateTime)->Box<Iterator<Item=Range>>>;

//enum TmDir {
    //Future,
    //Past,
//}

//struct RefTime {
    //start: DateTime,
    //dir: TmDir,
//}

// X: Sequences generate Ranges that have ENDtime after reference-time
// see duckling http://goo.gl/gxU1Jo

pub fn day_of_week(dow: usize) -> Seq {
    Rc::new(move |reftime| {
        // given X-precondition: (endtime = tm + 1 day) > reftime
        let mut tm = reftime.date();
        while tm.weekday().num_days_from_sunday() != (dow as u32) {
            tm = tm.succ();
        }
        let tm = tm.and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + Duration::days(x * 7),
                end: tm + Duration::days(x * 7 + 1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn month_of_year(moy: usize) -> Seq {
    Rc::new(move |reftime| {
        // X-precondition: (endtime = end-of-month(tm)) > reftime
        let mut tm = Date::from_ymd(reftime.year(), reftime.month(), 1);
        Box::new((0..).map(move |_| {
            while tm.month() != (moy as u32) {
                tm = utils::startof_next_month(tm);
            }
            let t0 = tm;
            tm = utils::startof_next_month(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Month
            }
        }))
    })
}

pub fn day() -> Seq {
    Rc::new(|reftime| {
        // given X-precondition: (endtime = tm + 1 day) > reftime
        let tm = reftime.date().and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + Duration::days(x),
                end: tm + Duration::days(x+1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn month() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 month) > reftime
        let mut tm = Date::from_ymd(reftime.year(), reftime.month(), 1);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            tm = utils::startof_next_month(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Month
            }
        }))
    })
}

pub fn year() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 year) > reftime
        let mut tm = Date::from_ymd(reftime.year(), 1, 1);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            tm = utils::startof_next_year(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Year
            }
        }))
    })
}

fn startof_seq(s: Seq, reftime: DateTime) -> DateTime {
    s(reftime).next().unwrap().start
}

pub fn nth(n: usize, win: Seq, within: Seq) -> Seq {
    // For a predictable outcome you probably want aligned sequences
    // 1. take an instance of <within>
    // 2. cycle to the n-th instance if <win> within <within>
    {   // assert win-item.duration < within-item.duration
        let testtm = Date::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        let a = win(testtm).next().unwrap();
        let b = within(testtm).next().unwrap();
        assert!((a.end - a.start) <= (b.end - b.start));
    }
    Rc::new(move |reftime| {
        let win = win.clone();
        let align = within(reftime).next().unwrap().start;
        Box::new(within(reftime)
                    .take(SEQFUSE) // TODO: panic ? looks like wrong place
                    .filter_map(move |outer| {
            // we restart <win> each time instead of continuing because we
            // could have overflowed the outer interval and we cant miss items
            let x = win(align).skip_while(|inner| inner.start < outer.start)
                              .nth(n - 1).unwrap();
            // if there's no n-th item in this <within> instance, try next
            match x.start >= outer.start && x.end <= outer.end {
                true => Some(x),
                false => None
            }
        }).skip_while(move |range| range.end < reftime)) // overcome alignment
    })
}

pub fn intersect(a: Seq, b: Seq) -> Seq {
    Rc::new(move |tm| {
        // TODO: this looks ugly
        let x = a(tm).next().unwrap();
        let y = b(tm).next().unwrap();
        let (a, b) = match (y.end - y.start) < (x.end - x.start) {
            true => (b.clone(), a.clone()),
            false => (a.clone(), b.clone())
        };
        // we need a clone of win each time instead of continuing because we could have
        // overflowed the outer <within> interval and we don't want to miss items
        Box::new(b(tm).flat_map(move |outer| {
            a(tm).skip_while(move |inner| inner.start < outer.start)
                 .take_while(move |inner| inner.end <= outer.end)
        }))
    })
}

//fn fn take_n() -> Seq {} // or first 3 weeks ?