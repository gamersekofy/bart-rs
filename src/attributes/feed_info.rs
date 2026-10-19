//! `feed_info.txt` — metadata about the feed itself.

use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_gtfs_date, read_table};

/// Metadata about the feed: who published it, what it covers, and which
/// version it is.
///
/// The coverage window is what the staleness check is based on: BART
/// guarantees the schedule is valid through `end_date`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedInfo {
    /// The name of the feed publisher.
    #[serde(rename = "feed_publisher_name")]
    pub publisher_name: String,
    /// The publisher's URL.
    #[serde(rename = "feed_publisher_url", default)]
    pub publisher_url: Option<String>,
    /// The feed's primary language.
    #[serde(rename = "feed_lang", default)]
    pub lang: Option<String>,
    /// The first date the feed's schedule covers.
    #[serde(rename = "feed_start_date", default, deserialize_with = "de_gtfs_date")]
    pub start_date: Option<NaiveDate>,
    /// The last date the feed's schedule covers.
    #[serde(rename = "feed_end_date", default, deserialize_with = "de_gtfs_date")]
    pub end_date: Option<NaiveDate>,
    /// The feed version. BART increments this with each release.
    #[serde(rename = "feed_version", default)]
    pub version: Option<String>,
}

impl FeedInfo {
    /// Read `feed_info.txt` from `path`.
    pub fn parse_feed_info_file(path: &Path) -> Result<Vec<FeedInfo>> {
        read_table(path)
    }
}
