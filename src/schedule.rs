//! Service-day resolution.
//!
//! GTFS splits a schedule into *service days* rather than calendar days. A
//! trip that leaves at 00:30 belongs to the previous service day, and a trip
//! that leaves at 23:59 belongs to the current one. Answering "what is
//! running tonight" therefore needs two pieces of information: which service
//! ids are active on a given date, and which service day "now" falls in.

use std::collections::BTreeSet;

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

use crate::attributes::calendar_dates::ExceptionType;
use crate::feed::Feed;

/// The timezone BART operates in.
pub const PACIFIC: Tz = chrono_tz::America::Los_Angeles;

/// The hour, in Pacific time, before which we consider it to be "the next
/// service day" rather than the current one.
///
/// BART's last trains leave around 00:30–01:00, so a 02:00 query is really
/// asking about last night's service.
pub const SERVICE_DAY_CUTOFF_HOUR: u32 = 4;

/// The service ids that are in effect on `date`.
///
/// The rules are:
///
/// 1. A service in `calendar.txt` is active if the date falls within its
///    start and end dates and its weekly pattern includes that weekday.
/// 2. A `calendar_dates.txt` row with `exception_type = 1` **adds** a service
///    on that date.
/// 3. A `calendar_dates.txt` row with `exception_type = 2` **removes** a
///    service on that date.
///
/// A removal wins over an addition if both name the same service.
pub fn service_ids_for(feed: &Feed, date: NaiveDate) -> BTreeSet<String> {
    let mut active: BTreeSet<String> = feed
        .calendar
        .iter()
        .filter(|cal| cal.runs_on(date))
        .map(|cal| cal.service_id.clone())
        .collect();

    for exception in feed.calendar_dates.iter().filter(|e| e.date == Some(date)) {
        match exception.exception_type {
            ExceptionType::Added => {
                active.insert(exception.service_id.clone());
            }
            ExceptionType::Removed => {
                active.remove(&exception.service_id);
            }
        }
    }

    active
}

/// The service date that `now` falls in, in Pacific time.
///
/// Between midnight and [`SERVICE_DAY_CUTOFF_HOUR`] Pacific, the service
/// date is the *previous* calendar date.
pub fn current_service_date(now: DateTime<Tz>) -> NaiveDate {
    let local = now.with_timezone(&PACIFIC);
    if local.hour() < SERVICE_DAY_CUTOFF_HOUR {
        local.date_naive() - Duration::days(1)
    } else {
        local.date_naive()
    }
}

/// The current time in Pacific time.
pub fn now_pacific() -> DateTime<Tz> {
    Utc::now().with_timezone(&PACIFIC)
}

/// The service date and time-of-day for "now".
pub fn now_service_day() -> (NaiveDate, u32) {
    let now = now_pacific();
    let date = current_service_date(now);
    let seconds = now.hour() * 3600 + now.minute() * 60 + now.second();
    (date, seconds)
}

/// Format a time as the GTFS feed writes it, e.g. `25:30:00`.
///
/// Times past 24 hours are preserved, because that is how the feed expresses
/// them and how the arithmetic works.
pub fn fmt_gtfs_time(seconds: u32) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}

/// Format a time for a human, e.g. `01:30:00 (+1d)`.
pub fn fmt_human_time(seconds: u32) -> String {
    let day = seconds / 86_400;
    let rest = seconds % 86_400;
    let base = format!(
        "{:02}:{:02}:{:02}",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    );
    if day == 0 {
        base
    } else {
        format!("{base} (+{day}d)")
    }
}

/// Format a time as `HH:MM`, e.g. `01:30`.
pub fn fmt_short_time(seconds: u32) -> String {
    let rest = seconds % 86_400;
    format!("{:02}:{:02}", rest / 3600, (rest % 3600) / 60)
}

/// The start of the Unix epoch expressed in the GTFS feed's date format.
pub fn epoch_to_gtfs_date(epoch: i64) -> Option<NaiveDate> {
    Utc.timestamp_opt(epoch, 0)
        .single()
        .map(|dt| dt.with_timezone(&PACIFIC).date_naive())
}

/// Whether a date is the last day of a month.
pub fn is_month_end(date: NaiveDate) -> bool {
    let (year, month) = (date.year(), date.month());
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.map(|n| n - Duration::days(1) == date).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pacific(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Tz> {
        PACIFIC
            .with_ymd_and_hms(y, m, d, h, min, 0)
            .single()
            .expect("valid local time")
    }

    #[test]
    fn late_night_belongs_to_the_previous_service_day() {
        let date = current_service_date(pacific(2026, 10, 19, 2, 0));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 10, 18).unwrap());
    }

    #[test]
    fn just_after_the_cutoff_is_the_current_day() {
        let date = current_service_date(pacific(2026, 10, 19, 4, 1));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 10, 19).unwrap());
    }

    #[test]
    fn exactly_at_the_cutoff_is_the_current_day() {
        // The service day rolls over at 04:00 Pacific, so [00:00, 04:00) is
        // the previous service day and 04:00 onwards is the current one.
        let date = current_service_date(pacific(2026, 10, 19, 4, 0));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 10, 19).unwrap());
    }

    #[test]
    fn just_before_the_cutoff_is_the_previous_day() {
        let date = current_service_date(pacific(2026, 10, 19, 3, 59));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 10, 18).unwrap());
    }

    #[test]
    fn evening_is_the_current_day() {
        let date = current_service_date(pacific(2026, 10, 19, 23, 59));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 10, 19).unwrap());
    }

    #[test]
    fn formats_times_past_midnight() {
        assert_eq!(fmt_gtfs_time(0), "00:00:00");
        assert_eq!(fmt_gtfs_time(91_800), "25:30:00");
        assert_eq!(fmt_human_time(91_800), "01:30:00 (+1d)");
        assert_eq!(fmt_human_time(3600), "01:00:00");
        assert_eq!(fmt_short_time(91_800), "01:30");
    }
}
