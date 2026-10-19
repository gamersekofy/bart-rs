//! `rider_categories.txt` — the rider categories BART recognises.
//!
//! This is a BART extension, not part of the core GTFS specification.

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
    #[serde(rename = "rider_category_description")]
    pub description: String,
}

impl RiderCategory {
    /// Read `rider_categories.txt` from `path`.
    pub fn parse_rider_categories_file(path: &Path) -> Result<Vec<RiderCategory>> {
        read_table(path)
    }
}
