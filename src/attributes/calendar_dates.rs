//! `calendar_dates.txt` — exceptions to the recurring service patterns.

use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_gtfs_date, read_table};

/// Whether a service is added or removed on a particular date.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum ExceptionType {
    /// The service is added on this date.
    #[serde(rename = "1")]
    Added,
    /// The service is removed on this date.
    #[serde(rename = "2")]
    Removed,
}

/// A single-date exception to a `calendar.txt` service pattern.
///
/// Both kinds occur in BART's feed: a holiday that falls on a Sunday is
/// commonly *added* as a weekday service, and a holiday that falls on a
/// weekday is commonly *removed*.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarDate {
    /// The service id this exception applies to.
    #[serde(rename = "service_id")]
    pub service_id: String,
    /// The date of the exception.
    #[serde(rename = "date", deserialize_with = "de_gtfs_date")]
    pub date: Option<NaiveDate>,
    /// Whether the service is added or removed.
    #[serde(rename = "exception_type")]
    pub exception_type: ExceptionType,
}

impl CalendarDate {
    /// Read `calendar_dates.txt` from `path`.
    pub fn parse_calendar_dates_file(path: &Path) -> Result<Vec<CalendarDate>> {
        read_table(path)
    }
}
