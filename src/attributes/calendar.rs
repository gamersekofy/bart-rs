//! `calendar.txt` — the recurring service patterns.

use std::path::Path;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::attributes::calendar_dates::CalendarDate;
use crate::error::Result;
use crate::helper::{de_bool_01, de_gtfs_date, read_table};

/// The window during which the given service patterns actually run.
///
/// This is the earliest start and the latest end across every
/// `calendar.txt` row, widened by any single-date exception in
/// `calendar_dates.txt`.
///
/// It is *not* the same thing as `feed_info.txt`'s `feed_start_date` and
/// `feed_end_date`: BART publishes a new feed without updating those, so
/// they describe the previous release. See `docs/03-data-sources.md`.
pub fn coverage(
    calendar: &[Calendar],
    calendar_dates: &[CalendarDate],
) -> Option<(NaiveDate, NaiveDate)> {
    let mut start: Option<NaiveDate> = None;
    let mut end: Option<NaiveDate> = None;

    let mut widen = |date: NaiveDate| {
        start = Some(match start {
            Some(current) => current.min(date),
            None => date,
        });
        end = Some(match end {
            Some(current) => current.max(date),
            None => date,
        });
    };

    for row in calendar {
        if let Some(date) = row.start_date {
            widen(date);
        }
        if let Some(date) = row.end_date {
            widen(date);
        }
    }

    for row in calendar_dates {
        if let Some(date) = row.date {
            widen(date);
        }
    }

    Some((start?, end?))
}

/// A recurring service pattern.
///
/// Each row says "this service id runs on these days of the week between
/// these dates". `calendar_dates.txt` then adds and removes individual
/// exceptions to it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Calendar {
    /// The service id, referenced by `trips.service_id`.
    #[serde(rename = "service_id")]
    pub service_id: String,
    /// Whether the service runs on Mondays.
    #[serde(rename = "monday", deserialize_with = "de_bool_01")]
    pub monday: bool,
    /// Whether the service runs on Tuesdays.
    #[serde(rename = "tuesday", deserialize_with = "de_bool_01")]
    pub tuesday: bool,
    /// Whether the service runs on Wednesdays.
    #[serde(rename = "wednesday", deserialize_with = "de_bool_01")]
    pub wednesday: bool,
    /// Whether the service runs on Thursdays.
    #[serde(rename = "thursday", deserialize_with = "de_bool_01")]
    pub thursday: bool,
    /// Whether the service runs on Fridays.
    #[serde(rename = "friday", deserialize_with = "de_bool_01")]
    pub friday: bool,
    /// Whether the service runs on Saturdays.
    #[serde(rename = "saturday", deserialize_with = "de_bool_01")]
    pub saturday: bool,
    /// Whether the service runs on Sundays.
    #[serde(rename = "sunday", deserialize_with = "de_bool_01")]
    pub sunday: bool,
    /// The first date the service is in effect.
    #[serde(rename = "start_date", deserialize_with = "de_gtfs_date")]
    pub start_date: Option<NaiveDate>,
    /// The last date the service is in effect.
    #[serde(rename = "end_date", deserialize_with = "de_gtfs_date")]
    pub end_date: Option<NaiveDate>,
}

impl Calendar {
    /// Read `calendar.txt` from `path`.
    pub fn parse_calendar_file(path: &Path) -> Result<Vec<Calendar>> {
        read_table(path)
    }

    /// Whether this service's weekly pattern includes `date`.
    ///
    /// The date must fall within the service's start and end dates.
    pub fn runs_on(&self, date: NaiveDate) -> bool {
        if let Some(start) = self.start_date
            && date < start
        {
            return false;
        }
        if let Some(end) = self.end_date
            && date > end
        {
            return false;
        }

        // `weekday()` returns Monday = 0, matching the column order used by
        // the GTFS spec.
        match date.weekday() {
            chrono::Weekday::Mon => self.monday,
            chrono::Weekday::Tue => self.tuesday,
            chrono::Weekday::Wed => self.wednesday,
            chrono::Weekday::Thu => self.thursday,
            chrono::Weekday::Fri => self.friday,
            chrono::Weekday::Sat => self.saturday,
            chrono::Weekday::Sun => self.sunday,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    fn weekly() -> Calendar {
        Calendar {
            service_id: "s1".into(),
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31),
        }
    }

    #[test]
    fn weekday_pattern_matches_only_weekdays() {
        let cal = weekly();
        let monday = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
        assert_eq!(monday.weekday(), Weekday::Mon);
        assert!(cal.runs_on(monday));
        assert!(!cal.runs_on(sunday));
    }

    #[test]
    fn dates_outside_the_window_are_excluded() {
        let mut cal = weekly();
        cal.start_date = NaiveDate::from_ymd_opt(2026, 10, 1);
        cal.end_date = NaiveDate::from_ymd_opt(2026, 10, 31);
        assert!(!cal.runs_on(NaiveDate::from_ymd_opt(2026, 9, 30).unwrap()));
        assert!(!cal.runs_on(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap()));
    }
}
