//! `fare_rider_categories.txt` — discounted prices per rider category.
//!
//! This is a BART extension, not part of the core GTFS specification.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_cents, read_table};

/// The price of a fare for one rider category.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FareRiderCategory {
    /// The fare id this price applies to.
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    /// The rider category id, referenced by `rider_categories`.
    #[serde(rename = "rider_category_id")]
    pub rider_category_id: String,
    /// The price for that category, in cents.
    #[serde(rename = "price", deserialize_with = "de_cents")]
    pub price_cents: i64,
}

impl FareRiderCategory {
    /// Read `fare_rider_categories.txt` from `path`.
    pub fn parse_fare_rider_categories_file(path: &Path) -> Result<Vec<FareRiderCategory>> {
        read_table(path)
    }
}
