//! `directions.txt` — compass direction for a route and direction.
//!
//! This is a BART extension, not part of the core GTFS specification.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// The compass direction of one direction of one route.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Direction {
    /// The route id.
    #[serde(rename = "route_id")]
    pub route_id: String,
    /// The direction id, matching `trips.direction_id`.
    #[serde(rename = "direction_id")]
    pub direction_id: u8,
    /// The direction, e.g. `North` or `South`.
    #[serde(rename = "direction")]
    pub direction: String,
}

impl Direction {
    /// Read `directions.txt` from `path`.
    pub fn parse_directions_file(path: &Path) -> Result<Vec<Direction>> {
        read_table(path)
    }
}
