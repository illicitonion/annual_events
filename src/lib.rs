use chrono::{Datelike, Month, NaiveDate, Utc, Weekday};
use sha2::{Digest, Sha256};

use std::io::Write;
use std::num::NonZeroU8;

pub fn make_calendar<W: Write>(mut out: W) -> std::io::Result<()> {
    let now = Utc::now();
    let current_year = now.year();
    let now = now.format("%Y%m%dT%H%M%SZ");

    writeln!(out, "BEGIN:VCALENDAR")?;
    writeln!(out, "PRODID:-//illicitonion//Annual events calendar//EN")?;
    writeln!(out, "VERSION:2.0")?;

    for year in (current_year - 10)..(current_year + 10) {
        for (summary, datespec) in gen::events() {
            let date = datespec.to_date(year);
            let dtstart = format!("{:04}{:02}{:02}", date.year(), date.month(), date.day());

            let uid = hash_event(year, &summary);

            writeln!(out, "BEGIN:VEVENT")?;
            writeln!(out, "SUMMARY:{summary}", summary = summary)?;
            writeln!(out, "DTSTART:{dtstart}", dtstart = dtstart)?;
            writeln!(out, "UID:{uid}", uid = uid)?;
            writeln!(out, "DTSTAMP:{now}", now = now)?;
            writeln!(out, "CREATED:{now}", now = now)?;
            writeln!(out, "LAST-MODIFIED:{now}", now = now)?;
            writeln!(out, "SEQUENCE:0")?;
            writeln!(out, "END:VEVENT")?;
        }
    }

    writeln!(out, "END:VCALENDAR")?;

    Ok(())
}

type DayOfMonth = u32;

enum Event {
    FixedDate(Month, DayOfMonth),
    FixedDayOfMonth(Month, Weekday, WeekInMonth),
}

#[allow(dead_code)]
enum WeekInMonth {
    First,
    Second,
    Third,
    Fourth,
    Last,
}

enum Direction {
    Forwards(NonZeroU8),
    Backwards(NonZeroU8),
}

impl WeekInMonth {
    fn to_direction(&self) -> Direction {
        unsafe {
            match self {
                Self::First => Direction::Forwards(NonZeroU8::new_unchecked(1)),
                Self::Second => Direction::Forwards(NonZeroU8::new_unchecked(2)),
                Self::Third => Direction::Forwards(NonZeroU8::new_unchecked(3)),
                Self::Fourth => Direction::Forwards(NonZeroU8::new_unchecked(4)),
                Self::Last => Direction::Backwards(NonZeroU8::new_unchecked(1)),
            }
        }
    }
}

impl Event {
    fn to_date(&self, year: i32) -> NaiveDate {
        match self {
            Event::FixedDate(month, day_of_month) => {
                NaiveDate::from_ymd(year, month.number_from_month(), *day_of_month)
            }
            Event::FixedDayOfMonth(month, weekday, week_in_month) => {
                let target = week_in_month.to_direction();
                let (mut count, mut possible_date, next): (
                    _,
                    _,
                    Box<dyn Fn(NaiveDate) -> NaiveDate>,
                ) = match target {
                    Direction::Forwards(count) => (
                        count.get(),
                        NaiveDate::from_ymd(year, month.number_from_month(), 1),
                        Box::new(|d| d.succ()),
                    ),
                    Direction::Backwards(count) => {
                        let next_month = month.succ();
                        let year_of_next_month =
                            if next_month.number_from_month() < month.number_from_month() {
                                year + 1
                            } else {
                                year
                            };
                        let possible_date = NaiveDate::from_ymd(
                            year_of_next_month,
                            next_month.number_from_month(),
                            1,
                        )
                        .pred();
                        (count.get(), possible_date, Box::new(|d| d.pred()))
                    }
                };
                loop {
                    if possible_date.weekday() == *weekday {
                        count -= 1;
                        if count == 0 {
                            return possible_date;
                        }
                    }
                    possible_date = next(possible_date);
                }
            }
        }
    }
}

fn hash_event(year: i32, summary: &str) -> String {
    use std::fmt::Write;

    let mut hasher = Sha256::new();
    hasher.update(year.to_be_bytes());
    hasher.update(summary.as_bytes());

    let hash = hasher.finalize();

    let mut uid = String::with_capacity(2 * hash.len());
    for byte in hash {
        write!(uid, "{:02x}", byte).unwrap();
    }

    uid
}

mod gen {
    use crate::*;
    include!(concat!(env!("OUT_DIR"), "/events.rs"));
}
