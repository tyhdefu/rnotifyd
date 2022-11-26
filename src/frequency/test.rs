use chrono::Utc;
use crate::frequency::Frequency::*;
use crate::frequency::*;

fn next_datetime(freq: &Frequency, now: &DateTime<Local>, last_unix_time: Option<u64>) -> DateTime<Utc> {
    let pred = freq.next(&now, last_unix_time);
    Utc.timestamp(pred as i64, 0)
}

fn ymd(s: &str) -> NaiveDate {
    s.parse().unwrap()
}

fn hms(s: &str) -> NaiveTime {
    s.parse().unwrap()
}

////////////////////// Fixed Period //////////////////////

#[test]
fn fixed_period_no_prev() {
    let fixed_period = FixedPeriod(FixedPeriodInner { hours: 3, minutes: 13, seconds: 3});
    let date = ymd("2022-03-17");
    let start = Local.from_local_datetime(&date.and_time(hms("05:46:13"))).unwrap();

    let pred = next_datetime(&fixed_period, &start, None);

    assert_eq!(pred, Utc.from_utc_datetime(&start.naive_utc()));
}

#[test]
fn fixed_period() {
    let fixed_period = FixedPeriod(FixedPeriodInner { hours: 3, minutes: 13, seconds: 3});
    let date = ymd("2022-03-17");
    let start = Local.from_local_datetime(&date.and_time(hms("05:46:13"))).unwrap();
    let end   = Local.from_local_datetime(&date.and_time(hms("08:59:16"))).unwrap();

    let pred = next_datetime(&fixed_period, &start, Some(start.timestamp() as u64));

    assert_eq!(pred, Utc.from_utc_datetime(&end.naive_utc()));
}


////////////////////// Daily //////////////////////

#[test]
fn daily_same_day() {
    let start_date = ymd("2021-07-17");

    let daily = Daily { time: hms("07:01:30") };

    let start = Local.from_local_datetime(&start_date.and_time(hms("04:47:14"))).unwrap();
    let next = Local.from_local_datetime(&start_date.and_time(hms("07:01:30"))).unwrap();
    let next_predicted = daily.next(&start, None);

    let utc_datetime_pred = Utc.timestamp(next_predicted as i64, 0);
    assert_eq!(utc_datetime_pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn daily_next_day() {
    let start = Local.from_local_datetime(&ymd("2021-03-12").and_time(hms("16:55:19"))).unwrap();

    let daily = Daily { time: hms("12:02:16") };

    let next = Local.from_local_datetime(&ymd("2021-03-13").and_time(hms("12:02:16"))).unwrap();

    let pred = next_datetime(&daily, &start, None);

    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()))
}


////////////////////// Weekly //////////////////////

#[test]
fn weekly_same_day() {
    let start = Local.from_local_datetime(&ymd("2021-03-12").and_time(hms("08:55:19"))).unwrap();

    let weekly = Weekly { days: vec![Weekday::Fri], time: hms("12:00:00") };

    let next = Local.from_local_datetime(&ymd("2021-03-12").and_time(hms("12:00: 00"))).unwrap();

    let pred = next_datetime(&weekly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn weekly_same_week_pre() {
    let start = Local.from_local_datetime(&ymd("2021-03-10").and_time(hms("08:30:19"))).unwrap();

    let weekly = Weekly { days: vec![Weekday::Fri], time: hms("12:00:00") };

    let next = Local.from_local_datetime(&ymd("2021-03-12").and_time(hms("12:00: 00"))).unwrap();

    let pred = next_datetime(&weekly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn weekly_post_weekday() {
    let start = Local.from_local_datetime(&ymd("2021-03-06").and_time(hms("08:30:19"))).unwrap();

    let weekly = Weekly { days: vec![Weekday::Fri], time: hms("12:00:00") };

    let next = Local.from_local_datetime(&ymd("2021-03-12").and_time(hms("12:00:00"))).unwrap();

    let pred = next_datetime(&weekly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}


////////////////////// Monthly //////////////////////

#[test]
fn monthly_same_day_pre() {
    let start = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("03:22:30"))).unwrap();

    let monthly = Monthly {
        day: 15,
        time: hms("13:00:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("13:00:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn monthly_same_day_post() {
    let start = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("17:22:30"))).unwrap();

    let monthly = Monthly {
        day: 15,
        time: hms("13:00:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-08-15").and_time(hms("13:00:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn monthly_same_month_pre() {
    let start = Local.from_local_datetime(&ymd("2020-07-13").and_time(hms("03:22:30"))).unwrap();

    let monthly = Monthly {
        day: 15,
        time: hms("13:00:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("13:00:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}

#[test]
fn monthly_same_month_post() {
    let start = Local.from_local_datetime(&ymd("2020-07-16").and_time(hms("03:22:30"))).unwrap();

    let monthly = Monthly {
        day: 15,
        time: hms("13:00:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-08-15").and_time(hms("13:00:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()));
}


////////////////////// Yearly //////////////////////

#[test]
fn yearly_same_day() {
    let start = Local.from_local_datetime(&ymd("2020-07-05").and_time(hms("15:04:24"))).unwrap();

    let monthly = Yearly {
        months: vec![Month::July],
        day: 05,
        time: hms("15:30:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-07-05").and_time(hms("15:30:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()))
}

#[test]
fn yearly_same_month_pre() {
    let start = Local.from_local_datetime(&ymd("2020-07-05").and_time(hms("15:04:24"))).unwrap();

    let monthly = Yearly {
        months: vec![Month::July],
        day: 14,
        time: hms("15:30:00"),
    };

    let next = Local.from_local_datetime(&ymd("2020-07-14").and_time(hms("15:30:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()))
}

#[test]
fn yearly_same_month_post() {
    let start = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("15:04:24"))).unwrap();

    let monthly = Yearly {
        months: vec![Month::July],
        day: 14,
        time: hms("15:30:00"),
    };

    let next = Local.from_local_datetime(&ymd("2021-07-14").and_time(hms("15:30:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()))
}

#[test]
fn yearly_post_day() {
    let start = Local.from_local_datetime(&ymd("2020-07-15").and_time(hms("15:04:24"))).unwrap();

    let monthly = Yearly {
        months: vec![Month::July],
        day: 14,
        time: hms("15:30:00"),
    };

    let next = Local.from_local_datetime(&ymd("2021-07-14").and_time(hms("15:30:00"))).unwrap();

    let pred = next_datetime(&monthly, &start, None);
    assert_eq!(pred, Utc.from_utc_datetime(&next.naive_utc()))
}