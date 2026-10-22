//! `rider_categories.txt` — the rider categories BART recognises.
//!
//! This is a core GTFS file, optional in the specification, and BART
//! publishes it. BART does not follow the specification exactly, though: the
//! specification calls the second field `rider_category_name`, and BART
//! calls it `rider_category_description`. BART also leaves out
//! `is_default_fare_category`, which the specification requires.
//!
//! We follow BART, because BART is the feed we are given. The reference
//! implementation used as a test oracle follows the specification instead,
//! and so cannot parse this file; see `tests/live_feed.rs`.
//!
//! The specification is at
//! <https://gtfs.org/documentation/schedule/reference/#rider_categoriestxt>.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A category of rider, e.g. `Youth Clipper`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiderCategory {
    /// The rider category id, referenced by `fare_rider_categories`.
    #[serde(rename = "rider_category_id")]
    pub rider_category_id: String,
    /// The human-readable description.
    ///
    /// The specification calls this `rider_category_name`; BART does not.
    #[serde(rename = "rider_category_description")]
    pub description: String,
}

impl RiderCategory {
    /// Read `rider_categories.txt` from `path`.
    pub fn parse_rider_categories_file(path: &Path) -> Result<Vec<RiderCategory>> {
        read_table(path)
    }
}
