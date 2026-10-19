//! `fare_rules.txt` — which fare applies between which places.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A rule saying which fare applies on which part of the network.
///
/// In BART's feed `origin_id` and `destination_id` are **station** ids (such
/// as `SSAN` or `SFIA`), not platform ids. A rule with equal origin and
/// destination is the "anywhere to anywhere" base fare.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FareRule {
    /// The fare id this rule selects.
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    /// The route this rule is limited to, if any.
    #[serde(rename = "route_id", default)]
    pub route_id: Option<String>,
    /// The origin station id.
    #[serde(rename = "origin_id", default)]
    pub origin_id: Option<String>,
    /// The destination station id.
    #[serde(rename = "destination_id", default)]
    pub destination_id: Option<String>,
    /// The fare applies to any stop within this area, if set.
    #[serde(rename = "contains_id", default)]
    pub contains_id: Option<String>,
}

impl FareRule {
    /// Read `fare_rules.txt` from `path`.
    pub fn parse_fare_rules_file(path: &Path) -> Result<Vec<FareRule>> {
        read_table(path)
    }
}
