//! `route_attributes.txt` — BART's internal route classification.
//!
//! This is a BART extension, not part of the core GTFS specification. The
//! integer codes are not published with a key, so they are treated as opaque.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// Attributes describing how a route is operated.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteAttribute {
    /// The route id.
    #[serde(rename = "route_id")]
    pub route_id: String,
    /// An opaque category code. `2` is used for the heavy-rail subway routes.
    #[serde(rename = "category", default)]
    pub category: Option<u32>,
    /// An opaque subcategory code.
    #[serde(rename = "subcategory", default)]
    pub subcategory: Option<u32>,
    /// An opaque running-way code.
    #[serde(rename = "running_way", default)]
    pub running_way: Option<u32>,
}

impl RouteAttribute {
    /// Read `route_attributes.txt` from `path`.
    pub fn parse_route_attributes_file(path: &Path) -> Result<Vec<RouteAttribute>> {
        read_table(path)
    }
}
