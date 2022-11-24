use chrono::{Datelike, DateTime, Duration, Local, Month, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Weekday};
use serde::{Deserialize, Serialize};
use num_traits::cast::FromPrimitive;

#[cfg(test)]
mod test;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Frequency {
    Daily { time: NaiveTime },
    Weekly{ days: Vec<Weekday>, time: NaiveTime},
    Monthly{ day: u32, time: NaiveTime },
    Yearly{ months: Vec<Month>, day: u32, time: NaiveTime},
    FixedPeriod(FixedPeriodInner),
}

impl Frequency {

    pub fn next(&self, now: &DateTime<Local>, last_unix_time: Option<u64>) -> u64 {
        return match &self {
            Frequency::FixedPeriod(inner) => {
                match last_unix_time {
                    None => now.timestamp() as u64,
                    Some(last) => last + inner.as_seconds(),
                }
            }
            Frequency::Daily { time } => {
                let mut cur = now.clone();
                if &cur.time() > time {
                    cur += Duration::days(1);
                }
                for _ in 0..100 {
                    let result = Local.from_local_datetime(&NaiveDateTime::new(cur.date_naive(), time.clone()));
                    if let Some(x) = result.earliest() {
                        return x.timestamp() as u64;
                    }
                    cur += Duration::days(1)
                }
                eprintln!("Couldn't find any suitable time within 100 days. {:?}", &self);
                return u64::MAX
            },
            Frequency::Weekly { days, time } => {
                if days.is_empty() {
                    eprintln!("No days in {:?}", &self);
                    return u64::MAX;
                }
                let mut cur = now.clone();
                for _ in 0..300 {
                    if !days.contains(&cur.weekday()) {
                        cur += Duration::days(1);
                        continue;
                    }
                    let result = Local.from_local_datetime(&NaiveDateTime::new(cur.date_naive(), time.clone()));
                    if let Some(x) = result.earliest() {
                        return x.timestamp() as u64;
                    }
                    cur += Duration::days(1)
                }
                eprintln!("Couldn't find any suitable day/time within 300 days. {:?}", &self);
                return u64::MAX
            },
            Frequency::Monthly { day, time } => {
                let mut cur = now.clone();
                if cur.time() > *time {
                    cur += Duration::days(1);
                }
                for _ in 0..100 {
                    if &cur.day() != day {
                        cur += Duration::days(1);
                        continue;
                    }
                    let result = Local.from_local_datetime(&NaiveDateTime::new(cur.date_naive(), time.clone()));
                    if let Some(x) = result.earliest() {
                        return x.timestamp() as u64;
                    }
                    cur += Duration::days(1)
                }
                eprintln!("Couldn't find any suitable day/time within 100 days. {:?}. Reached date {:?}.", &self, cur);
                return u64::MAX;
            }
            Frequency::Yearly { months, day, time } => {
                if months.is_empty() {
                    eprintln!("No months in {:?}", &self);
                    return u64::MAX;
                }
                let mut naive_date = now.date_naive();
                if now.time() > *time {
                    naive_date += Duration::days(1);
                }
                if naive_date.day() > *day {
                    naive_date = naive_date.checked_add_months(Months::new(1)).unwrap();
                    naive_date = naive_date.with_day(1).unwrap();
                }
                for _ in 0..100 {
                    if months.contains(&Month::from_u32(naive_date.month()).unwrap()) {
                        // Valid month.
                        if let Some(valid_day) = naive_date.with_day(*day) {
                            if let Some(x) = Local.from_local_datetime(&NaiveDateTime::new(valid_day, time.clone())).earliest() {
                                return x.timestamp() as u64;
                            }
                        }
                    }

                    let mut found_valid_month = false;
                    for month in months {
                        let month_num = month.number_from_month();
                        if month_num > naive_date.month() {
                            if let Some(valid_date) = naive_date.with_month(month.number_from_month()) {
                                found_valid_month = true;
                                naive_date = valid_date;
                                break;
                            }
                        }
                    }
                    if !found_valid_month {
                        naive_date = NaiveDate::from_ymd(naive_date.year() + 1, 1, 1);
                        continue;
                    }
                }
                eprintln!("Couldn't find any suitable day/time within 300 tries. {:?}. Reached date {:?}.", &self, naive_date);
                return u64::MAX
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct FixedPeriodInner {
    hours: u32,
    minutes: u32,
    seconds: u64,
}

impl FixedPeriodInner {
    pub fn new(hours: u32, minutes: u32, seconds: u64) -> Self {
        Self {
            hours,
            minutes,
            seconds,
        }
    }

    pub fn as_seconds(&self) -> u64 {
        let mut seconds = (self.hours as u64) * 60 * 60;
        seconds += self.minutes as u64 * 60;
        seconds += self.seconds;
        seconds
    }
}
