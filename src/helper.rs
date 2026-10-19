//! Shared parsing helpers.
//!
//! The GTFS specification has a handful of small but awkward value encodings —
//! times that run past midnight, dates without separators, `0`/`1` booleans,
//! empty strings standing in for nulls. The deserializers in this module turn
//! them into ordinary Rust types, and are used by the `attributes` modules via
//! `#[serde(deserialize_with = "...")]`.

use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

use crate::error::{Error, Result};

/// Strip surrounding double quotes from a string.
///
/// Retained from the original implementation, which split lines on `,` by
/// hand. The `csv` crate already removes CSV quoting, so this is only needed
/// for values that arrive from somewhere other than a CSV reader.
pub fn trim_quotes(s: &str) -> &str {
    s.trim_matches('"')
}

/// Read a GTFS table from `path` into a vector of deserializable rows.
///
/// The reader is strict: a row whose column count does not match the header
/// is an error rather than being silently padded. Every file BART publishes
/// is uniform, so this catches corruption rather than costing us anything.
pub fn read_table<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Err(Error::MissingTable {
            path: path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf()),
            table: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string()),
        });
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(false)
        .from_path(path)
        .map_err(|e| Error::csv(path, e))?;

    let mut rows = Vec::new();
    for record in reader.deserialize() {
        rows.push(record.map_err(|e| Error::csv(path, e))?);
    }
    Ok(rows)
}

/// Parse a GTFS table from CSV text.
///
/// The same strictness as [`read_table`]: a row whose column count does not
/// match the header is an error rather than being silently padded. `name` is
/// only used in error messages.
pub fn parse_table<T: serde::de::DeserializeOwned>(text: &str, name: &str) -> Result<Vec<T>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(false)
        .from_reader(text.as_bytes());

    let mut rows = Vec::new();
    for record in reader.deserialize() {
        rows.push(record.map_err(|e| Error::msg(format!("could not parse {name}: {e}")))?);
    }
    Ok(rows)
}

/// Parse a GTFS time (`HH:MM:SS`) into seconds since midnight.
///
/// GTFS permits hours greater than 24: a train that leaves at 00:30 the
/// following morning is written as `24:30:00`. Such a value is preserved
/// rather than wrapped, because the service day a trip belongs to is exactly
/// the information that wrapping destroys.
pub fn parse_gtfs_time(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut parts = s.split(':');
    let hours: u32 = parts.next()?.trim().parse().ok()?;
    let minutes: u32 = parts.next()?.trim().parse().ok()?;
    let seconds: u32 = parts.next().unwrap_or("0").trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(hours * 3600 + minutes * 60 + seconds)
}

/// Parse a GTFS date (`YYYYMMDD`) into a [`NaiveDate`].
pub fn parse_gtfs_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let year: i32 = s[0..4].parse().ok()?;
    let month: u32 = s[4..6].parse().ok()?;
    let day: u32 = s[6..8].parse().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}

/// Parse a decimal money string into integer cents.
///
/// `7.55` becomes `755`. Money is never held as a float.
pub fn parse_cents(s: &str) -> i64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    match s.split_once('.') {
        Some((whole, fraction)) => {
            let whole: i64 = whole.parse().unwrap_or(0);
            let mut fraction = fraction.to_string();
            fraction.truncate(2);
            while fraction.len() < 2 {
                fraction.push('0');
            }
            whole * 100 + fraction.parse::<i64>().unwrap_or(0)
        }
        None => s.parse::<i64>().unwrap_or(0) * 100,
    }
}

/// Deserialize a GTFS time into seconds since midnight.
pub fn de_gtfs_time<'de, D>(deserializer: D) -> std::result::Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(raw.as_deref().and_then(parse_gtfs_time))
}

/// Deserialize a GTFS date into a [`NaiveDate`].
pub fn de_gtfs_date<'de, D>(deserializer: D) -> std::result::Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(raw.as_deref().and_then(parse_gtfs_date))
}

/// Deserialize GTFS's `0`/`1` boolean encoding.
pub fn de_bool_01<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(matches!(raw.as_deref().map(str::trim), Some("1")))
}

/// Deserialize a decimal money string into integer cents.
pub fn de_cents<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(raw.as_deref().map(parse_cents).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_times_within_the_day() {
        assert_eq!(parse_gtfs_time("05:52:00"), Some(5 * 3600 + 52 * 60));
        assert_eq!(parse_gtfs_time("00:00:00"), Some(0));
        assert_eq!(parse_gtfs_time("23:59:59"), Some(23 * 3600 + 59 * 60 + 59));
    }

    #[test]
    fn preserves_times_past_midnight() {
        assert_eq!(parse_gtfs_time("24:00:00"), Some(86_400));
        assert_eq!(parse_gtfs_time("25:30:00"), Some(91_800));
        assert_eq!(parse_gtfs_time("26:15:45"), Some(94_545));
    }

    #[test]
    fn rejects_nonsense_times() {
        assert_eq!(parse_gtfs_time(""), None);
        assert_eq!(parse_gtfs_time("   "), None);
        assert_eq!(parse_gtfs_time("bogus"), None);
        assert_eq!(parse_gtfs_time("1:2:3:4"), None);
    }

    #[test]
    fn parses_dates() {
        assert_eq!(
            parse_gtfs_date("20260112"),
            NaiveDate::from_ymd_opt(2026, 1, 12)
        );
        assert_eq!(parse_gtfs_date("2026-01-12"), None);
        assert_eq!(parse_gtfs_date("20261312"), None);
        assert_eq!(parse_gtfs_date(""), None);
    }

    #[test]
    fn parses_money_as_cents() {
        assert_eq!(parse_cents("7.55"), 755);
        assert_eq!(parse_cents("0.00"), 0);
        assert_eq!(parse_cents("10.10"), 1010);
        assert_eq!(parse_cents("4"), 400);
        assert_eq!(parse_cents("4.5"), 450);
        assert_eq!(parse_cents(""), 0);
    }

    #[test]
    fn trims_quotes() {
        assert_eq!(trim_quotes("\"hello\""), "hello");
        assert_eq!(trim_quotes("hello"), "hello");
    }
}
